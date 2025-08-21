//! Shared data models and utilities for Unity Catalog acceptance testing
//!
//! This module provides common data structures, builders, and utilities
//! that are used across different acceptance testing scenarios.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Configuration for integration testing
#[derive(Debug, Clone)]
pub struct IntegrationConfig {
    /// Whether integration tests are enabled
    pub enabled: bool,
    /// Unity Catalog server URL
    pub server_url: Option<String>,
    /// Authentication token
    pub auth_token: Option<String>,
    /// Whether to record responses during tests
    pub record_responses: bool,
    /// Whether to overwrite existing recordings
    pub overwrite_recordings: bool,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
}

impl IntegrationConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            enabled: std::env::var("RUN_INTEGRATION_TESTS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            server_url: std::env::var("UC_SERVER_URL").ok(),
            auth_token: std::env::var("UC_AUTH_TOKEN").ok(),
            record_responses: std::env::var("RECORD_JOURNEY_RESPONSES")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            overwrite_recordings: std::env::var("OVERWRITE_JOURNEY_RESPONSES")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            request_timeout_secs: std::env::var("REQUEST_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
        }
    }

    /// Check if integration testing is properly configured
    pub fn is_valid(&self) -> bool {
        if !self.enabled {
            return true; // Valid for mock-only testing
        }

        self.server_url.is_some() && self.auth_token.is_some()
    }

    /// Get the server URL or default
    pub fn server_url(&self) -> String {
        self.server_url
            .as_deref()
            .unwrap_or("http://localhost:8080")
            .to_string()
    }
}

/// Test execution context that carries state across test steps
#[derive(Debug, Clone)]
pub struct TestContext {
    /// Test run identifier
    pub test_id: String,
    /// Integration configuration
    pub config: IntegrationConfig,
    /// Variables available for substitution
    pub variables: HashMap<String, Value>,
    /// Metadata about the test execution
    pub metadata: HashMap<String, Value>,
}

impl TestContext {
    /// Create a new test context
    pub fn new(test_id: String) -> Self {
        Self {
            test_id,
            config: IntegrationConfig::from_env(),
            variables: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a variable to the context
    pub fn set_variable(&mut self, key: &str, value: Value) {
        self.variables.insert(key.to_string(), value);
    }

    /// Get a variable from the context
    pub fn get_variable(&self, key: &str) -> Option<&Value> {
        self.variables.get(key)
    }

    /// Add metadata to the context
    pub fn set_metadata(&mut self, key: &str, value: Value) {
        self.metadata.insert(key.to_string(), value);
    }

    /// Generate a unique resource name for this test
    pub fn unique_name(&self, prefix: &str) -> String {
        let suffix_len = std::cmp::min(8, self.test_id.len());
        format!("{}_{}", prefix, &self.test_id[..suffix_len])
    }
}

/// Builder for creating catalog test data
#[derive(Debug, Clone)]
pub struct CatalogBuilder {
    name: String,
    comment: Option<String>,
    storage_root: Option<String>,
    properties: HashMap<String, String>,
}

impl CatalogBuilder {
    /// Create a new catalog builder
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            comment: None,
            storage_root: None,
            properties: HashMap::new(),
        }
    }

    /// Set the catalog comment
    pub fn with_comment(mut self, comment: &str) -> Self {
        self.comment = Some(comment.to_string());
        self
    }

    /// Set the storage root
    pub fn with_storage_root(mut self, storage_root: &str) -> Self {
        self.storage_root = Some(storage_root.to_string());
        self
    }

    /// Add a property
    pub fn with_property(mut self, key: &str, value: &str) -> Self {
        self.properties.insert(key.to_string(), value.to_string());
        self
    }

    /// Build the catalog as JSON
    pub fn build_json(&self) -> Value {
        let mut catalog = serde_json::json!({
            "name": self.name
        });

        if let Some(comment) = &self.comment {
            catalog["comment"] = Value::String(comment.clone());
        }

        if let Some(storage_root) = &self.storage_root {
            catalog["storage_root"] = Value::String(storage_root.clone());
        }

        if !self.properties.is_empty() {
            catalog["properties"] = serde_json::to_value(&self.properties).unwrap();
        }

        catalog
    }

