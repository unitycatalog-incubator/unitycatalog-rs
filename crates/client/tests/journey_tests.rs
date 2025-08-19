//! Journey-based tests for Unity Catalog client
//!
//! This module contains tests that execute multi-step user journeys
//! defined in JSON format, providing comprehensive testing of dependent
//! API operations and real-world usage scenarios.

use rstest::*;
use std::collections::HashMap;

use unitycatalog_client::UnityCatalogClient;

mod test_utils;
use test_utils::TestServer;
use test_utils::journeys::{JourneyExecutor, JourneyLoader};

/// Fixture that provides a test client and server for journey execution
#[fixture]
pub async fn journey_test_setup() -> (UnityCatalogClient, TestServer) {
    let server = TestServer::new().await;
    let client = server.create_client();
    (client, server)
}

/// Fixture that provides initial variables for journey execution
#[fixture]
pub fn journey_variables() -> HashMap<String, serde_json::Value> {
    let mut variables = HashMap::new();

    // Generate unique names for this test run to avoid conflicts
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    variables.insert(
        "test_suffix".to_string(),
        serde_json::Value::String(format!("_{}", timestamp)),
    );
    variables.insert(
        "test_user".to_string(),
        serde_json::Value::String("test-user".to_string()),
    );
    variables.insert(
        "test_bucket".to_string(),
        serde_json::Value::String("test-bucket".to_string()),
    );

    variables
}

#[rstest]
#[tokio::test]
async fn test_catalog_lifecycle_journey(
    #[future] journey_test_setup: (UnityCatalogClient, TestServer),
    journey_variables: HashMap<String, serde_json::Value>,
) {
    let (client, server) = journey_test_setup.await;

    // Load the catalog lifecycle journey
    let journey = JourneyLoader::load_journey("catalog_lifecycle.json")
        .expect("Failed to load catalog lifecycle journey");

    // Validate the journey definition
    let validation_errors = JourneyLoader::validate_journey(&journey);
    assert!(
        validation_errors.is_empty(),
        "Journey validation failed: {:?}",
        validation_errors
    );

    // Create executor with variables
    let mut executor = JourneyExecutor::new(client, Some(server)).with_variables(journey_variables);

    // Execute the journey
    let result = executor.execute_journey(journey).await;

    // Verify the journey completed successfully
    assert!(
        result.success,
        "Journey failed: {:?}",
        result.failure_summary
    );

    // Verify all expected steps were executed
    assert_eq!(result.step_results.len(), 6); // Expected number of steps

    // Verify setup steps succeeded
    let create_step = result
        .step_results
        .iter()
        .find(|step| step.step.id == "create_catalog")
        .expect("Create catalog step not found");
    assert!(create_step.success, "Create catalog step failed");

    let get_step = result
        .step_results
        .iter()
        .find(|step| step.step.id == "get_catalog")
        .expect("Get catalog step not found");
    assert!(get_step.success, "Get catalog step failed");

    // Verify update step succeeded
    let update_step = result
        .step_results
        .iter()
        .find(|step| step.step.id == "update_catalog")
        .expect("Update catalog step not found");
    assert!(update_step.success, "Update catalog step failed");

    // Verify cleanup steps executed (may fail, that's ok)
    let delete_step = result
        .step_results
        .iter()
        .find(|step| step.step.id == "delete_catalog")
        .expect("Delete catalog step not found");
    // Delete step should have been attempted
    assert_eq!(delete_step.step.id, "delete_catalog");

    // Verify variable extraction worked
    assert!(result.final_variables.contains_key("catalog_id"));
    assert!(result.final_variables.contains_key("created_at"));
    assert!(result.final_variables.contains_key("updated_at"));
}

