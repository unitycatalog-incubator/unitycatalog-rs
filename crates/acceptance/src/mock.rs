//! Mock server and test utilities for Unity Catalog acceptance testing
//!
//! This module provides mock server functionality, test data loading, and
//! common assertion helpers for acceptance tests.

use crate::{AcceptanceError, AcceptanceResult};
use cloud_client::CloudClient;
use mockito::{Mock, Server, ServerGuard};
use serde_json::Value;
use unitycatalog_client::UnityCatalogClient;
use url::Url;

/// Test server wrapper that provides utilities for mocking Unity Catalog API responses
pub struct TestServer {
    _server: ServerGuard,
    base_url: String,
}

impl TestServer {
    /// Create a new test server
    pub async fn new() -> Self {
        let server = Server::new_async().await;
        let base_url = server.url();

        Self {
            _server: server,
            base_url,
        }
    }

    /// Get the base URL of the mock server
    pub fn url(&self) -> &str {
        &self.base_url
    }

    /// Create a mock for any API endpoint
    pub fn mock_endpoint(&mut self, method: &str, path: &str) -> Mock {
        self._server.mock(method, path)
    }

    /// Create a mock for a catalog API endpoint
    pub fn mock_catalog_endpoint(&mut self, method: &str, path: &str) -> Mock {
        self._server.mock(method, path)
    }

    /// Create a mock for a schema API endpoint
    pub fn mock_schema_endpoint(&mut self, method: &str, path: &str) -> Mock {
        self._server.mock(method, path)
    }

    /// Create a mock for a table API endpoint
    pub fn mock_table_endpoint(&mut self, method: &str, path: &str) -> Mock {
        self._server.mock(method, path)
    }

    /// Create a Unity Catalog client configured to use this test server
    pub fn create_client(&self) -> UnityCatalogClient {
        let cloud_client = CloudClient::new_unauthenticated();
        let base_url = Url::parse(&self.base_url).unwrap();
        UnityCatalogClient::new(cloud_client, base_url)
    }

    /// Set up default mock responses for common endpoints
    pub async fn setup_default_mocks(&mut self) {
        // Mock health check endpoint
        self.mock_endpoint("GET", "/health")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"status": "healthy"}"#)
            .create_async()
            .await;

        // Mock catalogs list endpoint
        self.mock_catalog_endpoint("GET", "/api/2.1/unity-catalog/catalogs")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"catalogs": []}"#)
            .create_async()
            .await;
    }

    /// Set up mock responses for a specific journey step
    pub async fn setup_journey_mocks(&mut self, journey_name: &str) -> AcceptanceResult<()> {
        // Load journey-specific mock responses if they exist
        if let Ok(mock_data) =
            TestDataLoader::load_response("mocks", &format!("{}.json", journey_name))
        {
            if let Some(mocks) = mock_data.as_object() {
                for (endpoint, response) in mocks {
                    // Parse endpoint (format: "METHOD /path")
                    let parts: Vec<&str> = endpoint.splitn(2, ' ').collect();
                    if parts.len() == 2 {
                        let method = parts[0];
                        let path = parts[1];

                        self.mock_endpoint(method, path)
                            .with_status(200)
                            .with_header("content-type", "application/json")
                            .with_body(&response.to_string())
                            .create_async()
                            .await;
                    }
                }
            }
        }

        Ok(())
    }
}

/// Load test data from JSON files
pub struct TestDataLoader;

impl TestDataLoader {
    /// Load a JSON response from the test_data directory
    pub fn load_response(category: &str, filename: &str) -> AcceptanceResult<Value> {
        let path = format!("test_data/{}/{}", category, filename);
        let content = std::fs::read_to_string(&path).map_err(|e| AcceptanceError::Io(e))?;

        let value: Value = serde_json::from_str(&content)?;
        Ok(value)
    }

