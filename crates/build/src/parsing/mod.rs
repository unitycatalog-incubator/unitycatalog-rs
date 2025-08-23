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
use std::collections::HashMap;

// Known extension field numbers
const GOOGLE_API_HTTP_EXTENSION: u32 = 72295728; // google.api.http
const GNOSTIC_OPERATION_EXTENSION: u32 = 1143; // gnostic.openapi.v3.operation
const GOOGLE_API_RESOURCE_EXTENSION: u32 = 1053; // google.api.resource
const GOOGLE_API_FIELD_BEHAVIOR_EXTENSION: u32 = 1052; // google.api.field_behavior

/// Process a single protobuf file descriptor
///
/// Extracts all messages, services, and annotations from the file.
/// Collects metadata for code generation.
pub fn process_file_descriptor(
    file_desc: &FileDescriptorProto,
    codegen_metadata: &mut CodeGenMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = file_desc.name();

    // Extract source code info for documentation
    let source_code_info = file_desc.source_code_info.as_ref();

    // Process messages in the file
    for (message_index, message) in file_desc.message_type.iter().enumerate() {
        let package_name = file_desc.package();
        let type_prefix = if package_name.is_empty() {
            String::new()
        } else {
            format!(".{}", package_name)
        };
        process_message(
            message,
            file_name,
            codegen_metadata,
            &type_prefix,
            source_code_info,
            &[4, message_index as i32],
        )?;
    }

    // Process services in the file
    for (service_index, service) in file_desc.service.iter().enumerate() {
        process_service(
            service,
            file_name,
            codegen_metadata,
            source_code_info,
            service_index,
        )?;
    }

    Ok(())
}

