use prost::Message as _;
use protobuf::Message;
use protobuf::descriptor::{
    DescriptorProto, FieldDescriptorProto, FileDescriptorProto, MethodDescriptorProto,
    ServiceDescriptorProto,
};

use crate::CodeGenMetadata;
use crate::MessageField;
use crate::MessageInfo;
use crate::MethodMetadata;
use crate::gnostic::openapi::v3::Operation;
use crate::google::api::HttpRule;

// Known extension field numbers
const GOOGLE_API_HTTP_EXTENSION: u32 = 72295728; // google.api.http
const GNOSTIC_OPERATION_EXTENSION: u32 = 1143; // gnostic.openapi.v3.operation

/// Process a single protobuf file descriptor
///
/// Extracts all messages, services, and annotations from the file.
/// Collects metadata for code generation.
pub fn process_file_descriptor(
    file_desc: &FileDescriptorProto,
    codegen_metadata: &mut CodeGenMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = file_desc.name();

    // Process messages in the file
    for message in &file_desc.message_type {
        let package_name = file_desc.package();
        let type_prefix = if package_name.is_empty() {
            String::new()
        } else {
            format!(".{}", package_name)
        };
        process_message(message, file_name, codegen_metadata, &type_prefix)?;
    }

    // Process services in the file
    for service in &file_desc.service {
        process_service(service, file_name, codegen_metadata)?;
    }

    Ok(())
}

/// Process a protobuf message definition
fn process_message(
    message: &DescriptorProto,
    file_name: &str,
    codegen_metadata: &mut CodeGenMetadata,
    type_prefix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let message_name = message.name();
    let full_type_name = if type_prefix.is_empty() {
        format!(".{}", message_name)
    } else {
        format!("{}.{}", type_prefix, message_name)
    };

    // Collect field information, handling oneof fields specially
    let mut fields = Vec::new();
    let mut oneof_groups: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    // First pass: collect regular fields and identify oneof groups
    for field in &message.field {
        // In Proto3, fields are optional if:
        // 1. They have the LABEL_OPTIONAL label AND proto3_optional is true, OR
        // 2. They have LABEL_REPEATED (repeated fields are inherently optional)
        // All other fields are required in Proto3
        let is_optional = match field.label() {
            protobuf::descriptor::field_descriptor_proto::Label::LABEL_REPEATED => true,
            protobuf::descriptor::field_descriptor_proto::Label::LABEL_OPTIONAL => {
                // Check if this is a proto3 optional field
                field.proto3_optional()
            }
            protobuf::descriptor::field_descriptor_proto::Label::LABEL_REQUIRED => false,
        };

        // Check if this field belongs to a oneof group
        if field.has_oneof_index() {
            let oneof_index = field.oneof_index() as usize;
            if oneof_index < message.oneof_decl.len() {
                let oneof_name = message.oneof_decl[oneof_index].name().to_string();

                // Skip proto3_optional fields - they're not true oneofs
                if !field.proto3_optional() {
                    oneof_groups
                        .entry(oneof_name)
                        .or_default()
                        .push(field.name().to_string());
                    continue; // Skip adding this field individually
                }
            }
        }

        // Add regular field (including proto3_optional fields)
        let field_info = MessageField {
            name: field.name().to_string(),
            field_type: format_field_type(field),
            optional: is_optional,
            oneof_name: None,
        };
        fields.push(field_info);
    }

    // Second pass: create single fields for each oneof group
    for (oneof_name, _field_names) in oneof_groups {
        // Create a single field representing the oneof enum
        // The enum type name follows the pattern: message_name::OneofName
        let enum_type_name = format!(
            "{}::{}",
            message_name.to_lowercase(),
            capitalize_first_letter(&oneof_name)
        );

        let oneof_field = MessageField {
            name: oneof_name,
            field_type: format!("TYPE_ONEOF:{}", enum_type_name),
            optional: true,   // oneof fields are always optional (Option<enum>)
            oneof_name: None, // This is the oneof field itself, not a member
        };

        fields.push(oneof_field);
    }

    // Store message information
    let message_info = MessageInfo {
        name: full_type_name.clone(),
        fields,
    };
    codegen_metadata
        .messages
        .insert(full_type_name.clone(), message_info);

    // Process nested messages
    for nested_message in &message.nested_type {
        process_message(nested_message, file_name, codegen_metadata, &full_type_name)?;
    }

    Ok(())
}