    /// Load test data with variable substitution
    pub fn load_response_with_variables(
        category: &str,
        filename: &str,
        variables: &std::collections::HashMap<String, Value>,
    ) -> AcceptanceResult<Value> {
        let mut content = std::fs::read_to_string(&format!("test_data/{}/{}", category, filename))
            .map_err(|e| AcceptanceError::Io(e))?;

        // Simple variable substitution
        for (key, value) in variables {
            let placeholder = format!("{{{}}}", key);
            let replacement = match value {
                Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            content = content.replace(&placeholder, &replacement);
        }

        let value: Value = serde_json::from_str(&content)?;
        Ok(value)
    }

    /// Load all test data files from a category directory
    pub fn load_all_responses(
        category: &str,
    ) -> AcceptanceResult<std::collections::HashMap<String, Value>> {
        let mut responses = std::collections::HashMap::new();
        let dir_path = format!("test_data/{}", category);

        for entry in std::fs::read_dir(&dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                    let content = std::fs::read_to_string(&path)?;
                    let value: Value = serde_json::from_str(&content)?;
                    responses.insert(filename.to_string(), value);
                }
            }
        }

        Ok(responses)
    }

    /// Check if test data file exists
    pub fn data_exists(category: &str, filename: &str) -> bool {
        let path = format!("test_data/{}/{}", category, filename);
        std::path::Path::new(&path).exists()
    }
}

/// Common test fixtures and data generators
pub struct TestFixtures;

impl TestFixtures {
    /// Generate a test catalog info object
    pub fn catalog_info(name: &str) -> Value {
        serde_json::json!({
            "name": name,
            "comment": format!("Test catalog: {}", name),
            "storage_root": format!("s3://test-bucket/{}", name),
            "created_at": chrono::Utc::now().timestamp_millis(),
            "updated_at": chrono::Utc::now().timestamp_millis(),
            "properties": {
                "environment": "test",
                "owner": "test-user"
            }
        })
    }

    /// Generate a test schema info object
    pub fn schema_info(catalog_name: &str, schema_name: &str) -> Value {
        serde_json::json!({
            "name": schema_name,
            "catalog_name": catalog_name,
            "comment": format!("Test schema: {}.{}", catalog_name, schema_name),
            "full_name": format!("{}.{}", catalog_name, schema_name),
            "created_at": chrono::Utc::now().timestamp_millis(),
            "updated_at": chrono::Utc::now().timestamp_millis(),
            "properties": {
                "environment": "test"
            }
        })
    }

    /// Generate a test table info object
    pub fn table_info(catalog_name: &str, schema_name: &str, table_name: &str) -> Value {
        serde_json::json!({
            "name": table_name,
            "catalog_name": catalog_name,
            "schema_name": schema_name,
            "full_name": format!("{}.{}.{}", catalog_name, schema_name, table_name),
            "table_type": "MANAGED",
            "data_source_format": "DELTA",
            "comment": format!("Test table: {}.{}.{}", catalog_name, schema_name, table_name),
            "created_at": chrono::Utc::now().timestamp_millis(),
            "updated_at": chrono::Utc::now().timestamp_millis(),
            "properties": {
                "environment": "test"
            },
            "columns": [
                {
                    "name": "id",
                    "type_name": "BIGINT",
                    "type_text": "bigint",
                    "nullable": false,
                    "comment": "Primary key"
                },
                {
                    "name": "name",
                    "type_name": "STRING",
                    "type_text": "string",
                    "nullable": true,
                    "comment": "Name field"
                }
            ]
        })
    }

    /// Generate random test data with UUID
    pub fn random_name(prefix: &str) -> String {
        format!(
            "{}_{}",
            prefix,
            uuid::Uuid::new_v4().to_string().replace('-', "")[..8].to_lowercase()
        )
    }

    /// Generate timestamp for test data
    pub fn test_timestamp() -> i64 {
        chrono::Utc::now().timestamp_millis()
    }

    /// Generate test error response
    pub fn error_response(status: u16, message: &str) -> Value {
        serde_json::json!({
            "error_code": format!("TEST_ERROR_{}", status),
            "message": message,
            "details": {
                "timestamp": Self::test_timestamp(),
                "request_id": uuid::Uuid::new_v4().to_string()
            }
        })
    }
}

