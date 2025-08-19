//! Response fixtures for catalog tests
//!
//! This module provides pre-defined response data that matches what the Unity Catalog
//! server would return, making tests more maintainable and realistic.

use serde_json::{Value, json};
use std::collections::HashMap;

/// Catalog response fixtures
pub struct CatalogResponses;

impl CatalogResponses {
    /// Standard catalog info response
    pub fn catalog_info(name: &str, comment: Option<&str>) -> Value {
        json!({
            "name": name,
            "comment": comment,
            "storage_root": format!("s3://my-bucket/catalogs/{}", name),
            "properties": {},
            "owner": "test-user",
            "created_at": 1699564800000i64, // 2023-11-09T20:00:00Z
            "updated_at": 1699564800000i64,
            "catalog_type": "MANAGED_CATALOG",
            "provider_name": null,
            "share_name": null,
            "isolation_mode": "OPEN",
            "options": {},
            "effective_predictive_optimization_flag": {
                "value": "INHERIT",
                "inherited_from_type": "SYSTEM_DEFAULT"
            }
        })
    }

    /// List catalogs response
    pub fn list_catalogs(
        catalogs: Vec<(&str, Option<&str>)>,
        next_page_token: Option<&str>,
    ) -> Value {
        let catalog_list: Vec<Value> = catalogs
            .into_iter()
            .map(|(name, comment)| Self::catalog_info(name, comment))
            .collect();

        json!({
            "catalogs": catalog_list,
            "next_page_token": next_page_token
        })
    }

    /// Create catalog response (same as catalog_info)
    pub fn create_catalog(name: &str, comment: Option<&str>) -> Value {
        Self::catalog_info(name, comment)
    }

    /// Update catalog response
    pub fn update_catalog(_old_name: &str, new_name: &str, comment: Option<&str>) -> Value {
        let mut catalog = Self::catalog_info(new_name, comment);
        // Update the updated_at timestamp to be different
        catalog["updated_at"] = json!(1699568400000i64); // 1 hour later
        catalog
    }

    /// Catalog with properties
    pub fn catalog_with_properties(name: &str, properties: HashMap<String, String>) -> Value {
        let mut catalog = Self::catalog_info(name, None);
        catalog["properties"] = json!(properties);
        catalog
    }

    /// Sharing catalog response
    pub fn sharing_catalog(name: &str, provider_name: &str, share_name: &str) -> Value {
        json!({
            "name": name,
            "comment": null,
            "storage_root": null,
            "properties": {},
            "owner": "test-user",
            "created_at": 1699564800000i64,
            "updated_at": 1699564800000i64,
            "catalog_type": "DELTASHARING_CATALOG",
            "provider_name": provider_name,
            "share_name": share_name,
            "isolation_mode": "ISOLATED",
            "options": {},
            "effective_predictive_optimization_flag": {
                "value": "INHERIT",
                "inherited_from_type": "SYSTEM_DEFAULT"
            }
        })
    }

    /// Empty list response
    pub fn empty_list() -> Value {
        json!({
            "catalogs": [],
            "next_page_token": null
        })
    }

    /// Paginated response - first page
    pub fn first_page() -> Value {
        Self::list_catalogs(
            vec![
                ("catalog1", Some("First catalog")),
                ("catalog2", Some("Second catalog")),
            ],
            Some("next_page_token_123"),
        )
    }

    /// Paginated response - second page
    pub fn second_page() -> Value {
        Self::list_catalogs(
            vec![("catalog3", Some("Third catalog")), ("catalog4", None)],
            None, // No more pages
        )
    }
}

/// Error response fixtures
pub struct ErrorResponses;

impl ErrorResponses {
    /// Resource not found error
    pub fn not_found(resource_type: &str, resource_name: &str) -> Value {
        json!({
            "error_code": "RESOURCE_DOES_NOT_EXIST",
            "message": format!("{} '{}' not found", resource_type, resource_name),
            "details": [
                {
                    "reason": "RESOURCE_DOES_NOT_EXIST",
                    "domain": "UNITY_CATALOG",
                    "metadata": {
                        "resource_type": resource_type,
                        "resource_name": resource_name
                    }
                }
            ]
        })
    }

