use std::time::Duration;
use tokio::time::sleep;

use crate::output::OutputFormatter;

pub(crate) async fn run_demo() -> Result<(), Box<dyn std::error::Error>> {
    // Show welcome banner
    OutputFormatter::banner("Unity Catalog Integration Tests - Demo Mode");

    OutputFormatter::info("This is a demonstration of professional CLI output formatting");
    OutputFormatter::separator();

    // Demo test execution
    let start_time = std::time::Instant::now();

    OutputFormatter::section_header("Catalog Client Tests");

    // Demo Category 1: Lifecycle Tests
    OutputFormatter::test_category_start("Catalog Lifecycle");

    let spinner =
        OutputFormatter::operation_start("Creating catalog with storage root and comment");
    sleep(Duration::from_millis(800)).await;
    OutputFormatter::operation_success(&spinner, "Catalog created successfully");

    OutputFormatter::step("Validating catalog properties");
    OutputFormatter::validation_result("Name", "test_catalog", "test_catalog", true);
    OutputFormatter::validation_result("Comment", "Test comment", "Test comment", true);
    OutputFormatter::validation_result(
        "Storage Root",
        "s3://bucket/root",
        "s3://bucket/root",
        true,
    );

    OutputFormatter::key_value_pairs(&[
        ("Catalog ID", "cat_12345678"),
        ("Created At", "2024-01-15T10:30:00Z"),
    ]);

    let spinner = OutputFormatter::operation_start("Retrieving catalog by name");
    sleep(Duration::from_millis(500)).await;
    OutputFormatter::operation_success(&spinner, "Catalog retrieved successfully");

    let spinner = OutputFormatter::operation_start("Deleting catalog");
    sleep(Duration::from_millis(600)).await;
    OutputFormatter::operation_success(&spinner, "Catalog deleted successfully");

    OutputFormatter::info("Category completed in 2.1s");
    OutputFormatter::test_category_end("Catalog Lifecycle", true);

    // Demo Category 2: List Operations
    OutputFormatter::test_category_start("Catalog List Operations");

    OutputFormatter::step("Creating 3 test catalogs");
    for i in 1..=3 {
        let spinner =
            OutputFormatter::operation_start(&format!("Creating catalog: list_test_catalog_{}", i));
        sleep(Duration::from_millis(400)).await;
        OutputFormatter::operation_success(&spinner, &format!("Created list_test_catalog_{}", i));
    }

    let spinner = OutputFormatter::operation_start("Listing all catalogs");
    sleep(Duration::from_millis(700)).await;
    OutputFormatter::operation_success(&spinner, "Retrieved 5 catalogs");

    // Demo catalog table
    OutputFormatter::step("Displaying catalog information");
    let demo_catalogs = vec![
        create_demo_catalog("list_test_catalog_1", "cat_001", "First test catalog"),
        create_demo_catalog("list_test_catalog_2", "cat_002", "Second test catalog"),
        create_demo_catalog("list_test_catalog_3", "cat_003", "Third test catalog"),
    ];
    OutputFormatter::catalog_table(&demo_catalogs);

    OutputFormatter::step("Verifying test catalogs in results");
    for i in 1..=3 {
        OutputFormatter::validation_result(
            &format!("Catalog list_test_catalog_{}", i),
            "present",
            "present",
            true,
        );
    }

    OutputFormatter::step("Cleaning up test catalogs");
    for i in 1..=3 {
        let spinner =
            OutputFormatter::operation_start(&format!("Deleting list_test_catalog_{}", i));
        sleep(Duration::from_millis(300)).await;
        OutputFormatter::operation_success(&spinner, &format!("Deleted list_test_catalog_{}", i));
    }

    OutputFormatter::info("Category completed in 3.4s");
    OutputFormatter::test_category_end("Catalog List Operations", true);

    // Demo Category 3: Error Handling
    OutputFormatter::test_category_start("Error Handling");

    let spinner = OutputFormatter::operation_start("Creating first catalog");
    sleep(Duration::from_millis(500)).await;
    OutputFormatter::operation_success(&spinner, "First catalog created");

    let spinner = OutputFormatter::operation_start("Attempting to create duplicate catalog");
    sleep(Duration::from_millis(400)).await;
    OutputFormatter::operation_success(&spinner, "Duplicate creation properly rejected");

    let spinner = OutputFormatter::operation_start("Attempting to get non-existent catalog");
    sleep(Duration::from_millis(300)).await;
    OutputFormatter::operation_success(&spinner, "Non-existent catalog properly rejected");

    // Demo a failure case
    let spinner = OutputFormatter::operation_start("Testing edge case scenario");
    sleep(Duration::from_millis(600)).await;
    OutputFormatter::operation_failed(&spinner, "Edge case validation failed as expected");

    OutputFormatter::info("Category completed in 1.8s");
    OutputFormatter::test_category_end("Error Handling", false);

    // Demo Category 4: Schema Integration
    OutputFormatter::test_category_start("Schema Integration");

    let spinner = OutputFormatter::operation_start("Creating catalog for schema testing");
    sleep(Duration::from_millis(500)).await;
    OutputFormatter::operation_success(&spinner, "Test catalog created");

    OutputFormatter::key_value_pairs(&[
        ("Catalog Name", "catalog_with_schemas"),
        ("Catalog ID", "cat_schemas_001"),
    ]);

    OutputFormatter::step("Creating 3 schemas");
    for i in 1..=3 {
        let spinner = OutputFormatter::operation_start(&format!("Creating schema: schema_{}", i));
        sleep(Duration::from_millis(350)).await;
        OutputFormatter::operation_success(&spinner, &format!("Schema {} created", i));
    }

    let spinner = OutputFormatter::operation_start("Listing schemas in catalog");
    sleep(Duration::from_millis(400)).await;
    OutputFormatter::operation_success(&spinner, "Retrieved 3 schemas");

    // Demo schema table
    OutputFormatter::step("Displaying schema information");
    let demo_schemas = vec![
        create_demo_schema(
            "schema_1",
            "catalog_with_schemas.schema_1",
            "catalog_with_schemas",
            "First test schema",
        ),
        create_demo_schema(
            "schema_2",
            "catalog_with_schemas.schema_2",
            "catalog_with_schemas",
            "Second test schema",
        ),
        create_demo_schema(
            "schema_3",
            "catalog_with_schemas.schema_3",
            "catalog_with_schemas",
            "Third test schema",
        ),
    ];
    OutputFormatter::schema_table(&demo_schemas);

    OutputFormatter::step("Cleaning up schemas");
    for i in 1..=3 {
        let spinner = OutputFormatter::operation_start(&format!("Deleting schema: schema_{}", i));
        sleep(Duration::from_millis(250)).await;
        OutputFormatter::operation_success(&spinner, &format!("Schema {} deleted", i));
    }

    let spinner = OutputFormatter::operation_start("Cleaning up test catalog");
    sleep(Duration::from_millis(400)).await;
    OutputFormatter::operation_success(&spinner, "Test catalog cleaned up");

    OutputFormatter::info("Category completed in 2.9s");
    OutputFormatter::test_category_end("Schema Integration", true);

    // Print final results
    let duration = start_time.elapsed();
    OutputFormatter::separator();
    OutputFormatter::section_header("Test Execution Complete");
    OutputFormatter::info(&format!(
        "Total execution time: {:.2}s",
        duration.as_secs_f64()
    ));

    // Demo test summary with mixed results
    OutputFormatter::test_summary(3, 1, 4);

    // Category breakdown
    OutputFormatter::subsection_header("Category Breakdown");
    OutputFormatter::success("Catalog Lifecycle");
    OutputFormatter::success("Catalog List Operations");
    OutputFormatter::error("Error Handling");
    OutputFormatter::success("Schema Integration");

    OutputFormatter::info(
        "Demo completed successfully! This showcases the professional output formatting.",
    );

    Ok(())
}