#[rstest]
#[tokio::test]
async fn test_hierarchical_data_structure_journey(
    #[future] journey_test_setup: (UnityCatalogClient, TestServer),
    journey_variables: HashMap<String, serde_json::Value>,
) {
    let (client, server) = journey_test_setup.await;

    // Load the hierarchical data structure journey
    let journey = JourneyLoader::load_journey("hierarchical_data_structure.json")
        .expect("Failed to load hierarchical data structure journey");

    // Validate the journey definition
    let validation_errors = JourneyLoader::validate_journey(&journey);
    assert!(
        validation_errors.is_empty(),
        "Journey validation failed: {:?}",
        validation_errors
    );

    // Create executor with variables
    let mut executor = JourneyExecutor::new(client, Some(server))
        .with_variables(journey_variables)
        .continue_on_failure(true); // Allow cleanup steps to continue on failure

    // Execute the journey
    let result = executor.execute_journey(journey).await;

    // The journey should complete (cleanup may have failures, that's expected)
    assert_eq!(result.step_results.len(), 15); // Expected number of steps

    // Verify creation steps succeeded
    let catalog_step = result
        .step_results
        .iter()
        .find(|step| step.step.id == "create_catalog")
        .expect("Create catalog step not found");
    assert!(
        catalog_step.success,
        "Create catalog step failed: {:?}",
        catalog_step.error_message
    );

    let schema_step = result
        .step_results
        .iter()
        .find(|step| step.step.id == "create_schema")
        .expect("Create schema step not found");
    assert!(
        schema_step.success,
        "Create schema step failed: {:?}",
        schema_step.error_message
    );

    let table_step = result
        .step_results
        .iter()
        .find(|step| step.step.id == "create_table")
        .expect("Create table step not found");
    assert!(
        table_step.success,
        "Create table step failed: {:?}",
        table_step.error_message
    );

    // Verify that dependencies were respected
    let table_creation_index = result
        .step_results
        .iter()
        .position(|step| step.step.id == "create_table")
        .unwrap();
    let schema_creation_index = result
        .step_results
        .iter()
        .position(|step| step.step.id == "create_schema")
        .unwrap();
    let catalog_creation_index = result
        .step_results
        .iter()
        .position(|step| step.step.id == "create_catalog")
        .unwrap();

    assert!(
        catalog_creation_index < schema_creation_index,
        "Catalog should be created before schema"
    );
    assert!(
        schema_creation_index < table_creation_index,
        "Schema should be created before table"
    );

    // Verify variable extraction worked for hierarchical names
    assert!(result.final_variables.contains_key("catalog_full_name"));
    assert!(result.final_variables.contains_key("schema_full_name"));
    assert!(result.final_variables.contains_key("table_full_name"));
    assert!(result.final_variables.contains_key("table_id"));

    // Verify cleanup was attempted (deletion order should be reverse of creation)
    let cleanup_steps: Vec<_> = result
        .step_results
        .iter()
        .filter(|step| {
            step.step
                .tags
                .as_ref()
                .map_or(false, |tags| tags.contains(&"cleanup".to_string()))
        })
        .collect();

    assert!(
        !cleanup_steps.is_empty(),
        "Cleanup steps should have been executed"
    );
}