/// Process a protobuf message definition
fn process_message(
    message: &DescriptorProto,
    file_name: &str,
    codegen_metadata: &mut CodeGenMetadata,
    type_prefix: &str,
    source_code_info: Option<&protobuf::descriptor::SourceCodeInfo>,
    path_prefix: &[i32],
) -> Result<(), Box<dyn std::error::Error>> {
    let message_name = message.name();
    let full_type_name = if type_prefix.is_empty() {
        format!(".{}", message_name)
    } else {
        format!("{}.{}", type_prefix, message_name)
    };

    // Extract message-level documentation
    let message_documentation = extract_message_documentation(source_code_info, path_prefix);

    // Collect field information, handling oneof fields specially
    let mut fields = Vec::new();
    let mut oneof_groups: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let mut oneof_field_info: std::collections::HashMap<String, Vec<crate::OneofVariant>> =
        std::collections::HashMap::new();

    // Build a map of field paths to documentation
    let field_docs = extract_field_documentation(source_code_info, path_prefix);

    // First pass: collect regular fields and identify oneof groups
    for (field_index, field) in message.field.iter().enumerate() {
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

        // Check if this is a repeated field
        let is_repeated = matches!(
            field.label(),
            protobuf::descriptor::field_descriptor_proto::Label::LABEL_REPEATED
        );

        // Check if this field belongs to a oneof group
        if field.has_oneof_index() {
            let oneof_index = field.oneof_index() as usize;
            if oneof_index < message.oneof_decl.len() {
                let oneof_name = message.oneof_decl[oneof_index].name().to_string();

                // Skip proto3_optional fields - they're not true oneofs
                if !field.proto3_optional() {
                    let field_name = field.name().to_string();
                    oneof_groups
                        .entry(oneof_name.clone())
                        .or_default()
                        .push(field_name.clone());

                    // Collect variant information for oneof fields
                    let field_type_str = match field.type_() {
                        protobuf::descriptor::field_descriptor_proto::Type::TYPE_MESSAGE => {
                            if field.has_type_name() {
                                format!("TYPE_MESSAGE:{}", field.type_name())
                            } else {
                                "TYPE_MESSAGE".to_string()
                            }
                        }
                        _ => format!("TYPE_{:?}", field.type_()),
                    };

                    // Get field documentation
                    let field_path = [path_prefix, &[2, field_index as i32]].concat();
                    let documentation = field_docs.get(&field_path).cloned();

                    // Extract the rust type name from the field type
                    let rust_type = if field.has_type_name() {
                        // Remove leading dot and extract the type name
                        let clean_type = field.type_name().trim_start_matches('.');
                        clean_type
                            .split('.')
                            .next_back()
                            .unwrap_or(clean_type)
                            .to_string()
                    } else {
                        field_name.clone()
                    };

                    // Create variant name by capitalizing the field name segments
                    let variant_name = field_name
                        .split('_')
                        .map(capitalize_first_letter)
                        .collect::<String>();

                    let variant = crate::OneofVariant {
                        field_name: field_name.clone(),
                        variant_name,
                        rust_type,
                        documentation,
                    };

                    oneof_field_info
                        .entry(oneof_name)
                        .or_default()
                        .push(variant);

                    continue; // Skip adding this field individually
                }
            }
        }

        // Add regular field (including proto3_optional fields)
        let field_type_str = format_field_type(field);

        // Get documentation for this field
        let field_path = [path_prefix, &[2, field_index as i32]].concat();
        let documentation = field_docs.get(&field_path).cloned();

        // Extract field behavior annotations
        let field_behavior = extract_field_behavior_option(field)?;

        let field_info = MessageField {
            name: field.name().to_string(),
            field_type: field_type_str,
            optional: is_optional,
            repeated: is_repeated,
            oneof_name: None,
            documentation,
            oneof_variants: None,
            field_behavior,
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

        // Get the collected variant information for this oneof
        let variants = oneof_field_info.get(&oneof_name).cloned();

        let oneof_field = MessageField {
            name: oneof_name.clone(),
            field_type: format!("TYPE_ONEOF:{}", enum_type_name),
            optional: true,      // oneof fields are always optional (Option<enum>)
            repeated: false,     // oneof fields are never repeated
            oneof_name: None,    // This is the oneof field itself, not a member
            documentation: None, // TODO: Extract oneof documentation if needed
            oneof_variants: variants.clone(),
            field_behavior: vec![], // Oneof fields don't have field behavior
        };

        fields.push(oneof_field);
    }

    // Extract message-level options (like google.api.resource)
    let resource_descriptor = extract_message_resource_option(message)?;

    // Store message information
    let message_info = MessageInfo {
        name: full_type_name.clone(),
        fields,
        resource_descriptor,
        documentation: message_documentation,
    };
    codegen_metadata
        .messages
        .insert(full_type_name.clone(), message_info);

    // Process nested messages
    for (nested_index, nested_message) in message.nested_type.iter().enumerate() {
        let nested_path = [path_prefix, &[3, nested_index as i32]].concat();
        process_message(
            nested_message,
            file_name,
            codegen_metadata,
            &full_type_name,
            source_code_info,
            &nested_path,
        )?;
    }

    Ok(())
}

/// Process a protobuf service definition
fn process_service(
    service: &ServiceDescriptorProto,
    file_name: &str,
    codegen_metadata: &mut CodeGenMetadata,
    source_code_info: Option<&protobuf::descriptor::SourceCodeInfo>,
    service_index: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let service_name = service.name();

    // Extract service-level documentation
    let service_path = vec![6, service_index as i32]; // Services are at path [6, service_index]
    let service_documentation = extract_service_documentation(source_code_info, &service_path);

    // Store service information
    let service_info = crate::ServiceInfo {
        name: service_name.to_string(),
        documentation: service_documentation,
        methods: Vec::new(),
    };
    codegen_metadata
        .services
        .insert(service_name.to_string(), service_info);

    // Process methods in the service
    for (method_index, method) in service.method.iter().enumerate() {
        process_method(
            method,
            service_name,
            codegen_metadata,
            source_code_info,
            service_index,
            method_index,
        )?;
    }

    Ok(())
}

/// Process a gRPC method definition and extract REST API metadata
fn process_method(
    method: &MethodDescriptorProto,
    service_name: &str,
    codegen_metadata: &mut CodeGenMetadata,
    source_code_info: Option<&protobuf::descriptor::SourceCodeInfo>,
    service_index: usize,
    method_index: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let method_name = method.name();
    let input_type = method.input_type();
    let output_type = method.output_type();

    // Extract method-level documentation
    let method_path = vec![6, service_index as i32, 2, method_index as i32]; // Methods are at path [6, service_index, 2, method_index]
    let method_documentation = extract_method_documentation(source_code_info, &method_path);

    // Get input message fields
    let input_fields = codegen_metadata.get_message_fields(input_type);

    // Extract gnostic method-level annotations first to get required http_rule
    let (operation, http_rule) = extract_method_annotations(method, service_name, method_name)?;

    // Initialize method metadata with required http_rule
    let method_metadata = MethodMetadata {
        service_name: service_name.to_string(),
        method_name: method_name.to_string(),
        input_type: input_type.to_string(),
        output_type: output_type.to_string(),
        operation,
        http_rule,
        input_fields,
        documentation: method_documentation,
    };

    // Add to the service's methods
    if let Some(service_info) = codegen_metadata.services.get_mut(service_name) {
        service_info.methods.push(method_metadata);
    }

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
) -> Result<(Option<Operation>, HttpRule), Box<dyn std::error::Error>> {
    if method.options.is_none() {
        return Err(format!(
            "Method {}.{} has no options - HTTP rule annotation is required",
            service_name, method_name
        )
        .into());
    }

    let options = method.options.as_ref().unwrap();
    let unknown_fields = options.unknown_fields();

    let mut operation = None;
    let mut http_rule = None;

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
                        http_rule = Some(rule);
                    }
                    Err(e) => {
                        return Err(format!(
                            "Failed to parse HTTP rule for {}.{}: {}",
                            service_name, method_name, e
                        )
                        .into());
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
                    Ok(op) => {
                        println!("cargo:warning=      Operation: {:?}", op);
                        operation = Some(op);
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

    // Ensure HTTP rule was found
    let http_rule = http_rule.ok_or_else(|| {
        format!(
            "Method {}.{} is missing required google.api.http annotation",
            service_name, method_name
        )
    })?;

    Ok((operation, http_rule))
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

/// Extract google.api.resource option from message-level options
///
/// This function extracts the `google.api.resource` extension from protobuf message options.
/// The google.api.resource extension is used to annotate messages that represent resources
/// in REST APIs, providing information such as:
/// - Resource type (e.g., "unitycatalog.io/Catalog")
/// - URL patterns for the resource (e.g., "catalogs/{catalog}")
/// - Name field for the resource
///
/// This information is essential for generating REST API client libraries and documentation
/// that conform to Google's API Resource Model.
///
/// # Returns
/// - `Ok(Some(ResourceDescriptor))` if the extension is found and parsed successfully
/// - `Ok(None)` if no google.api.resource extension is present
/// - `Err(...)` if there's an error parsing the extension data
fn extract_message_resource_option(
    message: &DescriptorProto,
) -> Result<Option<crate::google::api::ResourceDescriptor>, Box<dyn std::error::Error>> {
    if message.options.is_none() {
        return Ok(None);
    }

    let options = message.options.as_ref().unwrap();
    let unknown_fields = options.unknown_fields();

    // Look for the google.api.resource extension
    for (field_number, field_value) in unknown_fields.iter() {
        if field_number == GOOGLE_API_RESOURCE_EXTENSION {
            let data = match field_value {
                protobuf::UnknownValueRef::LengthDelimited(bytes) => bytes,
                _ => {
                    println!(
                        "cargo:warning=    Skipping non-length-delimited google.api.resource field"
                    );
                    continue;
                }
            };

            // Parse ResourceDescriptor from extension data
            match crate::google::api::ResourceDescriptor::decode(data) {
                Ok(resource_descriptor) => {
                    return Ok(Some(resource_descriptor));
                }
                Err(e) => {
                    println!(
                        "cargo:warning=    Failed to parse google.api.resource: {}",
                        e
                    );
                }
            }
        }
    }

    Ok(None)
}

/// Extract google.api.field_behavior option from field-level options
///
/// This function extracts the `google.api.field_behavior` extension from protobuf field options.
/// The google.api.field_behavior extension is used to annotate fields with behavioral
/// information such as:
/// - REQUIRED: Field must be provided in requests
/// - OPTIONAL: Field is explicitly optional (for emphasis)
/// - OUTPUT_ONLY: Field is only included in responses
/// - INPUT_ONLY: Field is only included in requests
/// - IMMUTABLE: Field can only be set once during creation
/// - IDENTIFIER: Field is used as a unique identifier
/// - UNORDERED_LIST: Repeated field order is not guaranteed
/// - NON_EMPTY_DEFAULT: Field returns non-empty default if not set
///
/// # Returns
/// - `Ok(Vec<FieldBehavior>)` containing all field behaviors found
/// - `Err(...)` if there's an error parsing the extension data
fn extract_field_behavior_option(
    field: &FieldDescriptorProto,
) -> Result<Vec<crate::google::api::FieldBehavior>, Box<dyn std::error::Error>> {
    if field.options.is_none() {
        return Ok(vec![]);
    }

    let options = field.options.as_ref().unwrap();
    let unknown_fields = options.unknown_fields();

    // Look for the google.api.field_behavior extension
    let mut behaviors = Vec::new();

    for (field_number, field_value) in unknown_fields.iter() {
        if field_number == GOOGLE_API_FIELD_BEHAVIOR_EXTENSION {
            match field_value {
                protobuf::UnknownValueRef::Varint(value) => {
                    // Single varint value - this is the common case
                    if let Ok(behavior) = crate::google::api::FieldBehavior::try_from(value as i32)
                    {
                        behaviors.push(behavior);
                    }
                }
                protobuf::UnknownValueRef::LengthDelimited(bytes) => {
                    // Packed repeated field - multiple varints in one field
                    let mut cursor = std::io::Cursor::new(bytes);
                    while cursor.position() < bytes.len() as u64 {
                        match decode_varint(&mut cursor) {
                            Ok(value) => {
                                if let Ok(behavior) =
                                    crate::google::api::FieldBehavior::try_from(value as i32)
                                {
                                    behaviors.push(behavior);
                                }
                            }
                            Err(_) => break,
                        }
                    }
                }
                _ => {
                    // Skip unsupported field types
                }
            }
        }
    }

    if !behaviors.is_empty() {
        return Ok(behaviors);
    }

    Ok(vec![])
}

/// Decode a varint from the given cursor
fn decode_varint(cursor: &mut std::io::Cursor<&[u8]>) -> Result<u64, std::io::Error> {
    let mut result = 0u64;
    let mut shift = 0;

    loop {
        if cursor.position() >= cursor.get_ref().len() as u64 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Unexpected end of data while reading varint",
            ));
        }

        let byte = cursor.get_ref()[cursor.position() as usize];
        cursor.set_position(cursor.position() + 1);

        result |= ((byte & 0x7F) as u64) << shift;

        if (byte & 0x80) == 0 {
            break;
        }

        shift += 7;
        if shift >= 64 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Varint too long",
            ));
        }
    }

    Ok(result)
}

