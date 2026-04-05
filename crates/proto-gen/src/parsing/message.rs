use std::collections::HashMap;

use convert_case::{Case, Casing};
use prost::Message as _;
use protobuf::Message;
use protobuf::descriptor::field_descriptor_proto::{Label, Type};
use protobuf::descriptor::{DescriptorProto, FieldDescriptorProto, MessageOptions, SourceCodeInfo};

use super::{CodeGenMetadata, MessageField, MessageInfo, OneofVariant};
use crate::google::api::{FieldBehavior, ResourceDescriptor};
use crate::parsing::types::{BaseType, UnifiedType};
use crate::{Error, Result};

/// Process a protobuf message definition
pub(super) fn process_message(
    message: &DescriptorProto,
    _file_name: &str,
    codegen_metadata: &mut CodeGenMetadata,
    type_prefix: &str,
    source_code_info: Option<&SourceCodeInfo>,
    path_prefix: &[i32],
) -> Result<()> {
    let message_name = message.name();
    let full_type_name = if type_prefix.is_empty() {
        format!(".{}", message_name)
    } else {
        format!("{}.{}", type_prefix, message_name)
    };

    // Pre-collect which nested messages are map entries so we can detect map fields
    // without relying on naming heuristics. Protobuf marks map entry messages with
    // `option map_entry = true` in their MessageOptions.
    let map_entries = collect_map_entries(message, &full_type_name);

    let mut fields = Vec::new();
    let mut oneof_fields: HashMap<String, Vec<OneofVariant>> = HashMap::new();

    let field_docs = extract_field_documentation(source_code_info, path_prefix);

    // First pass: collect regular fields and identify oneof groups
    for (field_index, field) in message.field.iter().enumerate() {
        // In Proto3, fields are optional if:
        // 1. They have the LABEL_OPTIONAL label AND proto3_optional is true, OR
        // 2. They have LABEL_REPEATED (repeated fields are inherently optional)
        // All other fields are required in Proto3
        let is_optional = match field.label() {
            Label::LABEL_REPEATED => true,
            Label::LABEL_OPTIONAL => field.proto3_optional(),
            Label::LABEL_REQUIRED => false,
        };

        // Check if this is a repeated field
        let is_repeated = matches!(field.label(), Label::LABEL_REPEATED);

        // Get documentation for this field
        let field_path = [path_prefix, &[2, field_index as i32]].concat();
        let documentation = field_docs.get(&field_path).cloned();

        // Check if this field belongs to a oneof group.
        // Skip proto3_optional fields - they're not true oneofs
        if field.has_oneof_index() && !field.proto3_optional() {
            if let Some(oneof_desc) = message.oneof_decl.get(field.oneof_index() as usize) {
                let field_name = field.name().to_string();

                // Build a UnifiedType for the variant's inner value.
                let field_type = if field.has_type_name() {
                    let clean_type = field.type_name().trim_start_matches('.');
                    let base = if field.type_() == Type::TYPE_ENUM {
                        BaseType::Enum(clean_type.to_string())
                    } else {
                        BaseType::Message(clean_type.to_string())
                    };
                    UnifiedType {
                        base_type: base,
                        is_optional: false,
                        is_repeated: false,
                    }
                } else {
                    let base = match field.type_() {
                        Type::TYPE_STRING => BaseType::String,
                        Type::TYPE_INT32 => BaseType::Int32,
                        Type::TYPE_INT64 => BaseType::Int64,
                        Type::TYPE_BOOL => BaseType::Bool,
                        Type::TYPE_DOUBLE => BaseType::Float64,
                        Type::TYPE_FLOAT => BaseType::Float32,
                        Type::TYPE_BYTES => BaseType::Bytes,
                        _ => BaseType::String,
                    };
                    UnifiedType {
                        base_type: base,
                        is_optional: false,
                        is_repeated: false,
                    }
                };

                let variant = OneofVariant {
                    variant_name: field_name.to_case(Case::Pascal),
                    field_name,
                    field_type,
                    documentation,
                };

                let oneof_name = format!("{}.{}", full_type_name, oneof_desc.name());
                oneof_fields.entry(oneof_name).or_default().push(variant);
                continue;
            }
        }

        // Add regular field (including proto3_optional fields)
        let unified_type = parse_field_to_unified_type(field, &map_entries);

        // Extract field behavior annotations
        let field_behavior = extract_field_behavior_option(field)?;

        let field_info = MessageField {
            name: field.name().to_string(),
            unified_type,
            optional: is_optional,
            repeated: is_repeated,
            documentation,
            field_behavior,
            oneof_name: None,
            oneof_variants: None,
        };
        fields.push(field_info);
    }

    // Second pass: create single fields for each oneof group
    for (oneof_name, variants) in oneof_fields {
        // Create a single field representing the oneof enum.
        // The enum type name follows the pattern: message_name::OneofName
        let oneof_field_name = oneof_name.split('.').next_back().unwrap().to_string();
        let enum_type_name = format!(
            "{}::{}",
            message_name.to_case(Case::Snake),
            oneof_field_name.to_case(Case::Pascal)
        );

        let oneof_field = MessageField {
            // Use the simple field name (e.g. "credential_type") so that downstream
            // format_ident! calls produce valid Rust identifiers.  The full qualified
            // name is preserved in `oneof_name` for callers that need it.
            name: oneof_field_name.clone(),
            unified_type: UnifiedType {
                base_type: BaseType::OneOf(enum_type_name.clone()),
                is_optional: true,
                is_repeated: false,
            },
            oneof_variants: Some(variants),
            documentation: None, // TODO: Extract oneof documentation if needed
            optional: true,      // oneof fields are always optional (Option<enum>)
            repeated: false,     // oneof fields are never repeated
            oneof_name: Some(oneof_name), // The qualified name is still available here
            field_behavior: vec![], // Oneof fields don't have field behavior
        };

        fields.push(oneof_field);
    }

    // Extract message-level options (like google.api.resource)
    let resource_descriptor = extract_message_resource_option(message)?;
    // Extract message-level documentation
    let documentation = extract_message_documentation(source_code_info, path_prefix);

    // Store message information
    let message_info = MessageInfo {
        name: full_type_name.clone(),
        fields,
        resource_descriptor,
        documentation,
    };
    codegen_metadata
        .messages
        .insert(full_type_name.clone(), message_info);

    // Process nested messages
    for (nested_index, nested_message) in message.nested_type.iter().enumerate() {
        let nested_path = [path_prefix, &[3, nested_index as i32]].concat();
        process_message(
            nested_message,
            _file_name,
            codegen_metadata,
            &full_type_name,
            source_code_info,
            &nested_path,
        )?;
    }

    Ok(())
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
fn extract_field_behavior_option(field: &FieldDescriptorProto) -> Result<Vec<FieldBehavior>> {
    if field.options.is_none() {
        return Ok(vec![]);
    }

    let options = field.options.as_ref().unwrap();
    let unknown_fields = options.unknown_fields();

    // Look for the google.api.field_behavior extension
    let mut behaviors = Vec::new();

    for (field_number, field_value) in unknown_fields.iter() {
        if field_number == super::GOOGLE_API_FIELD_BEHAVIOR_EXTENSION {
            match field_value {
                protobuf::UnknownValueRef::Varint(value) => {
                    // Single varint value - this is the common case
                    if let Ok(behavior) = FieldBehavior::try_from(value as i32) {
                        behaviors.push(behavior);
                    }
                }
                protobuf::UnknownValueRef::LengthDelimited(bytes) => {
                    // Packed repeated field - multiple varints in one field
                    let mut cursor = std::io::Cursor::new(bytes);
                    while cursor.position() < bytes.len() as u64 {
                        match decode_varint(&mut cursor) {
                            Ok(value) => {
                                if let Ok(behavior) = FieldBehavior::try_from(value as i32) {
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

/// Extract google.api.resource option from message-level options
///
/// This function extracts the `google.api.resource` extension from protobuf message options.
/// The google.api.resource extension is used to annotate messages that represent resources
/// in REST APIs, providing information such as:
/// - Resource type (e.g., "example.io/Catalog")
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
) -> Result<Option<ResourceDescriptor>> {
    if message.options.is_none() {
        return Ok(None);
    }

    let options = message.options.as_ref().unwrap();
    let unknown_fields = options.unknown_fields();

    // Look for the google.api.resource extension
    for (field_number, field_value) in unknown_fields.iter() {
        if field_number == super::GOOGLE_API_RESOURCE_EXTENSION {
            let data = match field_value {
                protobuf::UnknownValueRef::LengthDelimited(bytes) => bytes,
                _ => {
                    println!("Skipping non-length-delimited google.api.resource field");
                    continue;
                }
            };

            // Parse ResourceDescriptor from extension data
            match ResourceDescriptor::decode(data) {
                Ok(resource_descriptor) => {
                    return Ok(Some(resource_descriptor));
                }
                Err(e) => {
                    return Err(Error::InvalidAnnotation {
                        object: message.name().to_string(),
                        message: format!("Failed to parse google.api.resource: {}", e),
                    });
                }
            }
        }
    }

    Ok(None)
}

/// Extract field documentation from source code info
fn extract_field_documentation(
    source_code_info: Option<&SourceCodeInfo>,
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
    source_code_info: Option<&SourceCodeInfo>,
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

/// Collect nested messages that have `option map_entry = true`.
///
/// Returns a map from the fully-qualified type name of the map entry message
/// to a (key_type, value_type) pair extracted from its `key` / `value` fields.
fn collect_map_entries(
    message: &DescriptorProto,
    parent_full_name: &str,
) -> HashMap<String, (BaseType, BaseType)> {
    let mut entries = HashMap::new();

    for nested in &message.nested_type {
        let is_map_entry = nested
            .options
            .as_ref()
            .is_some_and(|opts: &MessageOptions| opts.map_entry());

        if !is_map_entry {
            continue;
        }

        let entry_name = format!("{}.{}", parent_full_name, nested.name());

        let key_type = nested
            .field
            .iter()
            .find(|f| f.number() == 1)
            .map(proto_field_to_base_type)
            .unwrap_or(BaseType::String);

        let value_type = nested
            .field
            .iter()
            .find(|f| f.number() == 2)
            .map(proto_field_to_base_type)
            .unwrap_or(BaseType::String);

        entries.insert(entry_name, (key_type, value_type));
    }

    entries
}

/// Convert a proto field descriptor to a `BaseType` (for map entry key/value fields).
fn proto_field_to_base_type(field: &FieldDescriptorProto) -> BaseType {
    match field.type_() {
        Type::TYPE_STRING => BaseType::String,
        Type::TYPE_INT32 => BaseType::Int32,
        Type::TYPE_INT64 => BaseType::Int64,
        Type::TYPE_BOOL => BaseType::Bool,
        Type::TYPE_DOUBLE => BaseType::Float64,
        Type::TYPE_FLOAT => BaseType::Float32,
        Type::TYPE_BYTES => BaseType::Bytes,
        Type::TYPE_MESSAGE => {
            let type_name = field.type_name().trim_start_matches('.');
            BaseType::Message(type_name.to_string())
        }
        Type::TYPE_ENUM => {
            let type_name = field.type_name().trim_start_matches('.');
            BaseType::Enum(type_name.to_string())
        }
        _ => BaseType::String,
    }
}

/// Parse a protobuf field directly to UnifiedType.
///
/// `map_entries` provides the set of nested message type names that are
/// protobuf map entries (i.e. `option map_entry = true`), along with their
/// resolved key/value base types.
fn parse_field_to_unified_type(
    field: &FieldDescriptorProto,
    map_entries: &HashMap<String, (BaseType, BaseType)>,
) -> UnifiedType {
    let base_type = match field.type_() {
        Type::TYPE_STRING => BaseType::String,
        Type::TYPE_INT32 => BaseType::Int32,
        Type::TYPE_INT64 => BaseType::Int64,
        Type::TYPE_BOOL => BaseType::Bool,
        Type::TYPE_DOUBLE => BaseType::Float64,
        Type::TYPE_FLOAT => BaseType::Float32,
        Type::TYPE_BYTES => BaseType::Bytes,
        Type::TYPE_MESSAGE => {
            let type_name = field.type_name().trim_start_matches('.');
            if let Some((key_bt, val_bt)) = map_entries.get(field.type_name()) {
                UnifiedType::map(
                    UnifiedType {
                        base_type: key_bt.clone(),
                        is_optional: false,
                        is_repeated: false,
                    },
                    UnifiedType {
                        base_type: val_bt.clone(),
                        is_optional: false,
                        is_repeated: false,
                    },
                )
                .base_type
            } else {
                BaseType::Message(type_name.to_string())
            }
        }
        Type::TYPE_ENUM => {
            let type_name = field.type_name().trim_start_matches('.');
            BaseType::Enum(type_name.to_string())
        }
        _ => BaseType::String,
    };

    // Map types are encoded as repeated fields in proto, but we handle them as maps.
    let is_repeated =
        field.label() == Label::LABEL_REPEATED && !matches!(base_type, BaseType::Map(_, _));
    let is_optional = field.label() == Label::LABEL_OPTIONAL && field.proto3_optional();

    UnifiedType {
        base_type,
        is_optional,
        is_repeated,
    }
}

#[cfg(test)]
mod tests {
    use protobuf::descriptor::{DescriptorProto, FieldDescriptorProto};

    use super::*;

    #[test]
    fn test_extract_message_resource_option_no_options() {
        let mut message = DescriptorProto::new();
        message.set_name("TestMessage".to_string());
        // No options set

        let result = extract_message_resource_option(&message).unwrap();
        assert!(result.is_none());
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
    fn test_message_info_stores_resource_descriptor() {
        // Test that MessageInfo properly stores the resource descriptor
        let resource_descriptor = crate::google::api::ResourceDescriptor {
            r#type: "example.io/Schema".to_string(),
            pattern: vec!["catalogs/{catalog}/schemas/{schema}".to_string()],
            name_field: "name".to_string(),
            ..Default::default()
        };

        let message_info = MessageInfo {
            name: ".example.catalog.v1.Schema".to_string(),
            fields: vec![],
            resource_descriptor: Some(resource_descriptor.clone()),
            documentation: None,
        };

        assert!(message_info.resource_descriptor.is_some());
        let stored = message_info.resource_descriptor.unwrap();
        assert_eq!(stored.r#type, "example.io/Schema");
        assert_eq!(stored.pattern, vec!["catalogs/{catalog}/schemas/{schema}"]);
    }

    #[test]
    fn test_extract_message_documentation() {
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
}