    /// Build the catalog create request
    pub fn build_create_request(&self) -> Value {
        self.build_json()
    }

    /// Build the catalog update request
    pub fn build_update_request(&self, new_comment: Option<&str>) -> Value {
        let mut update = serde_json::json!({
            "name": self.name
        });

        if let Some(comment) = new_comment {
            update["comment"] = Value::String(comment.to_string());
        }

        update
    }
}

/// Builder for creating schema test data
#[derive(Debug, Clone)]
pub struct SchemaBuilder {
    name: String,
    catalog_name: String,
    comment: Option<String>,
    properties: HashMap<String, String>,
}

impl SchemaBuilder {
    /// Create a new schema builder
    pub fn new(catalog_name: &str, name: &str) -> Self {
        Self {
            name: name.to_string(),
            catalog_name: catalog_name.to_string(),
            comment: None,
            properties: HashMap::new(),
        }
    }

    /// Set the schema comment
    pub fn with_comment(mut self, comment: &str) -> Self {
        self.comment = Some(comment.to_string());
        self
    }

    /// Add a property
    pub fn with_property(mut self, key: &str, value: &str) -> Self {
        self.properties.insert(key.to_string(), value.to_string());
        self
    }

    /// Build the schema as JSON
    pub fn build_json(&self) -> Value {
        let mut schema = serde_json::json!({
            "name": self.name,
            "catalog_name": self.catalog_name
        });

        if let Some(comment) = &self.comment {
            schema["comment"] = Value::String(comment.clone());
        }

        if !self.properties.is_empty() {
            schema["properties"] = serde_json::to_value(&self.properties).unwrap();
        }

        schema
    }

    /// Build the schema create request
    pub fn build_create_request(&self) -> Value {
        self.build_json()
    }
}

/// Builder for creating table test data
#[derive(Debug, Clone)]
pub struct TableBuilder {
    name: String,
    catalog_name: String,
    schema_name: String,
    table_type: String,
    data_source_format: String,
    comment: Option<String>,
    columns: Vec<ColumnInfo>,
    properties: HashMap<String, String>,
}

impl TableBuilder {
    /// Create a new table builder
    pub fn new(catalog_name: &str, schema_name: &str, name: &str) -> Self {
        Self {
            name: name.to_string(),
            catalog_name: catalog_name.to_string(),
            schema_name: schema_name.to_string(),
            table_type: "MANAGED".to_string(),
            data_source_format: "DELTA".to_string(),
            comment: None,
            columns: Vec::new(),
            properties: HashMap::new(),
        }
    }

    /// Set the table type
    pub fn with_table_type(mut self, table_type: &str) -> Self {
        self.table_type = table_type.to_string();
        self
    }

    /// Set the data source format
    pub fn with_data_source_format(mut self, format: &str) -> Self {
        self.data_source_format = format.to_string();
        self
    }

    /// Set the table comment
    pub fn with_comment(mut self, comment: &str) -> Self {
        self.comment = Some(comment.to_string());
        self
    }

    /// Add a column
    pub fn with_column(mut self, column: ColumnInfo) -> Self {
        self.columns.push(column);
        self
    }

    /// Add a simple column
    pub fn with_simple_column(mut self, name: &str, type_name: &str, nullable: bool) -> Self {
        self.columns.push(ColumnInfo {
            name: name.to_string(),
            type_name: type_name.to_string(),
            type_text: type_name.to_lowercase(),
            nullable,
            comment: None,
        });
        self
    }

    /// Add a property
    pub fn with_property(mut self, key: &str, value: &str) -> Self {
        self.properties.insert(key.to_string(), value.to_string());
        self
    }

    /// Build the table as JSON
    pub fn build_json(&self) -> Value {
        let mut table = serde_json::json!({
            "name": self.name,
            "catalog_name": self.catalog_name,
            "schema_name": self.schema_name,
            "table_type": self.table_type,
            "data_source_format": self.data_source_format
        });

        if let Some(comment) = &self.comment {
            table["comment"] = Value::String(comment.clone());
        }

        if !self.columns.is_empty() {
            table["columns"] = serde_json::to_value(&self.columns).unwrap();
        }

        if !self.properties.is_empty() {
            table["properties"] = serde_json::to_value(&self.properties).unwrap();
        }

        table
    }