/// Extract field documentation from source code info
fn extract_field_documentation(
    source_code_info: Option<&protobuf::descriptor::SourceCodeInfo>,
    message_path: &[i32],
) -> HashMap<Vec<i32>, String> {
    let mut field_docs = HashMap::new();

    if let Some(sci) = source_code_info {
        for location in &sci.location {
            if location.path.len() >= message_path.len() + 2 {
                // Check if this path starts with our message path and has field info
                let path_slice = &location.path[..message_path.len()];
                if path_slice == message_path && location.path[message_path.len()] == 2 {
                    // This is a field (type 2) in our message
                    let mut documentation = String::new();

                    // Prefer leading comments, fall back to trailing comments
                    if location.has_leading_comments() {
                        documentation = location.leading_comments().trim().to_string();
                    } else if location.has_trailing_comments() {
                        documentation = location.trailing_comments().trim().to_string();
                    }

                    if !documentation.is_empty() {
                        field_docs.insert(location.path.clone(), documentation);
                    }
                }
            }
        }
    }

    field_docs
}

/// Extract documentation for a message at the given path
fn extract_message_documentation(
    source_code_info: Option<&protobuf::descriptor::SourceCodeInfo>,
    message_path: &[i32],
) -> Option<String> {
    if let Some(sci) = source_code_info {
        for location in &sci.location {
            if location.path == message_path {
                let mut documentation = String::new();

                // Prefer leading comments, fall back to trailing comments
                if location.has_leading_comments() {
                    documentation = location.leading_comments().trim().to_string();
                } else if location.has_trailing_comments() {
                    documentation = location.trailing_comments().trim().to_string();
                }

                if !documentation.is_empty() {
                    return Some(documentation);
                }
            }
        }
    }
    None
}

