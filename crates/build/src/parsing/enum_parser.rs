use protobuf::descriptor::{EnumDescriptorProto, SourceCodeInfo};

use super::{CodeGenMetadata, EnumInfo, EnumValue};

/// Process a protobuf enum definition
pub(super) fn process_enum(
    enum_desc: &EnumDescriptorProto,
    codegen_metadata: &mut CodeGenMetadata,
    type_prefix: &str,
    source_code_info: Option<&SourceCodeInfo>,
    path_prefix: &[i32],
) -> Result<(), Box<dyn std::error::Error>> {
    let enum_name = enum_desc.name();
    let full_type_name = if type_prefix.is_empty() {
        format!(".{}", enum_name)
    } else {
        format!("{}.{}", type_prefix, enum_name)
    };

    // Extract enum-level documentation
    let enum_documentation = extract_enum_documentation(source_code_info, path_prefix);

    // Process enum values
    let mut values = Vec::new();
    for (value_index, value_desc) in enum_desc.value.iter().enumerate() {
        let value_name = value_desc.name();
        let value_number = value_desc.number();

        // Extract value-level documentation
        let mut value_path = path_prefix.to_vec();
        value_path.extend_from_slice(&[2, value_index as i32]); // value is field 2 in EnumDescriptorProto
        let value_documentation = extract_enum_value_documentation(source_code_info, &value_path);

        values.push(EnumValue {
            name: value_name.to_string(),
            number: value_number,
            documentation: value_documentation,
        });
    }

    let enum_info = EnumInfo {
        name: enum_name.to_string(),
        values,
        documentation: enum_documentation,
    };

    codegen_metadata.enums.insert(full_type_name, enum_info);

    Ok(())
}

/// Extract documentation for an enum from source code info
fn extract_enum_documentation(
    source_code_info: Option<&SourceCodeInfo>,
    path: &[i32],
) -> Option<String> {
    let source_code_info = source_code_info?;

    for location in &source_code_info.location {
        if location.path == path {
            if let Some(leading_comments) = &location.leading_comments {
                let cleaned = clean_comment(leading_comments);
                if !cleaned.is_empty() {
                    return Some(cleaned);
                }
            }
            if let Some(trailing_comments) = &location.trailing_comments {
                let cleaned = clean_comment(trailing_comments);
                if !cleaned.is_empty() {
                    return Some(cleaned);
                }
            }
        }
    }
    None
}

/// Extract documentation for an enum value from source code info
fn extract_enum_value_documentation(
    source_code_info: Option<&SourceCodeInfo>,
    path: &[i32],
) -> Option<String> {
    let source_code_info = source_code_info?;

    for location in &source_code_info.location {
        if location.path == path {
            if let Some(leading_comments) = &location.leading_comments {
                let cleaned = clean_comment(leading_comments);
                if !cleaned.is_empty() {
                    return Some(cleaned);
                }
            }
            if let Some(trailing_comments) = &location.trailing_comments {
                let cleaned = clean_comment(trailing_comments);
                if !cleaned.is_empty() {
                    return Some(cleaned);
                }
            }
        }
    }
    None
}

/// Clean up protobuf comment text
fn clean_comment(comment: &str) -> String {
    comment
        .lines()
        .map(|line| line.trim())
        .map(|line| {
            // Remove leading comment markers
            if line.starts_with("//") {
                line[2..].trim()
            } else if line.starts_with("/*") {
                line[2..].trim()
            } else if line.starts_with("*/") {
                line[2..].trim()
            } else if line.starts_with('*') {
                line[1..].trim()
            } else {
                line
            }
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_comment() {
        assert_eq!(clean_comment("// This is a comment"), "This is a comment");
        assert_eq!(clean_comment("/* Block comment */"), "Block comment");
        assert_eq!(clean_comment("// Line 1\n// Line 2"), "Line 1 Line 2");
        assert_eq!(
            clean_comment("/**\n * Multi-line\n * comment\n */"),
            "Multi-line comment"
        );
    }
}
