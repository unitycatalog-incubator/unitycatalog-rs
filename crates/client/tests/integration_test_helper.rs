//! Integration test helper for Unity Catalog client tests
//!
//! This module provides utilities for running integration tests against
//! both mock servers and real Unity Catalog deployments. It includes
//! configuration management and test data collection functionality.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use tokio::time::{Duration, timeout};

/// Configuration for integration tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Base URL for Unity Catalog server (for integration tests)
    pub server_url: Option<String>,
    /// Authentication token (for integration tests)
    pub auth_token: Option<String>,
    /// Timeout for operations in seconds
    pub operation_timeout_secs: u64,
    /// Whether to run integration tests against real server
    pub run_integration_tests: bool,
    /// Directory to save captured responses
    pub response_capture_dir: PathBuf,
    /// Whether to capture responses for future mock tests
    pub capture_responses: bool,
    /// Test data directory
    pub test_data_dir: PathBuf,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            server_url: env::var("UC_SERVER_URL").ok(),
            auth_token: env::var("UC_AUTH_TOKEN").ok(),
            operation_timeout_secs: 30,
            run_integration_tests: env::var("RUN_INTEGRATION_TESTS")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            response_capture_dir: PathBuf::from("tests/captured_responses"),
            capture_responses: env::var("CAPTURE_RESPONSES")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            test_data_dir: PathBuf::from("tests/test_data"),
        }
    }
}

impl TestConfig {
    /// Load configuration from environment and config files
    pub fn load() -> Self {
        let mut config = Self::default();

        // Try to load from config file if it exists
        if let Ok(config_content) = fs::read_to_string("tests/test_config.toml") {
            if let Ok(file_config) = toml::from_str::<TestConfig>(&config_content) {
                // Merge file config with environment config
                config.operation_timeout_secs = file_config.operation_timeout_secs;
                config.response_capture_dir = file_config.response_capture_dir;
                config.test_data_dir = file_config.test_data_dir;
            }
        }

        config
    }

    /// Check if integration tests should be run
    pub fn should_run_integration_tests(&self) -> bool {
        self.run_integration_tests && self.server_url.is_some()
    }

    /// Get timeout duration for operations
    pub fn timeout_duration(&self) -> Duration {
        Duration::from_secs(self.operation_timeout_secs)
    }
}

/// Response capture utility for collecting real server responses
#[derive(Debug)]
pub struct ResponseCapture {
    config: TestConfig,
    captured_responses: HashMap<String, CapturedResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedResponse {
    /// HTTP method used
    pub method: String,
    /// Endpoint path
    pub path: String,
    /// Request body (if any)
    pub request_body: Option<serde_json::Value>,
    /// Response status code
    pub status_code: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub response_body: serde_json::Value,
    /// Timestamp when captured
    pub captured_at: String,
    /// Test scenario this response belongs to
    pub scenario: String,
}

impl ResponseCapture {
    pub fn new(config: TestConfig) -> Self {
        Self {
            config,
            captured_responses: HashMap::new(),
        }
    }

    /// Record a response for later use in mock tests
    pub fn record_response(
        &mut self,
        scenario: &str,
        method: &str,
        path: &str,
        request_body: Option<serde_json::Value>,
        status_code: u16,
        headers: HashMap<String, String>,
        response_body: serde_json::Value,
    ) {
        let key = format!(
            "{}_{}_{}_{}",
            scenario,
            method,
            path.replace('/', "_"),
            status_code
        );

        let captured = CapturedResponse {
            method: method.to_string(),
            path: path.to_string(),
            request_body,
            status_code,
            headers,
            response_body,
            captured_at: chrono::Utc::now().to_rfc3339(),
            scenario: scenario.to_string(),
        };

        self.captured_responses.insert(key, captured);
    }

    /// Save all captured responses to files
    pub fn save_captured_responses(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.capture_responses {
            return Ok(());
        }

        // Create capture directory if it doesn't exist
        fs::create_dir_all(&self.config.response_capture_dir)?;

        // Group responses by scenario
        let mut scenarios: HashMap<String, Vec<&CapturedResponse>> = HashMap::new();
        for response in self.captured_responses.values() {
            scenarios
                .entry(response.scenario.clone())
                .or_insert_with(Vec::new)
                .push(response);
        }

        let scenario_count = scenarios.len();

        // Save each scenario to a separate file
        for (scenario, responses) in scenarios {
            let filename = format!("{}_responses.json", scenario);
            let filepath = self.config.response_capture_dir.join(filename);

            let json_content = serde_json::to_string_pretty(&responses)?;
            fs::write(filepath, json_content)?;
        }

        println!(
            "Captured {} responses across {} scenarios",
            self.captured_responses.len(),
            scenario_count
        );

        Ok(())
    }

