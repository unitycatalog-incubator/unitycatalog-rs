//! Comprehensive tests for the Unity Catalog client implementation
//!
//! This module provides extensive test coverage for the catalog client,
//! including CRUD operations, error handling, and edge cases.

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

/// Test catalog creation with basic parameters
#[rstest]
#[tokio::test]
async fn test_create_catalog_basic(
    #[future] test_client: (UnityCatalogClient, TestServer),
    catalog_name: String,
) {
    let (client, mut server) = test_client.await;

    // Setup mock response
    let expected_response = CatalogResponses::catalog_info(&catalog_name, Some("Test catalog"));
    let mock = server
        .mock_catalog_endpoint("POST", "/catalogs")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&expected_response).unwrap())
        .create();

    // Execute test
    let catalog = client.catalog(&catalog_name);
    let result = catalog
        .create(
            Some("s3://test-bucket/catalogs/"),
            Some("Test catalog"),
            None::<HashMap<String, String>>,
        )
        .await;

    // Verify request was made
    mock.assert();

    // Verify response
    assert!(result.is_ok());
    let catalog_info = result.unwrap();
    TestAssertions::assert_catalog_info_matches(&catalog_info, &catalog_name, Some("Test catalog"));
}

/// Test catalog creation with properties
#[rstest]
#[tokio::test]
async fn test_create_catalog_with_properties(
    #[future] test_client: (UnityCatalogClient, TestServer),
    catalog_name: String,
    catalog_properties: HashMap<String, String>,
) {
    let (client, mut server) = test_client.await;

    let expected_response =
        CatalogResponses::catalog_with_properties(&catalog_name, catalog_properties.clone());
    let mock = server
        .mock_catalog_endpoint("POST", "/catalogs")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&expected_response).unwrap())
        .create();

    let catalog = client.catalog(&catalog_name);
    let result = catalog
        .create(
            Some("s3://test-bucket/catalogs/"),
            Some("Test catalog with properties"),
            Some(catalog_properties.clone()),
        )
        .await;

    mock.assert();
    assert!(result.is_ok());

    let catalog_info = result.unwrap();
    assert_eq!(catalog_info.name, catalog_name);
    assert_eq!(catalog_info.properties, catalog_properties);
}

/// Test catalog creation with sharing
#[rstest]
#[tokio::test]
async fn test_create_sharing_catalog(
    #[future] test_client: (UnityCatalogClient, TestServer),
    catalog_name: String,
    sharing_params: (String, String),
) {
    let (client, mut server) = test_client.await;
    let (provider_name, share_name) = sharing_params;

    let expected_response =
        CatalogResponses::sharing_catalog(&catalog_name, &provider_name, &share_name);
    let mock = server
        .mock_catalog_endpoint("POST", "/catalogs")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&expected_response).unwrap())
        .create();

    let catalog = client.catalog(&catalog_name);
    let result = catalog
        .create_sharing(
            provider_name.clone(),
            share_name.clone(),
            Some("Shared catalog"),
            None::<HashMap<String, String>>,
        )
        .await;

    mock.assert();
    assert!(result.is_ok());

    let catalog_info = result.unwrap();
    assert_eq!(catalog_info.name, catalog_name);
    assert_eq!(catalog_info.provider_name, Some(provider_name));
    assert_eq!(catalog_info.share_name, Some(share_name));
}

/// Test catalog creation failure - already exists
#[rstest]
#[tokio::test]
async fn test_create_catalog_already_exists(
    #[future] test_client: (UnityCatalogClient, TestServer),
    catalog_name: String,
) {
    let (client, mut server) = test_client.await;

    let error_response = ErrorResponses::already_exists("Catalog", &catalog_name);
    let mock = server
        .mock_catalog_endpoint("POST", "/catalogs")
        .with_status(409)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&error_response).unwrap())
        .create();

    let catalog = client.catalog(&catalog_name);
    let result = catalog
        .create(
            Some("s3://test-bucket/"),
            Some("Test catalog"),
            None::<HashMap<String, String>>,
        )
        .await;

    mock.assert();
    TestAssertions::assert_error_contains(&result, "already exists");
}

/// Test getting a catalog
#[rstest]
#[tokio::test]
async fn test_get_catalog(
    #[future] test_client: (UnityCatalogClient, TestServer),
    catalog_name: String,
) {
    let (client, mut server) = test_client.await;

    let expected_response = TestDataLoader::load_response("catalogs", "get_catalog.json")
        .expect("Failed to load test data");

    let mock = server
        .mock_catalog_endpoint("GET", &format!("/catalogs/{}", catalog_name))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&expected_response).unwrap())
        .create();

    let catalog = client.catalog(&catalog_name);
    let result = catalog.get().await;

    mock.assert();
    assert!(result.is_ok());

    let catalog_info = result.unwrap();
    assert_eq!(catalog_info.name, "test_catalog");
    assert_eq!(
        catalog_info.comment,
        Some("A test catalog for unit testing".to_string())
    );
}

