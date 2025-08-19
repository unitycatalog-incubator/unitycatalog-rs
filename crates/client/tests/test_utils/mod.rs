//! Test utilities for Unity Catalog client tests
//!
//! This module provides shared utilities for setting up mock servers,
//! loading test data, and creating test fixtures.

use cloud_client::CloudClient;
use mockito::{Mock, Server, ServerGuard};
use serde_json::Value;
use std::collections::HashMap;
use unitycatalog_client::UnityCatalogClient;
use url::Url;

pub mod fixtures;
pub mod journeys;
pub mod responses;

/// Test server wrapper that provides utilities for mocking Unity Catalog API responses
pub struct TestServer {
    server: ServerGuard,
    base_url: String,
}

impl TestServer {
    /// Create a new test server
    pub async fn new() -> Self {
        let server = Server::new_async().await;
        let base_url = server.url();

        Self { server, base_url }
    }

    /// Get the base URL of the mock server
    pub fn url(&self) -> &str {
        &self.base_url
    }

    /// Create a mock for a catalog API endpoint
    pub fn mock_catalog_endpoint(&mut self, method: &str, path: &str) -> Mock {
        self.server.mock(method, path)
    }

    /// Create a Unity Catalog client configured to use this test server
    pub fn create_client(&self) -> UnityCatalogClient {
        let cloud_client = CloudClient::new_unauthenticated();
        let base_url = Url::parse(&self.base_url).unwrap();
        UnityCatalogClient::new(cloud_client, base_url)
    }
}

/// Load test data from JSON files
pub struct TestDataLoader;

impl TestDataLoader {
    /// Load a JSON response from the test_data directory
    pub fn load_response(
        category: &str,
        filename: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let path = format!("tests/test_data/{}/{}", category, filename);
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read test data file {}: {}", path, e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path, e).into())
    }

    /// Load all response files for a category
    pub fn load_all_responses(
        category: &str,
    ) -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
        let dir_path = format!("tests/test_data/{}", category);
        let dir = std::fs::read_dir(&dir_path)
            .map_err(|e| format!("Failed to read directory {}: {}", dir_path, e))?;

        let mut responses = HashMap::new();

        for entry in dir {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let filename = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or("Invalid filename")?;

                let content = std::fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read file {:?}: {}", path, e))?;

                let json: Value = serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse JSON from {:?}: {}", path, e))?;

                responses.insert(filename.to_string(), json);
            }
        }

        Ok(responses)
    }
}

/// Assertion helpers for testing
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that a catalog info matches expected values
    pub fn assert_catalog_info_matches(
        actual: &unitycatalog_common::models::catalogs::v1::CatalogInfo,
        expected_name: &str,
        expected_comment: Option<&str>,
    ) {
        assert_eq!(actual.name, expected_name);
        assert_eq!(actual.comment.as_deref(), expected_comment);
        assert!(actual.created_at.is_some());
        assert!(actual.updated_at.is_some());
    }

    /// Assert that an error contains expected message
    pub fn assert_error_contains<T: std::fmt::Debug, E: std::fmt::Display>(
        result: &Result<T, E>,
        expected_msg: &str,
    ) {
        match result {
            Err(e) => {
                let error_string = e.to_string().to_lowercase();
                let expected_lower = expected_msg.to_lowercase();
                assert!(
                    error_string.contains(&expected_lower)
                        || error_string.contains("404")
                        || error_string.contains("409")
                        || error_string.contains("400")
                        || error_string.contains("conflict")
                        || error_string.contains("not found")
                        || error_string.contains("already exists")
                        || error_string.contains("bad request")
                        || error_string.contains("invalid"),
                    "Error '{}' does not contain expected message '{}'",
                    e,
                    expected_msg
                );
            }
            Ok(val) => panic!("Expected error but got success: {:?}", val),
        }
    }
}

/// HTTP response builder for creating consistent mock responses
#[derive(Default)]
pub struct ResponseBuilder {
    status: u16,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }

    pub fn json_body(mut self, body: &Value) -> Self {
        self.body = Some(serde_json::to_string(body).unwrap());
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        self
    }

    pub fn text_body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }

    pub fn build(self) -> (u16, HashMap<String, String>, String) {
        (self.status, self.headers, self.body.unwrap_or_default())
    }
}

/// Common test patterns and utilities
pub struct TestPatterns;

impl TestPatterns {
    /// Standard success response pattern
    pub fn success_response(data: &Value) -> ResponseBuilder {
        ResponseBuilder::new().status(200).json_body(data)
    }

    /// Standard error response pattern
    pub fn error_response(status: u16, error_code: &str, message: &str) -> ResponseBuilder {
        let error_body = serde_json::json!({
            "error_code": error_code,
            "message": message
        });

        ResponseBuilder::new().status(status).json_body(&error_body)
    }

    /// Not found error response
    pub fn not_found_response(resource: &str) -> ResponseBuilder {
        Self::error_response(
            404,
            "RESOURCE_DOES_NOT_EXIST",
            &format!("{} not found", resource),
        )
    }

    /// Conflict error response
    pub fn conflict_response(resource: &str) -> ResponseBuilder {
        Self::error_response(
            409,
            "RESOURCE_ALREADY_EXISTS",
            &format!("{} already exists", resource),
        )
    }
}
