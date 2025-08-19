//! Test fixtures using rstest for reusable test setup
//!
//! This module provides fixtures that can be used across multiple test files
//! to ensure consistent test setup and reduce code duplication.

use rstest::*;
use std::collections::HashMap;
use unitycatalog_client::UnityCatalogClient;

use super::TestServer;

/// Fixture that provides a Unity Catalog client configured with a mock server
#[fixture]
pub async fn test_client() -> (UnityCatalogClient, TestServer) {
    let server = TestServer::new().await;
    let client = server.create_client();
    (client, server)
}

/// Fixture for a basic catalog name
#[fixture]
pub fn catalog_name() -> String {
    "test_catalog".to_string()
}

/// Fixture for catalog properties
#[fixture]
pub fn catalog_properties() -> HashMap<String, String> {
    let mut properties = HashMap::new();
    properties.insert("environment".to_string(), "test".to_string());
    properties.insert("team".to_string(), "data-engineering".to_string());
    properties.insert("cost-center".to_string(), "12345".to_string());
    properties
}

/// Fixture for sharing catalog parameters
#[fixture]
pub fn sharing_params() -> (String, String) {
    ("test_provider".to_string(), "test_share".to_string())
}

/// Fixture for update scenarios with different field combinations
#[fixture]
pub fn update_params() -> (Option<String>, Option<String>, Option<String>) {
    (
        Some("new_catalog_name".to_string()),
        Some("Updated comment".to_string()),
        Some("new_owner".to_string()),
    )
}

/// Fixture for force delete parameter variations
#[fixture]
pub fn force_delete_variations() -> Option<bool> {
    Some(true)
}

/// Fixture for testing invalid catalog names
#[fixture]
pub fn invalid_catalog_names() -> String {
    "catalog@#$%".to_string()
}

/// Fixture for testing concurrent operations
#[fixture]
pub fn concurrent_operation_count() -> usize {
    10
}

/// Fixture for schema creation parameters
#[fixture]
pub fn schema_params() -> (String, Option<String>) {
    (
        "test_schema".to_string(),
        Some("Test schema for catalog tests".to_string()),
    )
}

/// Helper fixture for setting up complex test scenarios with multiple catalogs
#[fixture]
pub fn multi_catalog_scenario() -> Vec<(String, Option<String>, HashMap<String, String>)> {
    vec![
        (
            "production_catalog".to_string(),
            Some("Production data catalog".to_string()),
            {
                let mut props = HashMap::new();
                props.insert("environment".to_string(), "production".to_string());
                props.insert("tier".to_string(), "critical".to_string());
                props
            },
        ),
        (
            "staging_catalog".to_string(),
            Some("Staging data catalog".to_string()),
            {
                let mut props = HashMap::new();
                props.insert("environment".to_string(), "staging".to_string());
                props.insert("tier".to_string(), "standard".to_string());
                props
            },
        ),
        ("dev_catalog".to_string(), None, {
            let mut props = HashMap::new();
            props.insert("environment".to_string(), "development".to_string());
            props
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest]
    fn test_catalog_name_fixture(catalog_name: String) {
        assert_eq!(catalog_name, "test_catalog");
    }

    #[rstest]
    fn test_catalog_properties_fixture(catalog_properties: HashMap<String, String>) {
        assert!(catalog_properties.contains_key("environment"));
        assert!(catalog_properties.contains_key("team"));
        assert!(catalog_properties.contains_key("cost-center"));
    }
}