    /// Resource already exists error
    pub fn already_exists(resource_type: &str, resource_name: &str) -> Value {
        json!({
            "error_code": "RESOURCE_ALREADY_EXISTS",
            "message": format!("{} '{}' already exists", resource_type, resource_name),
            "details": [
                {
                    "reason": "RESOURCE_ALREADY_EXISTS",
                    "domain": "UNITY_CATALOG",
                    "metadata": {
                        "resource_type": resource_type,
                        "resource_name": resource_name
                    }
                }
            ]
        })
    }

    /// Invalid request error
    pub fn invalid_request(message: &str) -> Value {
        json!({
            "error_code": "INVALID_PARAMETER_VALUE",
            "message": message,
            "details": [
                {
                    "reason": "INVALID_PARAMETER_VALUE",
                    "domain": "UNITY_CATALOG"
                }
            ]
        })
    }

    /// Permission denied error
    pub fn permission_denied(resource_type: &str, resource_name: &str) -> Value {
        json!({
            "error_code": "PERMISSION_DENIED",
            "message": format!("Permission denied on {} '{}'", resource_type, resource_name),
            "details": [
                {
                    "reason": "PERMISSION_DENIED",
                    "domain": "UNITY_CATALOG",
                    "metadata": {
                        "resource_type": resource_type,
                        "resource_name": resource_name
                    }
                }
            ]
        })
    }

    /// Internal server error
    pub fn internal_error() -> Value {
        json!({
            "error_code": "INTERNAL_ERROR",
            "message": "An internal error occurred",
            "details": [
                {
                    "reason": "INTERNAL_ERROR",
                    "domain": "UNITY_CATALOG"
                }
            ]
        })
    }

    /// Rate limit exceeded error
    pub fn rate_limit_exceeded() -> Value {
        json!({
            "error_code": "RESOURCE_EXHAUSTED",
            "message": "Rate limit exceeded",
            "details": [
                {
                    "reason": "RESOURCE_EXHAUSTED",
                    "domain": "UNITY_CATALOG"
                }
            ]
        })
    }
}

/// Schema response fixtures for catalog tests that involve schemas
pub struct SchemaResponses;

impl SchemaResponses {
    /// Schema info response
    pub fn schema_info(catalog_name: &str, schema_name: &str, comment: Option<&str>) -> Value {
        json!({
            "name": schema_name,
            "catalog_name": catalog_name,
            "comment": comment,
            "properties": {},
            "owner": "test-user",
            "created_at": 1699564800000i64,
            "updated_at": 1699564800000i64,
            "schema_type": "MANAGED_SCHEMA",
            "storage_root": format!("s3://my-bucket/catalogs/{}/schemas/{}", catalog_name, schema_name),
            "full_name": format!("{}.{}", catalog_name, schema_name)
        })
    }

    /// List schemas response
    pub fn list_schemas(schemas: Vec<(&str, &str, Option<&str>)>) -> Value {
        let schema_list: Vec<Value> = schemas
            .into_iter()
            .map(|(catalog_name, schema_name, comment)| {
                Self::schema_info(catalog_name, schema_name, comment)
            })
            .collect();

        json!({
            "schemas": schema_list,
            "next_page_token": null
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_info_response() {
        let response = CatalogResponses::catalog_info("test_catalog", Some("Test comment"));

        assert_eq!(response["name"], "test_catalog");
        assert_eq!(response["comment"], "Test comment");
        assert_eq!(response["catalog_type"], "MANAGED_CATALOG");
        assert!(response["created_at"].is_number());
    }

    #[test]
    fn test_list_catalogs_response() {
        let catalogs = vec![("cat1", Some("Comment 1")), ("cat2", None)];
        let response = CatalogResponses::list_catalogs(catalogs, Some("token"));

        assert_eq!(response["catalogs"].as_array().unwrap().len(), 2);
        assert_eq!(response["next_page_token"], "token");
    }

    #[test]
    fn test_error_responses() {
        let error = ErrorResponses::not_found("Catalog", "missing_catalog");

        assert_eq!(error["error_code"], "RESOURCE_DOES_NOT_EXIST");
        assert!(
            error["message"]
                .as_str()
                .unwrap()
                .contains("missing_catalog")
        );
    }

    #[test]
    fn test_sharing_catalog_response() {
        let response = CatalogResponses::sharing_catalog("shared_cat", "provider1", "share1");

        assert_eq!(response["name"], "shared_cat");
        assert_eq!(response["catalog_type"], "DELTASHARING_CATALOG");
        assert_eq!(response["provider_name"], "provider1");
        assert_eq!(response["share_name"], "share1");
    }
}
