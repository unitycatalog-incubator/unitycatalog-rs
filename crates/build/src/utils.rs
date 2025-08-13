//! Shared utilities for the build crate
//!
//! This module contains common functions used across different parts of the code generation
//! pipeline to reduce duplication and improve maintainability.

use crate::MessageField;
use convert_case::{Case, Casing};

/// String manipulation utilities
pub mod strings {
    use super::*;

    /// Convert service name to handler trait name
    /// e.g., "CatalogsService" -> "CatalogHandler"
    pub fn service_to_handler_name(service_name: &str) -> String {
        if let Some(base) = service_name.strip_suffix("Service") {
            format!("{}Handler", base.trim_end_matches('s'))
        } else {
            format!("{}Handler", service_name)
        }
    }

    /// Convert operation ID to handler method name
    /// e.g., "ListCatalogs" -> "list_catalogs"
    pub fn operation_to_method_name(operation_id: &str) -> String {
        operation_id.to_case(Case::Snake)
    }

    /// Convert operation ID to route handler function name
    /// e.g., "ListCatalogs" -> "list_catalogs_handler"
    pub fn operation_to_route_name(operation_id: &str) -> String {
        format!("{}_handler", operation_to_method_name(operation_id))
    }

    /// Extract base path from service name
    /// e.g., "CatalogsService" -> "catalogs"
    pub fn service_to_base_path(service_name: &str) -> String {
        if let Some(base) = service_name.strip_suffix("Service") {
            base.to_case(Case::Snake)
        } else {
            service_name.to_case(Case::Snake)
        }
    }

    /// Extract the simple type name from a protobuf type
    /// e.g., ".unitycatalog.catalogs.v1.CatalogInfo" -> "CatalogInfo"
    pub fn extract_simple_type_name(full_type: &str) -> String {
        full_type
            .split('.')
            .next_back()
            .unwrap_or(full_type)
            .to_string()
    }

    /// Convert protobuf field name to Rust identifier
    /// e.g., "full_name" -> "full_name", "someField" -> "some_field"
    pub fn to_rust_field_name(proto_name: &str) -> String {
        proto_name.to_case(Case::Snake)
    }
}

/// Path parameter extraction and matching utilities
pub mod paths {
    use super::*;

    /// Extract path parameter names from URL template like "/catalogs/{name}"
    pub fn extract_path_parameters(path_template: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut chars = path_template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut param = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '}' {
                        chars.next(); // consume the '}'
                        break;
                    }
                    param.push(chars.next().unwrap());
                }
                if !param.is_empty() {
                    params.push(param);
                }
            }
        }

        params
    }

    /// Find matching field for a path parameter with fallback logic
    pub fn find_matching_field_for_path_param<'a>(
        path_param_name: &str,
        input_fields: &'a [MessageField],
    ) -> Option<&'a MessageField> {
        // First try exact match
        if let Some(field) = input_fields.iter().find(|f| f.name == path_param_name) {
            return Some(field);
        }

        // Try common fallback patterns based on Unity Catalog API conventions
        match path_param_name {
            "name" => {
                // For {name}, try full_name as fallback (common in schema operations)
                input_fields.iter().find(|f| f.name == "full_name")
            }
            "full_name" => {
                // For {full_name}, try name as fallback
                input_fields.iter().find(|f| f.name == "name")
            }
            _ => None,
        }
    }

    /// Generate URL formatting code for templates with path parameters
    pub fn format_url_template(path: &str, param_names: &[String]) -> (String, Vec<String>) {
        if param_names.is_empty() {
            return (path.to_string(), Vec::new());
        }

        let mut format_string = path.to_string();
        let mut format_args = Vec::new();

        for param_name in param_names {
            let placeholder = format!("{{{}}}", param_name);
            if format_string.contains(&placeholder) {
                format_string = format_string.replace(&placeholder, "{}");
                format_args.push(param_name.clone());
            }
        }

        (format_string, format_args)
    }
}

/// Type mapping utilities
pub mod types {
    use crate::RequestType;