    /// Load previously captured responses for a scenario
    pub fn load_captured_responses(
        &self,
        scenario: &str,
    ) -> Result<Vec<CapturedResponse>, Box<dyn std::error::Error>> {
        let filename = format!("{}_responses.json", scenario);
        let filepath = self.config.response_capture_dir.join(filename);

        if !filepath.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(filepath)?;
        let responses: Vec<CapturedResponse> = serde_json::from_str(&content)?;
        Ok(responses)
    }
}

/// Test data generator for creating comprehensive test scenarios
pub struct TestDataGenerator {
    config: TestConfig,
}

impl TestDataGenerator {
    pub fn new(config: TestConfig) -> Self {
        Self { config }
    }

    /// Generate test data files from templates
    pub fn generate_test_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.ensure_test_data_dir()?;

        // Generate catalog test data
        self.generate_catalog_test_data()?;

        // Generate schema test data
        self.generate_schema_test_data()?;

        // Generate error response test data
        self.generate_error_test_data()?;

        Ok(())
    }

    fn ensure_test_data_dir(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.config.test_data_dir)?;
        fs::create_dir_all(self.config.test_data_dir.join("catalogs"))?;
        fs::create_dir_all(self.config.test_data_dir.join("schemas"))?;
        fs::create_dir_all(self.config.test_data_dir.join("errors"))?;
        Ok(())
    }

    fn generate_catalog_test_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        let catalog_dir = self.config.test_data_dir.join("catalogs");

        // Create various catalog response scenarios
        let scenarios = vec![
            ("minimal_catalog", self.minimal_catalog_response()),
            ("full_catalog", self.full_catalog_response()),
            (
                "catalog_with_properties",
                self.catalog_with_properties_response(),
            ),
            ("sharing_catalog", self.sharing_catalog_response()),
            ("list_catalogs_paginated", self.paginated_list_response()),
            ("empty_catalog_list", self.empty_list_response()),
        ];

        for (name, response) in scenarios {
            let filename = format!("{}.json", name);
            let filepath = catalog_dir.join(filename);
            let json_content = serde_json::to_string_pretty(&response)?;
            fs::write(filepath, json_content)?;
        }

        Ok(())
    }

    fn generate_schema_test_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        let schema_dir = self.config.test_data_dir.join("schemas");

        let scenarios = vec![
            ("basic_schema", self.basic_schema_response()),
            (
                "schema_with_properties",
                self.schema_with_properties_response(),
            ),
            ("list_schemas", self.list_schemas_response()),
        ];

        for (name, response) in scenarios {
            let filename = format!("{}.json", name);
            let filepath = schema_dir.join(filename);
            let json_content = serde_json::to_string_pretty(&response)?;
            fs::write(filepath, json_content)?;
        }

        Ok(())
    }

    fn generate_error_test_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        let error_dir = self.config.test_data_dir.join("errors");

        let error_scenarios = vec![
            (
                "not_found",
                404,
                "RESOURCE_DOES_NOT_EXIST",
                "Resource not found",
            ),
            (
                "already_exists",
                409,
                "RESOURCE_ALREADY_EXISTS",
                "Resource already exists",
            ),
            (
                "permission_denied",
                403,
                "PERMISSION_DENIED",
                "Permission denied",
            ),
            (
                "invalid_parameter",
                400,
                "INVALID_PARAMETER_VALUE",
                "Invalid parameter value",
            ),
            (
                "internal_error",
                500,
                "INTERNAL_ERROR",
                "Internal server error",
            ),
            (
                "rate_limited",
                429,
                "RESOURCE_EXHAUSTED",
                "Rate limit exceeded",
            ),
        ];

        for (name, status, code, message) in error_scenarios {
            let error_response = self.error_response(status, code, message);
            let filename = format!("{}.json", name);
            let filepath = error_dir.join(filename);
            let json_content = serde_json::to_string_pretty(&error_response)?;
            fs::write(filepath, json_content)?;
        }

        Ok(())
    }

    // Response generators
    fn minimal_catalog_response(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "minimal_catalog",
            "comment": null,
            "storage_root": "s3://test-bucket/catalogs/minimal_catalog",
            "properties": {},
            "owner": "test-user",
            "created_at": 1699564800000i64,
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

    fn full_catalog_response(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "full_catalog",
            "comment": "A comprehensive catalog with all features",
            "storage_root": "s3://production-bucket/catalogs/full_catalog",
            "properties": {
                "environment": "production",
                "team": "data-platform",
                "cost-center": "engineering",
                "data-classification": "sensitive",
                "retention-policy": "7-years"
            },
            "owner": "data-admin",
            "created_at": 1699564800000i64,
            "updated_at": 1699651200000i64,
            "catalog_type": "MANAGED_CATALOG",
            "provider_name": null,
            "share_name": null,
            "isolation_mode": "ISOLATED",
            "options": {
                "enable_predictive_optimization": "true",
                "auto_maintenance": "enabled"
            },
            "effective_predictive_optimization_flag": {
                "value": "ENABLE",
                "inherited_from_type": "CATALOG"
            }
        })
    }

    fn catalog_with_properties_response(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "properties_catalog",
            "comment": "Catalog demonstrating custom properties",
            "storage_root": "abfss://container@storage.dfs.core.windows.net/catalogs/properties_catalog",
            "properties": {
                "business_unit": "analytics",
                "project": "customer-insights",
                "compliance": "gdpr,ccpa",
                "backup_enabled": "true",
                "encryption": "customer-managed-key"
            },
            "owner": "analytics-team",
            "created_at": 1699564800000i64,
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

    fn sharing_catalog_response(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "external_catalog",
            "comment": "Catalog shared from external provider",
            "storage_root": null,
            "properties": {
                "provider_region": "us-west-2",
                "sharing_protocol": "delta_sharing_v1"
            },
            "owner": "external-provider",
            "created_at": 1699564800000i64,
            "updated_at": 1699564800000i64,
            "catalog_type": "DELTASHARING_CATALOG",
            "provider_name": "external_data_provider",
            "share_name": "public_datasets",
            "isolation_mode": "ISOLATED",
            "options": {
                "delta_sharing_scope": "SELECT",
                "delta_sharing_recipient_token_lifetime_in_seconds": "3600"
            },
            "effective_predictive_optimization_flag": {
                "value": "INHERIT",
                "inherited_from_type": "SYSTEM_DEFAULT"
            }
        })
    }

    fn paginated_list_response(&self) -> serde_json::Value {
        serde_json::json!({
            "catalogs": [
                {
                    "name": "catalog_page1_item1",
                    "comment": "First catalog on first page",
                    "storage_root": "s3://bucket/catalogs/catalog_page1_item1",
                    "properties": {},
                    "owner": "user1",
                    "created_at": 1699564800000i64,
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
                },
                {
                    "name": "catalog_page1_item2",
                    "comment": "Second catalog on first page",
                    "storage_root": "s3://bucket/catalogs/catalog_page1_item2",
                    "properties": {},
                    "owner": "user2",
                    "created_at": 1699568400000i64,
                    "updated_at": 1699568400000i64,
                    "catalog_type": "MANAGED_CATALOG",
                    "provider_name": null,
                    "share_name": null,
                    "isolation_mode": "OPEN",
                    "options": {},
                    "effective_predictive_optimization_flag": {
                        "value": "INHERIT",
                        "inherited_from_type": "SYSTEM_DEFAULT"
                    }
                }
            ],
            "next_page_token": "page2_token_abc123"
        })
    }

    fn empty_list_response(&self) -> serde_json::Value {
        serde_json::json!({
            "catalogs": [],
            "next_page_token": null
        })
    }

    fn basic_schema_response(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "default_schema",
            "catalog_name": "test_catalog",
            "comment": "Default schema for testing",
            "properties": {},
            "owner": "test-user",
            "created_at": 1699564800000i64,
            "updated_at": 1699564800000i64,
            "schema_type": "MANAGED_SCHEMA",
            "storage_root": "s3://test-bucket/catalogs/test_catalog/schemas/default_schema",
            "full_name": "test_catalog.default_schema"
        })
    }

    fn schema_with_properties_response(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "analytics_schema",
            "catalog_name": "production_catalog",
            "comment": "Schema for analytics workloads",
            "properties": {
                "data_domain": "customer_analytics",
                "sla_tier": "gold",
                "backup_schedule": "daily"
            },
            "owner": "analytics-team",
            "created_at": 1699564800000i64,
            "updated_at": 1699564800000i64,
            "schema_type": "MANAGED_SCHEMA",
            "storage_root": "s3://prod-bucket/catalogs/production_catalog/schemas/analytics_schema",
            "full_name": "production_catalog.analytics_schema"
        })
    }

    fn list_schemas_response(&self) -> serde_json::Value {
        serde_json::json!({
            "schemas": [
                {
                    "name": "default",
                    "catalog_name": "test_catalog",
                    "comment": null,
                    "properties": {},
                    "owner": "system",
                    "created_at": 1699564800000i64,
                    "updated_at": 1699564800000i64,
                    "schema_type": "MANAGED_SCHEMA",
                    "storage_root": "s3://bucket/catalogs/test_catalog/schemas/default",
                    "full_name": "test_catalog.default"
                },
                {
                    "name": "staging",
                    "catalog_name": "test_catalog",
                    "comment": "Staging area for data processing",
                    "properties": {
                        "purpose": "staging"
                    },
                    "owner": "data-engineer",
                    "created_at": 1699568400000i64,
                    "updated_at": 1699568400000i64,
                    "schema_type": "MANAGED_SCHEMA",
                    "storage_root": "s3://bucket/catalogs/test_catalog/schemas/staging",
                    "full_name": "test_catalog.staging"
                }
            ],
            "next_page_token": null
        })
    }

    fn error_response(&self, status: u16, code: &str, message: &str) -> serde_json::Value {
        serde_json::json!({
            "error_code": code,
            "message": message,
            "details": [
                {
                    "reason": code,
                    "domain": "UNITY_CATALOG",
                    "metadata": {
                        "http_status": status
                    }
                }
            ]
        })
    }
}

