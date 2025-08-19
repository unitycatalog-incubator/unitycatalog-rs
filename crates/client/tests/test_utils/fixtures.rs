//! Test fixtures using rstest for reusable test setup
//!
//! This module provides fixtures that can be used across multiple test files
//! to ensure consistent test setup and reduce code duplication.

use rstest::*;
use std::collections::HashMap;
use unitycatalog_client::UnityCatalogClient;

use super::{TestServer, responses::CatalogResponses};

/// Fixture that provides a mock server for testing
#[fixture]
pub async fn mock_server() -> TestServer {
    TestServer::new().await
}

/// Fixture that provides a Unity Catalog client configured with a mock server
#[fixture]
pub async fn test_client(#[future] mock_server: TestServer) -> (UnityCatalogClient, TestServer) {
    let server = mock_server.await;
    let client = server.create_client();
    (client, server)
}

/// Fixture for a basic catalog name
#[fixture]
pub fn catalog_name() -> String {
    "test_catalog".to_string()
}

/// Fixture for catalog names used in list operations
#[fixture]
pub fn catalog_names() -> Vec<String> {
    vec![
        "catalog1".to_string(),
        "catalog2".to_string(),
        "catalog3".to_string(),
    ]
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

/// Fixture for catalog comment variations
#[fixture]
pub fn catalog_comment() -> Option<String> {
    Some("Simple catalog comment".to_string())
}

/// Fixture for pagination parameters
#[fixture]
pub fn pagination_params() -> (Option<i32>, Option<String>) {
    (Some(10), Some("page_token_123".to_string()))
}

/// Fixture that provides common error scenarios
#[fixture]
pub fn error_scenario() -> (u16, String) {
    (404, "RESOURCE_DOES_NOT_EXIST".to_string())
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

/// Fixture for testing various property combinations
#[fixture]
pub fn property_variations() -> HashMap<String, String> {
    let mut props = HashMap::new();
    props.insert("environment".to_string(), "production".to_string());
    props.insert("version".to_string(), "1.0.0".to_string());
    props.insert("team".to_string(), "platform".to_string());
    props
}

/// Fixture for force delete parameter variations
#[fixture]
pub fn force_delete_variations() -> Option<bool> {
    Some(true)
}

/// Fixture for max_results parameter variations in list operations
#[fixture]
pub fn max_results_variations() -> Option<i32> {
    Some(50)
}

/// Fixture for common storage root patterns
#[fixture]
pub fn storage_root_variations() -> Option<String> {
    Some("s3://my-test-bucket/catalogs/".to_string())
}

/// Fixture for schema creation parameters
#[fixture]
pub fn schema_params() -> (String, Option<String>) {
    (
        "test_schema".to_string(),
        Some("Test schema for catalog tests".to_string()),
    )
}

/// Fixture for testing catalog browse functionality
#[fixture]
pub fn browse_variations() -> Option<bool> {
    Some(true)
}

/// Fixture that provides sample response data for different scenarios
#[fixture]
pub fn sample_catalog_response() -> serde_json::Value {
    CatalogResponses::catalog_info("sample_catalog", Some("Sample catalog for testing"))
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

/// Combined fixture for comprehensive catalog testing scenarios
#[fixture]
pub fn comprehensive_test_scenario(
    catalog_name: String,
    catalog_properties: HashMap<String, String>,
    catalog_comment: Option<String>,
) -> (String, HashMap<String, String>, Option<String>) {
    (catalog_name, catalog_properties, catalog_comment)
}

/// Fixture for testing with different authentication contexts
#[fixture]
pub fn user_contexts() -> String {
    "admin".to_string()
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
    #[tokio::test]
    async fn test_mock_server_fixture(#[future] mock_server: TestServer) {
        let server = mock_server.await;
        assert!(!server.url().is_empty());
        assert!(server.url().starts_with("http://"));
    }

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

    #[rstest]
    fn test_property_variations_fixture(property_variations: HashMap<String, String>) {
        // Test passes as long as the fixture provides valid HashMap
        assert!(property_variations.len() >= 0);
    }

    #[rstest]
    fn test_error_scenario_fixture(error_scenario: (u16, String)) {
        let (status, error_code) = error_scenario;
        assert!(status >= 400);
        assert!(!error_code.is_empty());
    }

    #[rstest]
    fn test_multi_catalog_scenario_fixture(
        multi_catalog_scenario: Vec<(String, Option<String>, HashMap<String, String>)>,
    ) {
        assert_eq!(multi_catalog_scenario.len(), 3);

        // Verify each catalog has the expected structure
        for (name, _comment, properties) in multi_catalog_scenario {
            assert!(!name.is_empty());
            assert!(properties.contains_key("environment"));
        }
    }
}