    /// Convert protobuf field type to Rust type
    pub fn field_type_to_rust_type(field_type: &str) -> String {
        match field_type {
            "TYPE_STRING" => "String".to_string(),
            "TYPE_INT32" => "i32".to_string(),
            "TYPE_INT64" => "i64".to_string(),
            "TYPE_BOOL" => "bool".to_string(),
            "TYPE_DOUBLE" => "f64".to_string(),
            "TYPE_FLOAT" => "f32".to_string(),
            "TYPE_BYTES" => "Vec<u8>".to_string(),
            _ => {
                // For message types, extract the simple name
                if field_type.starts_with("TYPE_MESSAGE:") {
                    super::strings::extract_simple_type_name(&field_type[13..])
                } else if field_type.starts_with("TYPE_ENUM:") {
                    // Enum fields are represented as i32 in the generated structs
                    "i32".to_string()
                } else if let Some(oneof_name) = field_type.strip_prefix("TYPE_ONEOF:") {
                    // Extract the oneof name and create an enum type
                    super::strings::extract_simple_type_name(oneof_name)
                } else {
                    // Default to String for unknown types
                    "String".to_string()
                }
            }
        }
    }

    /// Determine Rust type for path parameters
    pub fn path_param_type(param_name: &str) -> String {
        match param_name {
            "id" => "uuid::Uuid".to_string(),
            _ => "String".to_string(),
        }
    }

    /// Determine if a request type should have a response body
    pub fn has_response_body(request_type: &RequestType) -> bool {
        !matches!(request_type, RequestType::Delete)
    }

    /// Make a type optional by wrapping in Option<T>
    pub fn make_optional(rust_type: &str) -> String {
        format!("Option<{}>", rust_type)
    }
}

/// Validation utilities
pub mod validation {
    use crate::codegen::{GenerationPlan, MethodPlan};

    /// Validate that a generation plan is complete and correct
    pub fn validate_plan(plan: &GenerationPlan) -> Result<(), Box<dyn std::error::Error>> {
        let mut errors = Vec::new();

        // Check that all services have at least one method
        for service in &plan.services {
            if service.methods.is_empty() {
                errors.push(format!("Service {} has no methods", service.service_name));
            }

            // Check that all methods have required information
            for method in &service.methods {
                validate_method_plan(method, &mut errors);
            }
        }

        if !errors.is_empty() {
            return Err(format!("Validation errors: {}", errors.join(", ")).into());
        }

        Ok(())
    }

    fn validate_method_plan(method: &MethodPlan, errors: &mut Vec<String>) {
        if method.handler_function_name.is_empty() {
            errors.push(format!(
                "Method {} has empty handler function name",
                method.metadata.method_name
            ));
        }

        if method.http_method.is_empty() {
            errors.push(format!(
                "Method {} has empty HTTP method",
                method.metadata.method_name
            ));
        }

        if method.http_path.is_empty() {
            errors.push(format!(
                "Method {} has empty HTTP path",
                method.metadata.method_name
            ));
        }

        // Validate that path parameters in URL match extracted parameters
        let url_params = super::paths::extract_path_parameters(&method.http_path);
        if url_params.len() != method.path_params.len() {
            errors.push(format!(
                "Method {} has mismatched path parameters: URL has {}, extracted {}",
                method.metadata.method_name,
                url_params.len(),
                method.path_params.len()
            ));
        }
    }
}

/// Request classification utilities
pub mod requests {
    use crate::RequestType;

    /// Determine request type from operation ID or method name
    pub fn classify_request_type(operation_id: Option<&str>, method_name: &str) -> RequestType {
        let name = operation_id.unwrap_or(method_name);

        if name.starts_with("List") {
            RequestType::List
        } else if name.starts_with("Create") || name.starts_with("Generate") {
            RequestType::Create
        } else if name.starts_with("Get") {
            RequestType::Get
        } else if name.starts_with("Update") || name.starts_with("Query") {
            RequestType::Update
        } else if name.starts_with("Delete") {
            RequestType::Delete
        } else {
            RequestType::Get // Default fallback
        }
    }

    /// Determine if a field should be extracted from request body
    pub fn should_be_body_field(field_name: &str, body_spec: &str) -> bool {
        match body_spec {
            "*" => true, // All fields not in path go to body
            "" => false, // No body fields
            specific_field => field_name == specific_field,
        }
    }