/// Test getting a non-existent catalog
#[rstest]
#[tokio::test]
async fn test_get_catalog_not_found(#[future] test_client: (UnityCatalogClient, TestServer)) {
    let (client, mut server) = test_client.await;
    let catalog_name = "nonexistent_catalog";

    let error_response = TestDataLoader::load_response("catalogs", "catalog_not_found.json")
        .expect("Failed to load test data");

    let mock = server
        .mock_catalog_endpoint("GET", &format!("/catalogs/{}", catalog_name))
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&error_response).unwrap())
        .create();

    let catalog = client.catalog(catalog_name);
    let result = catalog.get().await;

    mock.assert();
    TestAssertions::assert_error_contains(&result, "not found");
}

/// Test updating a catalog with various parameter combinations
#[rstest]
#[tokio::test]
async fn test_update_catalog(
    #[future] test_client: (UnityCatalogClient, TestServer),
    catalog_name: String,
    update_params: (Option<String>, Option<String>, Option<String>),
    catalog_properties: HashMap<String, String>,
) {
    let (client, mut server) = test_client.await;
    let (new_name, comment, owner) = update_params;

    let updated_name = new_name.as_deref().unwrap_or(&catalog_name);
    let expected_response =
        CatalogResponses::update_catalog(&catalog_name, updated_name, comment.as_deref());

    let mock = server
        .mock_catalog_endpoint("PATCH", &format!("/catalogs/{}", catalog_name))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&expected_response).unwrap())
        .create();

    let catalog = client.catalog(&catalog_name);
    let result = catalog
        .update(
            new_name.clone(),
            comment.clone(),
            owner.clone(),
            Some(catalog_properties.clone()),
        )
        .await;

    mock.assert();
    assert!(result.is_ok());

    let catalog_info = result.unwrap();
    assert_eq!(catalog_info.name, updated_name);
    if let Some(ref expected_comment) = comment {
        assert_eq!(catalog_info.comment, Some(expected_comment.clone()));
    }
}

/// Test deleting a catalog
#[rstest]
#[tokio::test]
async fn test_delete_catalog(
    #[future] test_client: (UnityCatalogClient, TestServer),
    catalog_name: String,
    force_delete_variations: Option<bool>,
) {
    let (client, mut server) = test_client.await;

    let mut mock = server.mock_catalog_endpoint("DELETE", &format!("/catalogs/{}", catalog_name));

    // Match query parameter if force is Some(true)
    if let Some(true) = force_delete_variations {
        mock = mock.match_query(mockito::Matcher::UrlEncoded("force".into(), "true".into()));
    }

    let mock = mock.with_status(200).create();

    let catalog = client.catalog(&catalog_name);
    let result = catalog.delete(force_delete_variations).await;

    mock.assert();
    assert!(result.is_ok());
}

/// Test deleting a non-existent catalog
#[rstest]
#[tokio::test]
async fn test_delete_catalog_not_found(
    #[future] test_client: (UnityCatalogClient, TestServer),
    catalog_name: String,
) {
    let (client, mut server) = test_client.await;

    let error_response = ErrorResponses::not_found("Catalog", &catalog_name);
    let mock = server
        .mock_catalog_endpoint("DELETE", &format!("/catalogs/{}", catalog_name))
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&error_response).unwrap())
        .create();

    let catalog = client.catalog(&catalog_name);
    let result = catalog.delete(None).await;

    mock.assert();
    TestAssertions::assert_error_contains(&result, "not found");
}

/// Test listing catalogs
#[rstest]
#[tokio::test]
async fn test_list_catalogs(#[future] test_client: (UnityCatalogClient, TestServer)) {
    let (client, mut server) = test_client.await;

    let expected_response = TestDataLoader::load_response("catalogs", "list_catalogs.json")
        .expect("Failed to load test data");

    let mock = server
        .mock_catalog_endpoint("GET", "/catalogs")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&expected_response).unwrap())
        .create();

    let mut catalog_stream = client.list_catalogs(None);
    let mut catalogs = Vec::new();

    // Collect all results from the stream
    use futures::StreamExt;
    while let Some(catalog_result) = catalog_stream.next().await {
        assert!(catalog_result.is_ok());
        catalogs.push(catalog_result.unwrap());
    }

    mock.assert();
    assert_eq!(catalogs.len(), 3);
    assert_eq!(catalogs[0].name, "production_catalog");
    assert_eq!(catalogs[1].name, "staging_catalog");
    assert_eq!(catalogs[2].name, "dev_catalog");
}

/// Test listing catalogs with empty result
#[rstest]
#[tokio::test]
async fn test_list_catalogs_empty(#[future] test_client: (UnityCatalogClient, TestServer)) {
    let (client, mut server) = test_client.await;

    let expected_response = CatalogResponses::empty_list();
    let mock = server
        .mock_catalog_endpoint("GET", "/catalogs")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&expected_response).unwrap())
        .create();

    let mut catalog_stream = client.list_catalogs(None);
    let mut catalogs = Vec::new();

    use futures::StreamExt;
    while let Some(catalog_result) = catalog_stream.next().await {
        assert!(catalog_result.is_ok());
        catalogs.push(catalog_result.unwrap());
    }

    mock.assert();
    assert_eq!(catalogs.len(), 0);
}

