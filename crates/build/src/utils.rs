//! Shared utilities for the build crate
//!
//! This module contains common functions used across different parts of the code generation
//! pipeline to reduce duplication and improve maintainability.
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
}

/// Type mapping utilities
pub mod types {

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
                if field_type.ends_with("PropertiesEntry") {
                    "HashMap<String, String>".to_string()
                } else if field_type.starts_with("TYPE_MESSAGE:") {
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

    /// Make a type optional by wrapping in Option<T>
    pub fn make_optional(rust_type: &str) -> String {
        format!("Option<{}>", rust_type)
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

    mod type_tests {
        use super::*;

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
    }
}