/// Process a gRPC service definition
fn process_service(
    service: &ServiceDescriptorProto,
    _file_name: &str,
    codegen_metadata: &mut CodeGenMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    let service_name = service.name();

    // Process methods in the service
    for method in &service.method {
        process_method(method, service_name, codegen_metadata)?;
    }

    Ok(())
}

/// Process a gRPC method definition and extract REST API metadata
fn process_method(
    method: &MethodDescriptorProto,
    service_name: &str,
    codegen_metadata: &mut CodeGenMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    let method_name = method.name();
    let input_type = method.input_type();
    let output_type = method.output_type();

    // Get input message fields
    let input_fields = codegen_metadata.get_message_fields(input_type);

    // Initialize method metadata
    let mut method_metadata = MethodMetadata {
        service_name: service_name.to_string(),
        method_name: method_name.to_string(),
        input_type: input_type.to_string(),
        output_type: output_type.to_string(),
        operation: None,
        http_rule: None,
        input_fields,
    };

    // Extract gnostic method-level annotations
    extract_method_annotations(method, service_name, method_name, &mut method_metadata)?;

    // Add to collected metadata
    codegen_metadata.methods.push(method_metadata);

    Ok(())
}

/// Extract gnostic annotations from method-level options
///
/// This is where we extract the key information for REST API generation:
/// - HTTP method and path from google.api.http extension
/// - Operation ID from gnostic.openapi.v3.operation extension
fn extract_method_annotations(
    method: &MethodDescriptorProto,
    service_name: &str,
    method_name: &str,
    method_metadata: &mut MethodMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    if method.options.is_none() {
        println!(
            "cargo:warning=    Method {}.{} has no options",
            service_name, method_name
        );
        return Ok(());
    }

    let options = method.options.as_ref().unwrap();

    let unknown_fields = options.unknown_fields();

    // Process each extension field
    for (field_number, field_value) in unknown_fields.iter() {
        let data = match field_value {
            protobuf::UnknownValueRef::LengthDelimited(bytes) => bytes,
            _ => {
                println!(
                    "cargo:warning=    Skipping non-length-delimited field {}",
                    field_number
                );
                continue;
            }
        };

        match field_number {
            GOOGLE_API_HTTP_EXTENSION => {
                println!(
                    "cargo:warning=    Found google.api.http extension in {}.{}",
                    service_name, method_name
                );

                // Parse HTTP rule from extension data
                match HttpRule::decode(data) {
                    Ok(rule) => {
                        println!(
                            "cargo:warning=      Successfully parsed HTTP rule: {:?}",
                            rule
                        );
                        method_metadata.http_rule = Some(rule);
                    }
                    Err(e) => {
                        println!("cargo:warning=      Failed to parse HTTP rule: {}", e);
                    }
                }
            }
            GNOSTIC_OPERATION_EXTENSION => {
                println!(
                    "cargo:warning=    Found gnostic.openapi.v3.operation extension in {}.{}",
                    service_name, method_name
                );

                // Parse operation from extension data
                match Operation::decode(data) {
                    Ok(operation) => {
                        println!("cargo:warning=      Operation: {:?}", operation);
                        method_metadata.operation = Some(operation);
                    }
                    Err(e) => {
                        println!(
                            "cargo:warning=      Failed to parse gnostic operation: {}",
                            e
                        );
                    }
                }
            }
            _ => {
                // Unknown extension field
                println!(
                    "cargo:warning=    Unknown extension field {} in {}.{}",
                    field_number, service_name, method_name
                );
            }
        }
    }

    Ok(())
}

/// Format a protobuf field type for use in Rust code generation
fn format_field_type(field: &FieldDescriptorProto) -> String {
    use protobuf::descriptor::field_descriptor_proto::Type;

    match field.type_() {
        Type::TYPE_STRING => "TYPE_STRING".to_string(),
        Type::TYPE_INT32 => "TYPE_INT32".to_string(),
        Type::TYPE_INT64 => "TYPE_INT64".to_string(),
        Type::TYPE_BOOL => "TYPE_BOOL".to_string(),
        Type::TYPE_DOUBLE => "TYPE_DOUBLE".to_string(),
        Type::TYPE_FLOAT => "TYPE_FLOAT".to_string(),
        Type::TYPE_BYTES => "TYPE_BYTES".to_string(),
        Type::TYPE_MESSAGE => {
            format!("TYPE_MESSAGE:{}", field.type_name())
        }
        Type::TYPE_ENUM => {
            format!("TYPE_ENUM:{}", field.type_name())
        }
        _ => "TYPE_UNKNOWN".to_string(),
    }
}

/// Capitalize the first letter of a string
fn capitalize_first_letter(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