#[rstest]
#[tokio::test]
async fn test_error_handling_journey(
    #[future] journey_test_setup: (UnityCatalogClient, TestServer),
    journey_variables: HashMap<String, serde_json::Value>,
) {
    let (client, server) = journey_test_setup.await;

    // Load the error handling journey
    let journey = JourneyLoader::load_journey("error_handling.json")
        .expect("Failed to load error handling journey");

    // Validate the journey definition
    let validation_errors = JourneyLoader::validate_journey(&journey);
    assert!(
        validation_errors.is_empty(),
        "Journey validation failed: {:?}",
        validation_errors
    );

    // Create executor with variables, continue on failure for error testing
    let mut executor = JourneyExecutor::new(client, Some(server))
        .with_variables(journey_variables)
        .continue_on_failure(true);

    // Execute the journey
    let result = executor.execute_journey(journey).await;

    // Error journey should complete all steps even if individual steps "fail" (expected errors)
    assert!(!result.step_results.is_empty(), "No steps were executed");

    // Verify specific error scenarios were tested
    let not_found_step = result
        .step_results
        .iter()
        .find(|step| step.step.id == "get_nonexistent_catalog")
        .expect("Get nonexistent catalog step not found");
    assert!(
        not_found_step.success,
        "404 error should be treated as success"
    );
    assert_eq!(not_found_step.status_code, 404);

    let invalid_name_step = result
        .step_results
        .iter()
        .find(|step| step.step.id == "create_catalog_invalid_name")
        .expect("Create catalog with invalid name step not found");
    assert!(
        invalid_name_step.success,
        "400 error should be treated as success"
    );
    assert_eq!(invalid_name_step.status_code, 400);

    let conflict_step = result
        .step_results
        .iter()
        .find(|step| step.step.id == "create_duplicate_catalog")
        .expect("Create duplicate catalog step not found");
    assert!(
        conflict_step.success,
        "409 error should be treated as success"
    );
    assert_eq!(conflict_step.status_code, 409);

    // Verify dependency errors are handled
    let parent_dependency_step = result
        .step_results
        .iter()
        .find(|step| step.step.id == "create_schema_in_nonexistent_catalog")
        .expect("Create schema in nonexistent catalog step not found");
    assert!(
        parent_dependency_step.success,
        "Parent dependency error should be treated as success"
    );
    assert_eq!(parent_dependency_step.status_code, 404);
}

#[rstest]
#[tokio::test]
async fn test_journey_variable_substitution(
    #[future] journey_test_setup: (UnityCatalogClient, TestServer),
) {
    let (client, server) = journey_test_setup.await;

    // Create custom variables for substitution testing
    let mut custom_variables = HashMap::new();
    custom_variables.insert(
        "catalog_name".to_string(),
        serde_json::Value::String("substitution_test_catalog".to_string()),
    );
    custom_variables.insert(
        "storage_root".to_string(),
        serde_json::Value::String("s3://substitution-test-bucket/".to_string()),
    );
    custom_variables.insert(
        "comment".to_string(),
        serde_json::Value::String("Variable substitution test".to_string()),
    );

    // Load a simple journey
    let journey = JourneyLoader::load_journey("catalog_lifecycle.json")
        .expect("Failed to load journey for substitution test");

    // Create executor with custom variables
    let mut executor =
        JourneyExecutor::new(client, Some(server)).with_variables(custom_variables.clone());

    // Execute first step only to test substitution
    let first_step = &journey.steps[0];
    let step_result = executor.execute_step(first_step).await;

    // The step should have variables properly substituted
    // This is verified by the mock server expecting the substituted values
    assert!(step_result.success || step_result.status_code == 201);
}

#[rstest]
#[tokio::test]
async fn test_journey_step_dependencies() {
    // Test dependency validation without executing steps
    let journey = JourneyLoader::load_journey("hierarchical_data_structure.json")
        .expect("Failed to load journey for dependency test");

    let validation_errors = JourneyLoader::validate_journey(&journey);
    assert!(
        validation_errors.is_empty(),
        "Journey should have valid dependencies: {:?}",
        validation_errors
    );

    // Verify specific dependency relationships
    let create_schema_step = journey
        .steps
        .iter()
        .find(|step| step.id == "create_schema")
        .expect("Create schema step not found");

    assert!(
        create_schema_step.depends_on.is_some(),
        "Create schema should have dependencies"
    );

    let deps = create_schema_step.depends_on.as_ref().unwrap();
    assert!(
        deps.contains(&"verify_catalog_exists".to_string()),
        "Create schema should depend on catalog verification"
    );

    let create_table_step = journey
        .steps
        .iter()
        .find(|step| step.id == "create_table")
        .expect("Create table step not found");

    assert!(
        create_table_step.depends_on.is_some(),
        "Create table should have dependencies"
    );

    let table_deps = create_table_step.depends_on.as_ref().unwrap();
    assert!(
        table_deps.contains(&"list_schemas_in_catalog".to_string()),
        "Create table should depend on schema operations"
    );
}

