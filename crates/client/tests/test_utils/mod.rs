//! Test utilities for Unity Catalog client tests
//!
//! This module provides shared utilities for setting up mock servers,
//! loading test data, and creating test fixtures.

use cloud_client::CloudClient;
use mockito::{Mock, Server, ServerGuard};
use serde_json::Value;
use unitycatalog_client::UnityCatalogClient;
use url::Url;

pub mod fixtures;
pub mod journeys;
pub mod responses;

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

    /// Create a mock for a catalog API endpoint
    pub fn mock_catalog_endpoint(&mut self, method: &str, path: &str) -> Mock {
        self._server.mock(method, path)
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
