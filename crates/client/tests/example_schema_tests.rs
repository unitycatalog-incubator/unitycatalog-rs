//! Example schema client tests demonstrating how to extend the test framework
//!
//! This file shows how to apply the same testing patterns used for catalogs
//! to other Unity Catalog client types like schemas, tables, etc.

use rstest::*;
use serde_json::json;
use std::collections::HashMap;
use unitycatalog_client::UnityCatalogClient;

mod test_utils;
use test_utils::{
    TestAssertions, TestDataLoader, TestServer,
    fixtures::*,
    responses::{CatalogResponses, ErrorResponses},
};

/// Test schema creation within a catalog
#[rstest]
#[tokio::test]
async fn test_create_schema_basic(
    #[future] test_client: (UnityCatalogClient, TestServer),
    catalog_name: String,
) {
    let (client, mut server) = test_client.await;
    let schema_name = "test_schema";

    // Setup mock response for schema creation
    let expected_response = json!({
        "name": schema_name,
        "catalog_name": catalog_name,
        "comment": "Test schema for demonstration",
        "properties": {},
        "owner": "test-user",
        "created_at": 1699564800000i64,
        "updated_at": 1699564800000i64,
        "schema_type": "MANAGED_SCHEMA",
        "storage_root": format!("s3://test-bucket/catalogs/{}/schemas/{}", catalog_name, schema_name),
        "full_name": format!("{}.{}", catalog_name, schema_name)
    });

    let mock = server
        .mock_catalog_endpoint("POST", "/schemas")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&expected_response).unwrap())
        .create();

    // Execute test
    let schema_result = client
        .create_schema(
            &catalog_name,
            schema_name,
            Some("Test schema for demonstration"),
        )
        .await;

    // Verify request was made
    mock.assert();

    // Verify response
    assert!(schema_result.is_ok());
    let schema_info = schema_result.unwrap();
    assert_eq!(schema_info.name, schema_name);
    assert_eq!(schema_info.catalog_name, catalog_name);
    assert_eq!(
        schema_info.comment,
        Some("Test schema for demonstration".to_string())
    );
}

/// Test listing schemas in a catalog
#[rstest]
#[tokio::test]
async fn test_list_schemas(
    #[future] test_client: (UnityCatalogClient, TestServer),
    catalog_name: String,
) {
    let (client, mut server) = test_client.await;

    // Create mock response with multiple schemas
    let expected_response = json!({
        "schemas": [
            {
                "name": "default",
                "catalog_name": catalog_name,
                "comment": null,
                "properties": {},
                "owner": "system",
                "created_at": 1699564800000i64,
                "updated_at": 1699564800000i64,
                "schema_type": "MANAGED_SCHEMA",
                "storage_root": format!("s3://bucket/catalogs/{}/schemas/default", catalog_name),
                "full_name": format!("{}.default", catalog_name)
            },
            {
                "name": "analytics",
                "catalog_name": catalog_name,
                "comment": "Analytics schema",
                "properties": {
                    "purpose": "analytics"
                },
                "owner": "data-team",
                "created_at": 1699568400000i64,
                "updated_at": 1699568400000i64,
                "schema_type": "MANAGED_SCHEMA",
                "storage_root": format!("s3://bucket/catalogs/{}/schemas/analytics", catalog_name),
                "full_name": format!("{}.analytics", catalog_name)
            }
        ],
        "next_page_token": null
    });

    let mock = server
        .mock_catalog_endpoint("GET", &format!("/schemas?catalog_name={}", catalog_name))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&expected_response).unwrap())
        .create();

    // Execute test
    let mut schema_stream = client.list_schemas(&catalog_name, None);
    let mut schemas = Vec::new();

    use futures::StreamExt;
    while let Some(schema_result) = schema_stream.next().await {
        assert!(schema_result.is_ok());
        schemas.push(schema_result.unwrap());
    }

    // Verify request was made
    mock.assert();

    // Verify response
    assert_eq!(schemas.len(), 2);
    assert_eq!(schemas[0].name, "default");
    assert_eq!(schemas[1].name, "analytics");
    assert_eq!(schemas[1].comment, Some("Analytics schema".to_string()));
}

/// Test getting a specific schema
#[rstest]
#[tokio::test]
async fn test_get_schema(
    #[future] test_client: (UnityCatalogClient, TestServer),
    catalog_name: String,
) {
    let (client, mut server) = test_client.await;
    let schema_name = "target_schema";

    let expected_response = json!({
        "name": schema_name,
        "catalog_name": catalog_name,
        "comment": "Schema retrieved by get operation",
        "properties": {
            "data_domain": "customer_analytics",
            "sla_tier": "gold"
        },
        "owner": "analytics-team",
        "created_at": 1699564800000i64,
        "updated_at": 1699564800000i64,
        "schema_type": "MANAGED_SCHEMA",
        "storage_root": format!("s3://bucket/catalogs/{}/schemas/{}", catalog_name, schema_name),
        "full_name": format!("{}.{}", catalog_name, schema_name)
    });

    let mock = server
        .mock_catalog_endpoint("GET", &format!("/schemas/{}.{}", catalog_name, schema_name))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&expected_response).unwrap())
        .create();

    // Execute test
    let schema = client.schema(&catalog_name, schema_name);
    let result = schema.get().await;

    // Verify request was made
    mock.assert();

    // Verify response
    assert!(result.is_ok());
    let schema_info = result.unwrap();
    assert_eq!(schema_info.name, schema_name);
    assert_eq!(schema_info.catalog_name, catalog_name);
    assert_eq!(
        schema_info.comment,
        Some("Schema retrieved by get operation".to_string())
    );
    assert!(schema_info.properties.contains_key("data_domain"));
}

