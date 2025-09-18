use prost::Message as _;
use protobuf::Message;
use protobuf::descriptor::{MethodDescriptorProto, ServiceDescriptorProto, SourceCodeInfo};

use super::{CodeGenMetadata, MethodMetadata, ServiceInfo};
use crate::gnostic::openapi::v3::Operation;
use crate::google::api::HttpRule;
use crate::{Error, Result};

/// Process a protobuf service definition
pub(super) fn process_service(
    service: &ServiceDescriptorProto,
    codegen_metadata: &mut CodeGenMetadata,
    source_code_info: Option<&SourceCodeInfo>,
    service_index: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let service_name = service.name();

    // Extract service-level documentation
    let service_path = vec![6, service_index as i32]; // Services are at path [6, service_index]
    let service_documentation = extract_service_documentation(source_code_info, &service_path);

    // Store service information
    let service_info = ServiceInfo {
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
    source_code_info: Option<&SourceCodeInfo>,
    service_index: usize,
    method_index: usize,
) -> Result<()> {
    let method_name = method.name();
    let input_type = method.input_type();
    let output_type = method.output_type();

    // Extract method-level documentation
    // Methods are at path [6, service_index, 2, method_index]
    let method_path = vec![6, service_index as i32, 2, method_index as i32];
    let method_documentation = extract_method_documentation(source_code_info, &method_path);

    // Get input message fields
    let input_fields = codegen_metadata.get_message_fields(input_type);

    // Extract gnostic method-level annotations first to get required http_rule
    let (operation, http_rule) = extract_method_annotations(method, service_name)?;

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
) -> Result<(Option<Operation>, HttpRule)> {
    if method.options.is_none() {
        return Err(Error::MissingAnnotation {
            object: method.name().to_string(),
            message: "missing required google.api.http annotation".to_string(),
        });
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
                println!("Skipping non-length-delimited field {}", field_number);
                continue;
            }
        };

        match field_number {
            super::GOOGLE_API_HTTP_EXTENSION => {
                // Parse HTTP rule from extension data
                match HttpRule::decode(data) {
                    Ok(rule) => {
                        http_rule = Some(rule);
                    }
                    Err(e) => {
                        return Err(Error::InvalidAnnotation {
                            object: method.name().to_string(),
                            message: format!(
                                "Failed to parse HTTP rule for {}.{}: {}",
                                service_name,
                                method.name(),
                                e
                            ),
                        });
                    }
                }
            }
            super::GNOSTIC_OPERATION_EXTENSION => {
                // Parse operation from extension data
                match Operation::decode(data) {
                    Ok(op) => {
                        operation = Some(op);
                    }
                    Err(e) => {
                        println!("Failed to parse gnostic operation: {}", e);
                    }
                }
            }
            _ => {
                // Unknown extension field
                println!(
                    "Unknown extension field {} in {}.{}",
                    field_number,
                    service_name,
                    method.name()
                );
            }
        }
    }

    // Ensure HTTP rule was found
    let http_rule = http_rule.ok_or_else(|| Error::MissingAnnotation {
        object: method.name().to_string(),
        message: "missing required google.api.http annotation".to_string(),
    })?;

    Ok((operation, http_rule))
}

/// Extract documentation for a service at the given path
fn extract_service_documentation(
    source_code_info: Option<&SourceCodeInfo>,
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
    source_code_info: Option<&SourceCodeInfo>,
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
    use protobuf::descriptor::MethodOptions;

    use super::*;

    #[test]
    fn test_extract_service_documentation() {
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
    fn test_missing_http_rule_causes_error() {
        // Create a method without any HTTP rule annotation
        let method = MethodDescriptorProto {
            name: Some("TestMethod".to_string()),
            input_type: Some(".test.TestRequest".to_string()),
            output_type: Some(".test.TestResponse".to_string()),
            options: Some(MethodOptions::default()).into(), // Empty options, no HTTP rule
            ..Default::default()
        };

        // This should return an error since HTTP rule is required
        let result = extract_method_annotations(&method, "TestService");
        assert!(result.is_err());

        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("missing required google.api.http annotation"));
    }

    #[test]
    fn test_method_without_options_causes_error() {
        // Create a method without any options at all
        let method = MethodDescriptorProto {
            name: Some("TestMethod".to_string()),
            input_type: Some(".test.TestRequest".to_string()),
            output_type: Some(".test.TestResponse".to_string()),
            options: None.into(), // No options at all
            ..Default::default()
        };

        // This should return an error since HTTP rule is required
        let result = extract_method_annotations(&method, "TestService");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::MissingAnnotation { .. }
        ));
    }
}