/// Test creating a schema in a catalog
#[rstest]
#[tokio::test]
async fn test_create_schema_in_catalog(
    #[future] test_client: (UnityCatalogClient, TestServer),
    catalog_name: String,
    schema_params: (String, Option<String>),
) {
    let (client, mut server) = test_client.await;
    let (schema_name, schema_comment) = schema_params;

    let expected_response = json!({
        "name": schema_name,
        "catalog_name": catalog_name,
        "comment": schema_comment,
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

    let catalog = client.catalog(&catalog_name);
    let result = catalog
        .create_schema(&schema_name, schema_comment.as_deref())
        .await;

    mock.assert();
    assert!(result.is_ok());

    let schema_info = result.unwrap();
    assert_eq!(schema_info.name, schema_name);
    assert_eq!(schema_info.catalog_name, catalog_name);
    assert_eq!(schema_info.comment, schema_comment);
}

/// Test error handling for various HTTP status codes
#[rstest]
#[tokio::test]
async fn test_error_handling(
    #[future] test_client: (UnityCatalogClient, TestServer),
    catalog_name: String,
) {
    let (client, mut server) = test_client.await;

    // Test 404 error
    let error_response = ErrorResponses::not_found("Catalog", &catalog_name);
    let mock = server
        .mock_catalog_endpoint("GET", &format!("/catalogs/{}", catalog_name))
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&error_response).unwrap())
        .create();

    let catalog = client.catalog(&catalog_name);
    let result = catalog.get().await;

    assert!(result.is_err());
    mock.assert();
    TestAssertions::assert_error_contains(&result, "not found");
}

/// Test concurrent operations on different catalogs
#[rstest]
#[tokio::test]
async fn test_concurrent_catalog_operations(
    #[future] test_client: (UnityCatalogClient, TestServer),
    multi_catalog_scenario: Vec<(String, Option<String>, HashMap<String, String>)>,
) {
    let (client, mut server) = test_client.await;

    // Setup mocks for multiple catalog operations
    let mut mocks = Vec::new();
    for (catalog_name, _comment, properties) in &multi_catalog_scenario {
        let expected_response =
            CatalogResponses::catalog_with_properties(catalog_name, properties.clone());

        let mock = server
            .mock_catalog_endpoint("GET", &format!("/catalogs/{}", catalog_name))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&expected_response).unwrap())
            .create();

        mocks.push(mock);
    }

    // Execute concurrent operations
    let mut tasks = Vec::new();
    for (catalog_name, _, _) in &multi_catalog_scenario {
        let catalog = client.catalog(catalog_name);
        let task = tokio::spawn(async move { catalog.get().await });
        tasks.push(task);
    }

    // Wait for all tasks to complete
    let results = futures::future::join_all(tasks).await;

    // Verify all mocks were called
    for mock in mocks {
        mock.assert();
    }

    // Verify all operations succeeded
    for (result, (expected_name, _, _)) in results.iter().zip(&multi_catalog_scenario) {
        let catalog_result = result.as_ref().unwrap();
        assert!(catalog_result.is_ok());
        let catalog_info = catalog_result.as_ref().unwrap();
        assert_eq!(catalog_info.name, *expected_name);
    }
}

/// Test pagination in catalog listing
#[rstest]
#[tokio::test]
async fn test_catalog_list_pagination(#[future] test_client: (UnityCatalogClient, TestServer)) {
    let (client, mut server) = test_client.await;

    // Setup first page mock
    let first_page_response = CatalogResponses::first_page();
    let first_mock = server
        .mock_catalog_endpoint("GET", "/catalogs")
        .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
            "max_results".into(),
            "2".into(),
        )]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&first_page_response).unwrap())
        .create();

    // Setup second page mock
    let second_page_response = CatalogResponses::second_page();
    let second_mock = server
        .mock_catalog_endpoint("GET", "/catalogs")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("max_results".into(), "2".into()),
            mockito::Matcher::UrlEncoded("page_token".into(), "next_page_token_123".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&second_page_response).unwrap())
        .create();

    // Test pagination by collecting all results
    let mut catalog_stream = client.list_catalogs(Some(2));
    let mut all_catalogs = Vec::new();

    use futures::StreamExt;
    while let Some(catalog_result) = catalog_stream.next().await {
        assert!(catalog_result.is_ok());
        all_catalogs.push(catalog_result.unwrap());
    }

    // Verify both pages were called
    first_mock.assert();
    second_mock.assert();

    // Verify we got all catalogs from both pages
    assert_eq!(all_catalogs.len(), 4);
    assert_eq!(all_catalogs[0].name, "catalog1");
    assert_eq!(all_catalogs[1].name, "catalog2");
    assert_eq!(all_catalogs[2].name, "catalog3");
    assert_eq!(all_catalogs[3].name, "catalog4");
}