/// Test schema not found error
#[rstest]
#[tokio::test]
async fn test_get_schema_not_found(#[future] test_client: (UnityCatalogClient, TestServer)) {
    let (client, mut server) = test_client.await;
    let catalog_name = "test_catalog";
    let schema_name = "nonexistent_schema";

    let error_response =
        ErrorResponses::not_found("Schema", &format!("{}.{}", catalog_name, schema_name));

    let mock = server
        .mock_catalog_endpoint("GET", &format!("/schemas/{}.{}", catalog_name, schema_name))
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&error_response).unwrap())
        .create();

    // Execute test
    let schema = client.schema(catalog_name, schema_name);
    let result = schema.get().await;

    // Verify request was made
    mock.assert();

    // Verify error response
    TestAssertions::assert_error_contains(&result, "not found");
}

/// Test updating a schema
#[rstest]
#[tokio::test]
async fn test_update_schema(
    #[future] test_client: (UnityCatalogClient, TestServer),
    catalog_name: String,
) {
    let (client, mut server) = test_client.await;
    let schema_name = "updatable_schema";

    let expected_response = json!({
        "name": schema_name,
        "catalog_name": catalog_name,
        "comment": "Updated schema comment",
        "properties": {
            "updated": "true",
            "version": "2.0"
        },
        "owner": "new-owner",
        "created_at": 1699564800000i64,
        "updated_at": 1699568400000i64, // Updated timestamp
        "schema_type": "MANAGED_SCHEMA",
        "storage_root": format!("s3://bucket/catalogs/{}/schemas/{}", catalog_name, schema_name),
        "full_name": format!("{}.{}", catalog_name, schema_name)
    });

    let mock = server
        .mock_catalog_endpoint(
            "PATCH",
            &format!("/schemas/{}.{}", catalog_name, schema_name),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&expected_response).unwrap())
        .create();

    // Execute test
    let schema = client.schema(&catalog_name, schema_name);
    let mut properties = HashMap::new();
    properties.insert("updated".to_string(), "true".to_string());
    properties.insert("version".to_string(), "2.0".to_string());

    let result = schema
        .update(
            Some("Updated schema comment"),
            Some("new-owner"),
            Some(properties),
        )
        .await;

    // Verify request was made
    mock.assert();

    // Verify response
    assert!(result.is_ok());
    let schema_info = result.unwrap();
    assert_eq!(
        schema_info.comment,
        Some("Updated schema comment".to_string())
    );
    assert_eq!(
        schema_info.properties.get("updated"),
        Some(&"true".to_string())
    );
}

/// Test deleting a schema
#[rstest]
#[tokio::test]
async fn test_delete_schema(
    #[future] test_client: (UnityCatalogClient, TestServer),
    catalog_name: String,
) {
    let (client, mut server) = test_client.await;
    let schema_name = "deletable_schema";

    let mock = server
        .mock_catalog_endpoint(
            "DELETE",
            &format!("/schemas/{}.{}", catalog_name, schema_name),
        )
        .with_status(200)
        .create();

    // Execute test
    let schema = client.schema(&catalog_name, schema_name);
    let result = schema.delete(None).await;

    // Verify request was made
    mock.assert();

    // Verify success
    assert!(result.is_ok());
}

/// Test concurrent schema operations
#[rstest]
#[tokio::test]
async fn test_concurrent_schema_operations(
    #[future] test_client: (UnityCatalogClient, TestServer),
    catalog_name: String,
) {
    let (client, mut server) = test_client.await;

    let schema_names = vec!["schema1", "schema2", "schema3"];
    let mut mocks = Vec::new();

    // Setup mocks for multiple schema get operations
    for schema_name in &schema_names {
        let expected_response = json!({
            "name": schema_name,
            "catalog_name": catalog_name,
            "comment": format!("Concurrent test schema {}", schema_name),
            "properties": {},
            "owner": "test-user",
            "created_at": 1699564800000i64,
            "updated_at": 1699564800000i64,
            "schema_type": "MANAGED_SCHEMA",
            "storage_root": format!("s3://bucket/catalogs/{}/schemas/{}", catalog_name, schema_name),
            "full_name": format!("{}.{}", catalog_name, schema_name)
        });

        let mock = server
            .mock_catalog_endpoint("GET", &format!("/schemas/{}.{}", catalog_name, schema_name))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&expected_response).unwrap())
            .create();

        mocks.push(mock);
    }

    // Execute concurrent operations
    let mut tasks = Vec::new();
    for schema_name in &schema_names {
        let schema = client.schema(&catalog_name, schema_name);
        let task = tokio::spawn(async move { schema.get().await });
        tasks.push(task);
    }

    // Wait for all tasks to complete
    let results = futures::future::join_all(tasks).await;

    // Verify all mocks were called
    for mock in mocks {
        mock.assert();
    }

    // Verify all operations succeeded
    for (result, expected_name) in results.iter().zip(&schema_names) {
        let schema_result = result.as_ref().unwrap();
        assert!(schema_result.is_ok());
        let schema_info = schema_result.as_ref().unwrap();
        assert_eq!(schema_info.name, *expected_name);
    }
}

// Additional test patterns that can be applied to other client types:
//
// 1. Property validation tests
// 2. Pagination tests for list operations
// 3. Error handling for various HTTP status codes
// 4. Unicode and special character handling
// 5. Large payload tests
// 6. Timeout and retry behavior tests
// 7. Authentication and authorization tests
//
// Each client type (tables, shares, volumes, etc.) can follow the same patterns:
// - Use rstest fixtures for test data setup
// - Create realistic mock responses using the TestServer
// - Use TestAssertions for consistent error checking
// - Load test data from JSON files when appropriate
// - Test both success and failure scenarios
// - Include concurrent operation testing where relevant