    /// Build the table create request
    pub fn build_create_request(&self) -> Value {
        self.build_json()
    }
}

/// Column information for table definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    /// Column name
    pub name: String,
    /// Column type name (e.g., "BIGINT", "STRING")
    pub type_name: String,
    /// Column type text (e.g., "bigint", "string")
    pub type_text: String,
    /// Whether the column is nullable
    pub nullable: bool,
    /// Optional column comment
    pub comment: Option<String>,
}

impl ColumnInfo {
    /// Create a new column info
    pub fn new(name: &str, type_name: &str, nullable: bool) -> Self {
        Self {
            name: name.to_string(),
            type_name: type_name.to_uppercase(),
            type_text: type_name.to_lowercase(),
            nullable,
            comment: None,
        }
    }

    /// Add a comment to the column
    pub fn with_comment(mut self, comment: &str) -> Self {
        self.comment = Some(comment.to_string());
        self
    }
}

/// Utility functions for test data generation
pub struct TestDataUtils;

impl TestDataUtils {
    /// Generate a test timestamp
    pub fn timestamp() -> i64 {
        chrono::Utc::now().timestamp_millis()
    }

    /// Generate a unique identifier
    pub fn unique_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Generate a short unique identifier (8 chars)
    pub fn short_id() -> String {
        uuid::Uuid::new_v4().to_string().replace('-', "")[..8].to_lowercase()
    }

    /// Generate test properties
    pub fn test_properties() -> HashMap<String, String> {
        let mut props = HashMap::new();
        props.insert("environment".to_string(), "test".to_string());
        props.insert("created_by".to_string(), "acceptance_test".to_string());
        props.insert("test_run_id".to_string(), Self::short_id());
        props
    }

    /// Clean up test resource names to be valid Unity Catalog identifiers
    pub fn sanitize_name(name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .trim_start_matches(|c: char| c.is_ascii_digit())
            .to_string()
    }

    /// Generate common column sets for testing
    pub fn standard_columns() -> Vec<ColumnInfo> {
        vec![
            ColumnInfo::new("id", "BIGINT", false).with_comment("Primary key"),
            ColumnInfo::new("name", "STRING", true).with_comment("Name field"),
            ColumnInfo::new("created_at", "TIMESTAMP", false).with_comment("Creation timestamp"),
            ColumnInfo::new("updated_at", "TIMESTAMP", true).with_comment("Update timestamp"),
        ]
    }

    /// Generate basic columns for simple tables
    pub fn basic_columns() -> Vec<ColumnInfo> {
        vec![
            ColumnInfo::new("id", "BIGINT", false),
            ColumnInfo::new("value", "STRING", true),
        ]
    }