/// Integration test runner that can execute tests against real or mock servers
pub struct IntegrationTestRunner {
    config: TestConfig,
    response_capture: ResponseCapture,
}

impl IntegrationTestRunner {
    pub fn new() -> Self {
        let config = TestConfig::load();
        let response_capture = ResponseCapture::new(config.clone());

        Self {
            config,
            response_capture,
        }
    }

    /// Run a test operation with timeout and optional response capture
    pub async fn run_test_operation<F, Fut, T>(
        &mut self,
        scenario: &str,
        operation_name: &str,
        operation: F,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error>>>,
    {
        println!(
            "Running test operation: {} in scenario: {}",
            operation_name, scenario
        );

        let result = timeout(self.config.timeout_duration(), operation()).await;

        match result {
            Ok(op_result) => match op_result {
                Ok(value) => {
                    println!("✅ {} completed successfully", operation_name);
                    Ok(value)
                }
                Err(e) => {
                    println!("❌ {} failed: {}", operation_name, e);
                    Err(e)
                }
            },
            Err(_) => {
                let error_msg = format!(
                    "{} timed out after {} seconds",
                    operation_name, self.config.operation_timeout_secs
                );
                println!("⏰ {}", error_msg);
                Err(error_msg.into())
            }
        }
    }

    /// Finalize test run and save captured responses
    pub fn finalize(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.response_capture.save_captured_responses()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_defaults() {
        let config = TestConfig::default();
        assert!(!config.run_integration_tests);
        assert_eq!(config.operation_timeout_secs, 30);
    }

    #[test]
    fn test_response_capture() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = TestConfig::default();
        config.response_capture_dir = temp_dir.path().to_path_buf();
        config.capture_responses = true;

        let mut capture = ResponseCapture::new(config);

        let headers = HashMap::new();
        let response_body = serde_json::json!({"name": "test"});

        capture.record_response(
            "test_scenario",
            "GET",
            "/api/catalogs/test",
            None,
            200,
            headers,
            response_body,
        );

        capture.save_captured_responses().unwrap();

        // Verify file was created
        let response_file = temp_dir.path().join("test_scenario_responses.json");
        assert!(response_file.exists());
    }

    #[test]
    fn test_data_generator() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = TestConfig::default();
        config.test_data_dir = temp_dir.path().to_path_buf();

        let generator = TestDataGenerator::new(config);
        generator.generate_test_data().unwrap();

        // Verify directories were created
        assert!(temp_dir.path().join("catalogs").exists());
        assert!(temp_dir.path().join("schemas").exists());
        assert!(temp_dir.path().join("errors").exists());

        // Verify some test files were created
        assert!(
            temp_dir
                .path()
                .join("catalogs/minimal_catalog.json")
                .exists()
        );
        assert!(temp_dir.path().join("errors/not_found.json").exists());
    }
}