    /// Check if a method requires pagination parameters
    pub fn needs_pagination(request_type: &RequestType) -> bool {
        matches!(request_type, RequestType::List)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod string_tests {
        use super::*;

        #[test]
        fn test_service_to_handler_name() {
            assert_eq!(
                strings::service_to_handler_name("CatalogsService"),
                "CatalogHandler"
            );
            assert_eq!(
                strings::service_to_handler_name("RecipientsService"),
                "RecipientHandler"
            );
            assert_eq!(
                strings::service_to_handler_name("SchemasService"),
                "SchemaHandler"
            );
        }

        #[test]
        fn test_operation_to_method_name() {
            assert_eq!(
                strings::operation_to_method_name("ListCatalogs"),
                "list_catalogs"
            );
            assert_eq!(
                strings::operation_to_method_name("CreateCatalog"),
                "create_catalog"
            );
            assert_eq!(
                strings::operation_to_method_name("GetCatalog"),
                "get_catalog"
            );
        }

        #[test]
        fn test_service_to_base_path() {
            assert_eq!(strings::service_to_base_path("CatalogsService"), "catalogs");
            assert_eq!(
                strings::service_to_base_path("RecipientsService"),
                "recipients"
            );
        }

        #[test]
        fn test_extract_simple_type_name() {
            assert_eq!(
                strings::extract_simple_type_name(".unitycatalog.catalogs.v1.CatalogInfo"),
                "CatalogInfo"
            );
            assert_eq!(
                strings::extract_simple_type_name("SimpleType"),
                "SimpleType"
            );
        }
    }

    mod path_tests {
        use super::*;

        #[test]
        fn test_extract_path_parameters() {
            assert_eq!(
                paths::extract_path_parameters("/catalogs/{name}"),
                vec!["name"]
            );
            assert_eq!(
                paths::extract_path_parameters("/shares/{share}/schemas/{schema}/tables/{name}"),
                vec!["share", "schema", "name"]
            );
            assert_eq!(
                paths::extract_path_parameters("/catalogs"),
                Vec::<String>::new()
            );
        }

        #[test]
        fn test_format_url_template() {
            let (format_str, args) =
                paths::format_url_template("/catalogs/{name}", &["name".to_string()]);
            assert_eq!(format_str, "/catalogs/{}");
            assert_eq!(args, vec!["name"]);

            let (format_str, args) = paths::format_url_template("/catalogs", &[]);
            assert_eq!(format_str, "/catalogs");
            assert_eq!(args, Vec::<String>::new());
        }
    }

    mod type_tests {
        use super::*;
        use crate::RequestType;

        #[test]
        fn test_field_type_to_rust_type() {
            assert_eq!(types::field_type_to_rust_type("TYPE_STRING"), "String");
            assert_eq!(types::field_type_to_rust_type("TYPE_INT32"), "i32");
            assert_eq!(types::field_type_to_rust_type("TYPE_BOOL"), "bool");
            assert_eq!(
                types::field_type_to_rust_type(
                    "TYPE_MESSAGE:.unitycatalog.catalogs.v1.CatalogInfo"
                ),
                "CatalogInfo"
            );
        }

        #[test]
        fn test_has_response_body() {
            assert!(types::has_response_body(&RequestType::Get));
            assert!(types::has_response_body(&RequestType::Create));
            assert!(types::has_response_body(&RequestType::Update));
            assert!(types::has_response_body(&RequestType::List));
            assert!(!types::has_response_body(&RequestType::Delete));
        }
    }

    mod request_tests {
        use super::*;
        use crate::RequestType;

        #[test]
        fn test_classify_request_type() {
            assert_eq!(
                requests::classify_request_type(Some("ListCatalogs"), "ListCatalogs"),
                RequestType::List
            );
            assert_eq!(
                requests::classify_request_type(Some("CreateCatalog"), "CreateCatalog"),
                RequestType::Create
            );
            assert_eq!(
                requests::classify_request_type(Some("GetCatalog"), "GetCatalog"),
                RequestType::Get
            );
            assert_eq!(
                requests::classify_request_type(Some("UpdateCatalog"), "UpdateCatalog"),
                RequestType::Update
            );
            assert_eq!(
                requests::classify_request_type(Some("DeleteCatalog"), "DeleteCatalog"),
                RequestType::Delete
            );
        }

        #[test]
        fn test_should_be_body_field() {
            assert!(requests::should_be_body_field("any_field", "*"));
            assert!(!requests::should_be_body_field("any_field", ""));
            assert!(requests::should_be_body_field("specific", "specific"));
            assert!(!requests::should_be_body_field("other", "specific"));
        }
    }
}