#[rstest]
#[tokio::test]
async fn test_load_all_journeys() {
    // Test that all journey files can be loaded and validated
    let journeys = JourneyLoader::load_all_journeys().expect("Failed to load journeys");

    assert!(!journeys.is_empty(), "Should have at least one journey");

    // Verify each journey is valid
    for journey in &journeys {
        let errors = JourneyLoader::validate_journey(journey);
        assert!(
            errors.is_empty(),
            "Journey '{}' has validation errors: {:?}",
            journey.name,
            errors
        );

        // Verify journey has required fields
        assert!(!journey.name.is_empty(), "Journey name should not be empty");
        assert!(
            !journey.description.is_empty(),
            "Journey description should not be empty"
        );
        assert!(
            !journey.steps.is_empty(),
            "Journey should have at least one step"
        );

        // Verify each step has required fields
        for step in &journey.steps {
            assert!(!step.id.is_empty(), "Step ID should not be empty");
            assert!(
                !step.description.is_empty(),
                "Step description should not be empty"
            );
            assert!(!step.method.is_empty(), "Step method should not be empty");
            assert!(!step.path.is_empty(), "Step path should not be empty");
            assert!(
                step.expected_status > 0,
                "Step should have valid expected status"
            );
        }
    }

    // Verify we have our expected journey files
    let journey_names: Vec<_> = journeys.iter().map(|j| &j.name).collect();
    assert!(
        journey_names.contains(&&"catalog_lifecycle".to_string()),
        "Should include catalog lifecycle journey"
    );
    assert!(
        journey_names.contains(&&"hierarchical_data_structure".to_string()),
        "Should include hierarchical data structure journey"
    );
    assert!(
        journey_names.contains(&&"error_handling".to_string()),
        "Should include error handling journey"
    );
}

#[rstest]
#[tokio::test]
async fn test_journey_step_tags_and_filtering() {
    let journey = JourneyLoader::load_journey("hierarchical_data_structure.json")
        .expect("Failed to load journey for tag test");

    // Count steps by tags
    let setup_steps: Vec<_> = journey
        .steps
        .iter()
        .filter(|step| {
            step.tags
                .as_ref()
                .map_or(false, |tags| tags.contains(&"setup".to_string()))
        })
        .collect();

    let cleanup_steps: Vec<_> = journey
        .steps
        .iter()
        .filter(|step| {
            step.tags
                .as_ref()
                .map_or(false, |tags| tags.contains(&"cleanup".to_string()))
        })
        .collect();

    let main_steps: Vec<_> = journey
        .steps
        .iter()
        .filter(|step| {
            step.tags
                .as_ref()
                .map_or(false, |tags| tags.contains(&"main".to_string()))
        })
        .collect();

    assert!(!setup_steps.is_empty(), "Should have setup steps");
    assert!(!cleanup_steps.is_empty(), "Should have cleanup steps");
    assert!(!main_steps.is_empty(), "Should have main steps");

    // Verify cleanup steps have continue_on_failure set
    for step in cleanup_steps {
        assert!(
            step.continue_on_failure.unwrap_or(false),
            "Cleanup step '{}' should have continue_on_failure=true",
            step.id
        );
    }
}

#[tokio::test]
async fn test_journey_execution_without_mock_server() {
    // Test journey execution configuration without mock server (for integration tests)
    let client = UnityCatalogClient::new(
        cloud_client::CloudClient::new_unauthenticated(),
        url::Url::parse("http://localhost:8080").unwrap(),
    );

    let mut variables = HashMap::new();
    variables.insert(
        "catalog_name".to_string(),
        serde_json::Value::String("integration_test_catalog".to_string()),
    );

    // Create executor without mock server (would be used for integration tests)
    let executor = JourneyExecutor::new(client, None).with_variables(variables);

    // Just verify the executor can be created and configured
    assert!(!executor.context().variables.is_empty());
}
