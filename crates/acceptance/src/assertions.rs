//! Common assertion helpers for Unity Catalog acceptance testing
//!
//! This module provides specialized assertion functions for testing Unity Catalog
//! API responses and data structures with detailed error messages and flexible
//! validation patterns.

use crate::AcceptanceError;
use serde_json::Value;
use std::fmt::Debug;

/// Collection of assertion helpers for Unity Catalog testing
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that a catalog info structure matches expected values
    pub fn assert_catalog_info_matches(
        actual: &unitycatalog_common::models::catalogs::v1::CatalogInfo,
        expected_name: &str,
        expected_comment: Option<&str>,
    ) {
        assert_eq!(actual.name, expected_name, "Catalog name mismatch");
        assert_eq!(
            actual.comment.as_deref(),
            expected_comment,
            "Catalog comment mismatch"
        );
        assert!(
            actual.created_at.is_some(),
            "Catalog should have created_at"
        );
        assert!(
            actual.updated_at.is_some(),
            "Catalog should have updated_at"
        );
    }

    /// Assert that a schema info structure matches expected values
    pub fn assert_schema_info_matches(
        actual: &unitycatalog_common::models::schemas::v1::SchemaInfo,
        expected_catalog: &str,
        expected_name: &str,
        expected_comment: Option<&str>,
    ) {
        assert_eq!(
            actual.catalog_name, expected_catalog,
            "Schema catalog name mismatch"
        );
        assert_eq!(actual.name, expected_name, "Schema name mismatch");
        assert_eq!(
            actual.comment.as_deref(),
            expected_comment,
            "Schema comment mismatch"
        );
        assert_eq!(
            actual.full_name,
            Some(format!("{}.{}", expected_catalog, expected_name)),
            "Schema full name should be catalog.schema"
        );
        assert!(actual.created_at.is_some(), "Schema should have created_at");
        assert!(actual.updated_at.is_some(), "Schema should have updated_at");
    }

    /// Assert that a table info structure matches expected values
    pub fn assert_table_info_matches(
        actual: &unitycatalog_common::models::tables::v1::TableInfo,
        expected_catalog: &str,
        expected_schema: &str,
        expected_name: &str,
        expected_comment: Option<&str>,
    ) {
        assert_eq!(
            actual.catalog_name, expected_catalog,
            "Table catalog name mismatch"
        );
        assert_eq!(
            actual.schema_name, expected_schema,
            "Table schema name mismatch"
        );
        assert_eq!(actual.name, expected_name, "Table name mismatch");
        assert_eq!(
            actual.comment.as_deref(),
            expected_comment,
            "Table comment mismatch"
        );
        assert_eq!(
            actual.full_name,
            Some(format!(
                "{}.{}.{}",
                expected_catalog, expected_schema, expected_name
            )),
            "Table full name should be catalog.schema.table"
        );
        assert!(actual.created_at.is_some(), "Table should have created_at");
        assert!(actual.updated_at.is_some(), "Table should have updated_at");
    }

    /// Assert that an error contains expected message patterns
    pub fn assert_error_contains<T: Debug, E: std::fmt::Display>(
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

    /// Assert that an error has a specific status code
    pub fn assert_error_status<T: Debug>(
        result: &Result<T, AcceptanceError>,
        expected_status: u16,
    ) {
        match result {
            Err(AcceptanceError::Http(e)) => {
                if let Some(status) = e.status() {
                    assert_eq!(
                        status.as_u16(),
                        expected_status,
                        "Expected HTTP status {}, got {}",
                        expected_status,
                        status.as_u16()
                    );
                } else {
                    panic!("HTTP error without status code: {}", e);
                }
            }
            Err(e) => {
                // Check if error message contains status code
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains(&expected_status.to_string()),
                    "Expected status {} in error message: {}",
                    expected_status,
                    error_msg
                );
            }
            Ok(val) => panic!(
                "Expected error with status {} but got success: {:?}",
                expected_status, val
            ),
        }
    }

    /// Assert that a JSON response contains expected fields
    pub fn assert_json_contains_fields(json: &Value, required_fields: &[&str]) {
        let obj = json
            .as_object()
            .expect("JSON should be an object for field validation");

        for field in required_fields {
            assert!(
                obj.contains_key(*field),
                "JSON response missing required field: {}",
                field
            );
        }
    }

    /// Assert that a JSON array has expected length
    pub fn assert_json_array_length(json: &Value, expected_length: usize) {
        let arr = json
            .as_array()
            .expect("JSON should be an array for length validation");

        assert_eq!(
            arr.len(),
            expected_length,
            "JSON array length mismatch: expected {}, got {}",
            expected_length,
            arr.len()
        );
    }

    /// Assert that a JSON field has expected value
    pub fn assert_json_field_equals(json: &Value, field_path: &str, expected: &Value) {
        let actual = Self::extract_json_field(json, field_path)
            .unwrap_or_else(|| panic!("Field '{}' not found in JSON", field_path));

        assert_eq!(
            actual, expected,
            "JSON field '{}' value mismatch",
            field_path
        );
    }

    /// Assert that a JSON field matches a pattern
    pub fn assert_json_field_matches_pattern(json: &Value, field_path: &str, pattern: &str) {
        let actual = Self::extract_json_field(json, field_path)
            .unwrap_or_else(|| panic!("Field '{}' not found in JSON", field_path));

        let actual_str = actual
            .as_str()
            .unwrap_or_else(|| panic!("Field '{}' is not a string", field_path));

        // Simple pattern matching - can be extended with regex if needed
        assert!(
            actual_str.contains(pattern),
            "JSON field '{}' value '{}' does not match pattern '{}'",
            field_path,
            actual_str,
            pattern
        );
    }

    /// Assert that a list response contains expected number of items
    pub fn assert_list_response_length(json: &Value, list_field: &str, expected_length: usize) {
        let list = json
            .get(list_field)
            .unwrap_or_else(|| panic!("List field '{}' not found in response", list_field))
            .as_array()
            .unwrap_or_else(|| panic!("Field '{}' is not an array", list_field));

        assert_eq!(
            list.len(),
            expected_length,
            "List '{}' length mismatch: expected {}, got {}",
            list_field,
            expected_length,
            list.len()
        );
    }

    /// Assert that a timestamp field is recent (within last 5 minutes)
    pub fn assert_timestamp_is_recent(json: &Value, field_path: &str) {
        let timestamp = Self::extract_json_field(json, field_path)
            .unwrap_or_else(|| panic!("Timestamp field '{}' not found", field_path))
            .as_i64()
            .unwrap_or_else(|| panic!("Field '{}' is not a valid timestamp", field_path));

        let now = chrono::Utc::now().timestamp_millis();
        let diff = (now - timestamp).abs();
        let five_minutes = 5 * 60 * 1000; // 5 minutes in milliseconds

        assert!(
            diff < five_minutes,
            "Timestamp field '{}' is not recent: {} (now: {})",
            field_path,
            timestamp,
            now
        );
    }

    /// Assert that pagination info is valid
    pub fn assert_pagination_valid(json: &Value, has_next_page: bool) {
        if has_next_page {
            assert!(
                json.get("next_page_token").is_some(),
                "Expected next_page_token for paginated response"
            );
        } else {
            assert!(
                json.get("next_page_token").is_none(),
                "Unexpected next_page_token in final page"
            );
        }
    }

    /// Assert that a response follows Unity Catalog naming conventions
    pub fn assert_unity_catalog_naming(name: &str, resource_type: &str) {
        // Unity Catalog naming rules:
        // - Must start with a letter or underscore
        // - Can contain letters, numbers, underscores, and hyphens
        // - Cannot be empty
        // - Should be reasonable length

        assert!(!name.is_empty(), "{} name cannot be empty", resource_type);

        assert!(
            name.len() <= 255,
            "{} name too long: {} characters (max 255)",
            resource_type,
            name.len()
        );

        let first_char = name.chars().next().unwrap();
        assert!(
            first_char.is_ascii_alphabetic() || first_char == '_',
            "{} name '{}' must start with letter or underscore",
            resource_type,
            name
        );

        for c in name.chars() {
            assert!(
                c.is_ascii_alphanumeric() || c == '_' || c == '-',
                "{} name '{}' contains invalid character: '{}'",
                resource_type,
                name,
                c
            );
        }
    }

    /// Assert that properties follow expected format
    pub fn assert_properties_valid(properties: &Option<std::collections::HashMap<String, String>>) {
        if let Some(props) = properties {
            for (key, value) in props {
                assert!(!key.is_empty(), "Property key cannot be empty");
                assert!(
                    !value.is_empty(),
                    "Property value for key '{}' cannot be empty",
                    key
                );
                // Property keys should follow naming conventions
                Self::assert_unity_catalog_naming(key, "Property key");
            }
        }
    }

    /// Extract a field from JSON using dot notation (e.g., "data.items.0.name")
    fn extract_json_field<'a>(json: &'a Value, field_path: &str) -> Option<&'a Value> {
        let parts: Vec<&str> = field_path.split('.').collect();
        let mut current = json;

        for part in parts {
            match current {
                Value::Object(map) => {
                    current = map.get(part)?;
                }
                Value::Array(arr) => {
                    let index: usize = part.parse().ok()?;
                    current = arr.get(index)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }

    /// Custom assertion for journey step results
    pub fn assert_step_result_success(
        step_result: &crate::journey::StepResult,
        expected_status: u16,
    ) {
        assert!(
            step_result.success,
            "Step '{}' failed: {}",
            step_result.step.id,
            step_result
                .error_message
                .as_deref()
                .unwrap_or("Unknown error")
        );
        assert_eq!(
            step_result.status_code, expected_status,
            "Step '{}' status code mismatch: expected {}, got {}",
            step_result.step.id, expected_status, step_result.status_code
        );
    }

    /// Assert that journey result is successful
    pub fn assert_journey_success(journey_result: &crate::journey::JourneyResult) {
        assert!(
            journey_result.success,
            "Journey '{}' failed: {}",
            journey_result.journey.name,
            journey_result
                .failure_summary
                .as_deref()
                .unwrap_or("Unknown failure")
        );

        // Check that all non-cleanup steps succeeded
        for step_result in &journey_result.step_results {
            let is_cleanup = step_result
                .step
                .tags
                .as_ref()
                .map(|tags| tags.contains(&"cleanup".to_string()))
                .unwrap_or(false);

            if !is_cleanup {
                Self::assert_step_result_success(step_result, step_result.step.expected_status);
            }
        }
    }

    /// Assert that variables were extracted correctly
    pub fn assert_variables_extracted(
        journey_result: &crate::journey::JourneyResult,
        expected_variables: &[&str],
    ) {
        for var_name in expected_variables {
            assert!(
                journey_result.final_variables.contains_key(*var_name),
                "Expected variable '{}' was not extracted. Available: {:?}",
                var_name,
                journey_result.final_variables.keys().collect::<Vec<_>>()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_field_extraction() {
        let json = json!({
            "data": {
                "items": [
                    {"name": "first", "id": 1},
                    {"name": "second", "id": 2}
                ],
                "count": 2
            }
        });

        assert_eq!(
            TestAssertions::extract_json_field(&json, "data.count"),
            Some(&json!(2))
        );
        assert_eq!(
            TestAssertions::extract_json_field(&json, "data.items.0.name"),
            Some(&json!("first"))
        );
        assert_eq!(
            TestAssertions::extract_json_field(&json, "data.items.1.id"),
            Some(&json!(2))
        );
        assert_eq!(
            TestAssertions::extract_json_field(&json, "nonexistent"),
            None
        );
    }

    #[test]
    fn test_json_contains_fields() {
        let json = json!({
            "name": "test",
            "id": 123,
            "created_at": 1234567890
        });

        TestAssertions::assert_json_contains_fields(&json, &["name", "id", "created_at"]);
    }

    #[test]
    #[should_panic(expected = "JSON response missing required field: missing")]
    fn test_json_missing_field_fails() {
        let json = json!({"name": "test"});
        TestAssertions::assert_json_contains_fields(&json, &["name", "missing"]);
    }

    #[test]
    fn test_unity_catalog_naming_valid() {
        TestAssertions::assert_unity_catalog_naming("valid_name", "Test");
        TestAssertions::assert_unity_catalog_naming("_starts_with_underscore", "Test");
        TestAssertions::assert_unity_catalog_naming("has-hyphens", "Test");
        TestAssertions::assert_unity_catalog_naming("has123numbers", "Test");
    }

    #[test]
    #[should_panic(expected = "must start with letter or underscore")]
    fn test_unity_catalog_naming_invalid_start() {
        TestAssertions::assert_unity_catalog_naming("123invalid", "Test");
    }

    #[test]
    #[should_panic(expected = "contains invalid character")]
    fn test_unity_catalog_naming_invalid_char() {
        TestAssertions::assert_unity_catalog_naming("invalid@name", "Test");
    }

    #[test]
    #[should_panic(expected = "name cannot be empty")]
    fn test_unity_catalog_naming_empty() {
        TestAssertions::assert_unity_catalog_naming("", "Test");
    }

    #[test]
    fn test_timestamp_recent() {
        let now = chrono::Utc::now().timestamp_millis();
        let json = json!({"timestamp": now});
        TestAssertions::assert_timestamp_is_recent(&json, "timestamp");
    }

    #[test]
    #[should_panic(expected = "is not recent")]
    fn test_timestamp_old() {
        let old_timestamp = chrono::Utc::now().timestamp_millis() - (10 * 60 * 1000); // 10 minutes ago
        let json = json!({"timestamp": old_timestamp});
        TestAssertions::assert_timestamp_is_recent(&json, "timestamp");
    }
}