/// Extract documentation for a service at the given path
fn extract_service_documentation(
    source_code_info: Option<&protobuf::descriptor::SourceCodeInfo>,
    service_path: &[i32],
) -> Option<String> {
    if let Some(sci) = source_code_info {
        for location in &sci.location {
            if location.path == service_path {
                let mut documentation = String::new();

                // Prefer leading comments, fall back to trailing comments
                if location.has_leading_comments() {
                    documentation = location.leading_comments().trim().to_string();
                } else if location.has_trailing_comments() {
                    documentation = location.trailing_comments().trim().to_string();
                }

                if !documentation.is_empty() {
                    return Some(documentation);
                }
            }
        }
    }
    None
}

/// Extract documentation for a method at the given path
fn extract_method_documentation(
    source_code_info: Option<&protobuf::descriptor::SourceCodeInfo>,
    method_path: &[i32],
) -> Option<String> {
    if let Some(sci) = source_code_info {
        for location in &sci.location {
            if location.path == method_path {
                let mut documentation = String::new();

                // Prefer leading comments, fall back to trailing comments
                if location.has_leading_comments() {
                    documentation = location.leading_comments().trim().to_string();
                } else if location.has_trailing_comments() {
                    documentation = location.trailing_comments().trim().to_string();
                }

                if !documentation.is_empty() {
                    return Some(documentation);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use protobuf::descriptor::{DescriptorProto, FieldDescriptorProto};

    #[test]
    fn test_extract_message_resource_option_no_options() {
        let mut message = DescriptorProto::new();
        message.set_name("TestMessage".to_string());
        // No options set

        let result = extract_message_resource_option(&message).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_message_info_stores_resource_descriptor() {
        // Test that MessageInfo properly stores the resource descriptor
        let resource_descriptor = crate::google::api::ResourceDescriptor {
            r#type: "unitycatalog.io/Schema".to_string(),
            pattern: vec!["catalogs/{catalog}/schemas/{schema}".to_string()],
            name_field: "name".to_string(),
            ..Default::default()
        };

        let message_info = MessageInfo {
            name: ".unitycatalog.schemas.v1.SchemaInfo".to_string(),
            fields: vec![],
            resource_descriptor: Some(resource_descriptor.clone()),
            documentation: None,
        };

        assert!(message_info.resource_descriptor.is_some());
        let stored = message_info.resource_descriptor.unwrap();
        assert_eq!(stored.r#type, "unitycatalog.io/Schema");
        assert_eq!(stored.pattern, vec!["catalogs/{catalog}/schemas/{schema}"]);
    }

    #[test]
    fn test_google_api_resource_extension_constant() {
        // Verify the extension field number is correct
        assert_eq!(GOOGLE_API_RESOURCE_EXTENSION, 1053);
    }

    #[test]
    fn test_google_api_field_behavior_extension_constant() {
        // Verify the extension field number is correct
        assert_eq!(GOOGLE_API_FIELD_BEHAVIOR_EXTENSION, 1052);
    }

    #[test]
    fn test_extract_field_behavior_option_no_options() {
        let field = FieldDescriptorProto::new();
        let result = extract_field_behavior_option(&field).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_field_behavior_option_function_exists() {
        // Test that the function can be called and handles empty field
        let field = FieldDescriptorProto::new();
        let result = extract_field_behavior_option(&field);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_extract_message_documentation() {
        use protobuf::descriptor::SourceCodeInfo;

        // Create mock source code info with message documentation
        let mut sci = SourceCodeInfo::new();
        let mut location = protobuf::descriptor::source_code_info::Location::new();
        location.path = vec![4, 0]; // Message at index 0
        location.set_leading_comments("This is a test message for documentation.".to_string());
        sci.location.push(location);

        let message_path = vec![4, 0];
        let result = extract_message_documentation(Some(&sci), &message_path);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "This is a test message for documentation.");
    }

    #[test]
    fn test_extract_service_documentation() {
        use protobuf::descriptor::SourceCodeInfo;

        // Create mock source code info with service documentation
        let mut sci = SourceCodeInfo::new();
        let mut location = protobuf::descriptor::source_code_info::Location::new();
        location.path = vec![6, 0]; // Service at index 0
        location.set_leading_comments("This is a test service for documentation.".to_string());
        sci.location.push(location);

        let service_path = vec![6, 0];
        let result = extract_service_documentation(Some(&sci), &service_path);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "This is a test service for documentation.");
    }

    #[test]
    fn test_extract_method_documentation() {
        use protobuf::descriptor::SourceCodeInfo;

        // Create mock source code info with method documentation
        let mut sci = SourceCodeInfo::new();
        let mut location = protobuf::descriptor::source_code_info::Location::new();
        location.path = vec![6, 0, 2, 0]; // Method 0 in service 0
        location.set_leading_comments("This method does something useful.".to_string());
        sci.location.push(location);

        let method_path = vec![6, 0, 2, 0];
        let result = extract_method_documentation(Some(&sci), &method_path);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "This method does something useful.");
    }

    #[test]
    fn test_message_field_includes_field_behavior() {
        // Test that MessageField properly stores field behavior
        let field_behavior = vec![
            crate::google::api::FieldBehavior::Required,
            crate::google::api::FieldBehavior::OutputOnly,
        ];

        let message_field = MessageField {
            name: "test_field".to_string(),
            field_type: "TYPE_STRING".to_string(),
            optional: false,
            repeated: false,
            oneof_name: None,
            documentation: None,
            oneof_variants: None,
            field_behavior: field_behavior.clone(),
        };

        assert_eq!(message_field.field_behavior.len(), 2);
        assert!(
            message_field
                .field_behavior
                .contains(&crate::google::api::FieldBehavior::Required)
        );
        assert!(
            message_field
                .field_behavior
                .contains(&crate::google::api::FieldBehavior::OutputOnly)
        );
    }

    #[test]
    fn test_field_behavior_types() {
        // Test different field behavior variants
        use crate::google::api::FieldBehavior;

        let behaviors = vec![
            FieldBehavior::Required,
            FieldBehavior::Optional,
            FieldBehavior::OutputOnly,
            FieldBehavior::InputOnly,
            FieldBehavior::Immutable,
            FieldBehavior::Identifier,
            FieldBehavior::UnorderedList,
            FieldBehavior::NonEmptyDefault,
        ];

        // Verify all enum variants are accessible
        assert_eq!(behaviors.len(), 8);

        // Test conversion to/from i32 values
        for behavior in behaviors {
            let value = behavior as i32;
            let converted_back = FieldBehavior::try_from(value);
            assert!(converted_back.is_ok());
            assert_eq!(converted_back.unwrap(), behavior);
        }
    }

    #[test]
    fn test_missing_http_rule_causes_error() {
        use protobuf::descriptor::{MethodDescriptorProto, MethodOptions};

        // Create a method without any HTTP rule annotation
        let method = MethodDescriptorProto {
            name: Some("TestMethod".to_string()),
            input_type: Some(".test.TestRequest".to_string()),
            output_type: Some(".test.TestResponse".to_string()),
            options: Some(MethodOptions::default()).into(), // Empty options, no HTTP rule
            ..Default::default()
        };

        // This should return an error since HTTP rule is required
        let result = extract_method_annotations(&method, "TestService", "TestMethod");
        assert!(result.is_err());

        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("missing required google.api.http annotation"));
    }

    #[test]
    fn test_method_without_options_causes_error() {
        use protobuf::descriptor::MethodDescriptorProto;

        // Create a method without any options at all
        let method = MethodDescriptorProto {
            name: Some("TestMethod".to_string()),
            input_type: Some(".test.TestRequest".to_string()),
            output_type: Some(".test.TestResponse".to_string()),
            options: None.into(), // No options at all
            ..Default::default()
        };

        // This should return an error since HTTP rule is required
        let result = extract_method_annotations(&method, "TestService", "TestMethod");
        assert!(result.is_err());

        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("has no options - HTTP rule annotation is required"));
    }
}