    /// Convert environment variables to test variables
    pub fn env_to_variables() -> HashMap<String, Value> {
        let mut variables = HashMap::new();

        // Common test variables from environment
        if let Ok(catalog_prefix) = std::env::var("TEST_CATALOG_PREFIX") {
            variables.insert("catalog_prefix".to_string(), Value::String(catalog_prefix));
        }

        if let Ok(test_suffix) = std::env::var("TEST_SUFFIX") {
            variables.insert("test_suffix".to_string(), Value::String(test_suffix));
        }

        // Generate default values if not provided
        if !variables.contains_key("catalog_prefix") {
            variables.insert(
                "catalog_prefix".to_string(),
                Value::String("test".to_string()),
            );
        }

        if !variables.contains_key("test_suffix") {
            variables.insert("test_suffix".to_string(), Value::String(Self::short_id()));
        }

        variables.insert(
            "timestamp".to_string(),
            Value::Number(serde_json::Number::from(Self::timestamp())),
        );
        variables.insert("test_id".to_string(), Value::String(Self::unique_id()));

        variables
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_config_from_env() {
        // Test with default values (no env vars set)
        let config = IntegrationConfig::from_env();
        assert!(!config.enabled);
        assert_eq!(config.request_timeout_secs, 30);
    }

    #[test]
    fn test_test_context() {
        let mut context = TestContext::new("test123".to_string());

        context.set_variable("test_var", Value::String("test_value".to_string()));
        assert_eq!(
            context.get_variable("test_var"),
            Some(&Value::String("test_value".to_string()))
        );

        let unique_name = context.unique_name("catalog");
        assert!(unique_name.starts_with("catalog_"));
        let suffix_len = std::cmp::min(8, context.test_id.len());
        assert_eq!(unique_name.len(), "catalog_".len() + suffix_len);
    }

    #[test]
    fn test_catalog_builder() {
        let catalog = CatalogBuilder::new("test_catalog")
            .with_comment("Test catalog")
            .with_storage_root("s3://test-bucket/catalogs/test")
            .with_property("environment", "test")
            .build_json();

        assert_eq!(catalog["name"], "test_catalog");
        assert_eq!(catalog["comment"], "Test catalog");
        assert_eq!(catalog["storage_root"], "s3://test-bucket/catalogs/test");
        assert_eq!(catalog["properties"]["environment"], "test");
    }

    #[test]
    fn test_schema_builder() {
        let schema = SchemaBuilder::new("test_catalog", "test_schema")
            .with_comment("Test schema")
            .with_property("owner", "test-user")
            .build_json();

        assert_eq!(schema["name"], "test_schema");
        assert_eq!(schema["catalog_name"], "test_catalog");
        assert_eq!(schema["comment"], "Test schema");
        assert_eq!(schema["properties"]["owner"], "test-user");
    }

    #[test]
    fn test_table_builder() {
        let table = TableBuilder::new("test_catalog", "test_schema", "test_table")
            .with_comment("Test table")
            .with_table_type("EXTERNAL")
            .with_data_source_format("PARQUET")
            .with_simple_column("id", "BIGINT", false)
            .with_simple_column("name", "STRING", true)
            .build_json();

        assert_eq!(table["name"], "test_table");
        assert_eq!(table["catalog_name"], "test_catalog");
        assert_eq!(table["schema_name"], "test_schema");
        assert_eq!(table["table_type"], "EXTERNAL");
        assert_eq!(table["data_source_format"], "PARQUET");
        assert_eq!(table["comment"], "Test table");

        let columns = table["columns"].as_array().unwrap();
        assert_eq!(columns.len(), 2);
        assert_eq!(columns[0]["name"], "id");
        assert_eq!(columns[0]["type_name"], "BIGINT");
        assert_eq!(columns[0]["nullable"], false);
    }

    #[test]
    fn test_column_info() {
        let column = ColumnInfo::new("test_col", "varchar", true).with_comment("Test column");

        assert_eq!(column.name, "test_col");
        assert_eq!(column.type_name, "VARCHAR");
        assert_eq!(column.type_text, "varchar");
        assert!(column.nullable);
        assert_eq!(column.comment, Some("Test column".to_string()));
    }

    #[test]
    fn test_test_data_utils() {
        let timestamp = TestDataUtils::timestamp();
        assert!(timestamp > 0);

        let id = TestDataUtils::unique_id();
        assert!(id.len() > 30); // UUID format

        let short_id = TestDataUtils::short_id();
        assert_eq!(short_id.len(), 8);

        let sanitized = TestDataUtils::sanitize_name("123-invalid@name!");
        assert!(!sanitized.starts_with(char::is_numeric));
        assert!(!sanitized.contains('@'));
        assert!(!sanitized.contains('!'));

        let columns = TestDataUtils::standard_columns();
        assert_eq!(columns.len(), 4);
        assert_eq!(columns[0].name, "id");

        let props = TestDataUtils::test_properties();
        assert!(props.contains_key("environment"));
        assert_eq!(props["environment"], "test");
    }

    #[test]
    fn test_env_to_variables() {
        let variables = TestDataUtils::env_to_variables();

        // Should always have default values
        assert!(variables.contains_key("catalog_prefix"));
        assert!(variables.contains_key("test_suffix"));
        assert!(variables.contains_key("timestamp"));
        assert!(variables.contains_key("test_id"));
    }
}
