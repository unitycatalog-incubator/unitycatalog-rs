use protobuf::descriptor::{EnumDescriptorProto, SourceCodeInfo}; // SourceCodeInfo used as param type

use super::{CodeGenMetadata, EnumInfo, EnumValue, extract_documentation};
use crate::Result;

/// Process a protobuf enum definition
pub(super) fn process_enum(
    enum_desc: &EnumDescriptorProto,
    codegen_metadata: &mut CodeGenMetadata,
    type_prefix: &str,
    source_code_info: Option<&SourceCodeInfo>,
    path_prefix: &[i32],
) -> Result<()> {
    let enum_name = enum_desc.name();
    let full_type_name = if type_prefix.is_empty() {
        format!(".{}", enum_name)
    } else {
        format!("{}.{}", type_prefix, enum_name)
    };

    // Extract enum-level documentation
    let enum_documentation = extract_documentation(source_code_info, path_prefix);

    // Process enum values
    let mut values = Vec::new();
    for (value_index, value_desc) in enum_desc.value.iter().enumerate() {
        let value_name = value_desc.name();
        let value_number = value_desc.number();

        // Extract value-level documentation (values are field 2 in EnumDescriptorProto)
        let value_path = [path_prefix, &[2, value_index as i32]].concat();
        let value_documentation = extract_documentation(source_code_info, &value_path);

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

#[cfg(test)]
mod tests {
    /// Clean up protobuf comment text for testing purposes.
    /// Comments read from compiled `.bin` descriptors are already stripped of
    /// `//`/`/*` markers by `buf`, so this helper is only needed for unit tests.
    fn clean_comment(comment: &str) -> String {
        comment
            .lines()
            .map(|line| line.trim())
            .map(|line| {
                let line = if let Some(s) = line.strip_prefix("/**") {
                    s.trim()
                } else if let Some(s) = line.strip_prefix("//") {
                    s.trim()
                } else if let Some(s) = line.strip_prefix("/*") {
                    s.trim()
                } else if let Some(s) = line.strip_prefix("*/") {
                    s.trim()
                } else if let Some(s) = line.strip_prefix('*') {
                    s.trim()
                } else {
                    line
                };
                if let Some(s) = line.strip_suffix("*/") {
                    s.trim()
                } else {
                    line
                }
            })
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

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