/// Response builders for common API patterns
pub struct ResponseBuilder;

impl ResponseBuilder {
    /// Build a paginated list response
    pub fn paginated_list<T>(items: Vec<T>, next_page_token: Option<String>) -> Value
    where
        T: serde::Serialize,
    {
        let mut response = serde_json::json!({
            "items": items
        });

        if let Some(token) = next_page_token {
            response["next_page_token"] = Value::String(token);
        }

        response
    }

    /// Build a catalog list response
    pub fn catalog_list(catalogs: Vec<Value>) -> Value {
        serde_json::json!({
            "catalogs": catalogs
        })
    }

    /// Build a schema list response
    pub fn schema_list(schemas: Vec<Value>) -> Value {
        serde_json::json!({
            "schemas": schemas
        })
    }

    /// Build a table list response
    pub fn table_list(tables: Vec<Value>) -> Value {
        serde_json::json!({
            "tables": tables
        })
    }

    /// Build a success response
    pub fn success() -> Value {
        serde_json::json!({
            "status": "success",
            "timestamp": TestFixtures::test_timestamp()
        })
    }

    /// Build a not found error response
    pub fn not_found(resource_type: &str, resource_name: &str) -> Value {
        TestFixtures::error_response(
            404,
            &format!("{} '{}' not found", resource_type, resource_name),
        )
    }

    /// Build a conflict error response
    pub fn conflict(resource_type: &str, resource_name: &str) -> Value {
        TestFixtures::error_response(
            409,
            &format!("{} '{}' already exists", resource_type, resource_name),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let server = TestServer::new().await;
        assert!(!server.url().is_empty());
        assert!(server.url().starts_with("http://"));
    }

    #[tokio::test]
    async fn test_client_creation() {
        let server = TestServer::new().await;
        let client = server.create_client();
        // Basic test that client was created successfully
        // Note: UnityCatalogClient doesn't expose base_url publicly
        assert!(!server.url().is_empty());
    }

    #[test]
    fn test_fixtures_catalog_info() {
        let catalog = TestFixtures::catalog_info("test_catalog");
        assert_eq!(catalog["name"], "test_catalog");
        assert!(
            catalog["comment"]
                .as_str()
                .unwrap()
                .contains("test_catalog")
        );
        assert!(catalog["created_at"].is_number());
    }

    #[test]
    fn test_fixtures_random_name() {
        let name1 = TestFixtures::random_name("test");
        let name2 = TestFixtures::random_name("test");
        assert_ne!(name1, name2);
        assert!(name1.starts_with("test_"));
        assert!(name2.starts_with("test_"));
    }

    #[test]
    fn test_response_builder_catalog_list() {
        let catalogs = vec![
            TestFixtures::catalog_info("cat1"),
            TestFixtures::catalog_info("cat2"),
        ];
        let response = ResponseBuilder::catalog_list(catalogs);
        assert!(response["catalogs"].is_array());
        assert_eq!(response["catalogs"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_response_builder_paginated() {
        let items = vec![serde_json::json!({"id": 1}), serde_json::json!({"id": 2})];
        let response = ResponseBuilder::paginated_list(items, Some("next_token".to_string()));
        assert!(response["items"].is_array());
        assert_eq!(response["next_page_token"], "next_token");
    }

    #[test]
    fn test_error_responses() {
        let not_found = ResponseBuilder::not_found("Catalog", "missing_catalog");
        assert_eq!(not_found["error_code"], "TEST_ERROR_404");
        assert!(
            not_found["message"]
                .as_str()
                .unwrap()
                .contains("missing_catalog")
        );

        let conflict = ResponseBuilder::conflict("Schema", "existing_schema");
        assert_eq!(conflict["error_code"], "TEST_ERROR_409");
        assert!(
            conflict["message"]
                .as_str()
                .unwrap()
                .contains("existing_schema")
        );
    }
}
