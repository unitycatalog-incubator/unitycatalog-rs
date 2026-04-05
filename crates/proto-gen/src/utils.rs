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
    }
}