fn create_demo_catalog(name: &str, id: &str, comment: &str) -> unitycatalog_common::CatalogInfo {
    use std::collections::HashMap;

    unitycatalog_common::CatalogInfo {
        name: name.to_string(),
        id: Some(id.to_string()),
        comment: Some(comment.to_string()),
        storage_root: Some("s3://demo-bucket/catalog-root".to_string()),
        properties: {
            let mut props = HashMap::new();
            props.insert("environment".to_string(), "demo".to_string());
            props.insert("created_by".to_string(), "integration_test".to_string());
            props
        },
        owner: Some("demo@example.com".to_string()),
        created_at: Some(1705320600), // 2024-01-15 10:30:00 UTC
        updated_at: Some(1705320600),
        provider_name: None,
        share_name: None,
        catalog_type: None,
        created_by: Some("demo@example.com".to_string()),
        updated_by: Some("demo@example.com".to_string()),
        browse_only: Some(false),
    }
}

fn create_demo_schema(
    name: &str,
    full_name: &str,
    catalog_name: &str,
    comment: &str,
) -> unitycatalog_common::SchemaInfo {
    use std::collections::HashMap;

    unitycatalog_common::SchemaInfo {
        name: name.to_string(),
        catalog_name: catalog_name.to_string(),
        comment: Some(comment.to_string()),
        properties: HashMap::new(),
        full_name: Some(full_name.to_string()),
        owner: Some("demo@example.com".to_string()),
        created_at: Some(1705320600), // 2024-01-15 10:30:00 UTC
        updated_at: Some(1705320600),
        created_by: Some("demo@example.com".to_string()),
        updated_by: Some("demo@example.com".to_string()),
        schema_id: Some(format!("schema_{}_{}", catalog_name, name)),
    }
}
