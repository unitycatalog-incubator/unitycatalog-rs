use futures::TryStreamExt;
use std::collections::HashMap;
use std::time::Instant;
use unitycatalog_client::UnityCatalogClient;

use crate::output::OutputFormatter;
use crate::{GlobalOpts, server::init_tracing};

pub(crate) async fn run(opts: &GlobalOpts) -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    // Show welcome banner
    OutputFormatter::banner("Unity Catalog Integration Tests");

    let client = cloud_client::CloudClient::new_unauthenticated();
    let base_url = format!("{}/api/2.1/unity-catalog/", opts.server);
    let client = UnityCatalogClient::new(client, url::Url::parse(&base_url)?);

    OutputFormatter::info(&format!("Testing against server: {}", opts.server));
    OutputFormatter::separator();

    let start_time = Instant::now();
    let mut test_results = TestResults::new();

    // Run catalog tests
    let catalog_tests = CatalogTests::new(client);
    catalog_tests.run_all(&mut test_results).await?;

    // Print final results
    let duration = start_time.elapsed();
    OutputFormatter::separator();
    OutputFormatter::section_header("Test Execution Complete");
    OutputFormatter::info(&format!(
        "Total execution time: {:.2}s",
        duration.as_secs_f64()
    ));

    test_results.print_summary();

    if test_results.has_failures() {
        std::process::exit(1);
    }

    Ok(())
}

#[derive(Default)]
struct TestResults {
    passed: usize,
    failed: usize,
    categories: Vec<(String, bool)>,
}

impl TestResults {
    fn new() -> Self {
        Self::default()
    }

    fn add_result(&mut self, category: String, success: bool) {
        if success {
            self.passed += 1;
        } else {
            self.failed += 1;
        }
        self.categories.push((category, success));
    }

    fn total(&self) -> usize {
        self.passed + self.failed
    }

    fn has_failures(&self) -> bool {
        self.failed > 0
    }

    fn print_summary(&self) {
        OutputFormatter::test_summary(self.passed, self.failed, self.total());

        if !self.categories.is_empty() {
            OutputFormatter::subsection_header("Category Breakdown");
            for (category, success) in &self.categories {
                if *success {
                    OutputFormatter::success(&category.to_string());
                } else {
                    OutputFormatter::error(&category.to_string());
                }
            }
        }
    }
}

struct CatalogTests {
    client: UnityCatalogClient,
}

impl CatalogTests {
    fn new(client: UnityCatalogClient) -> Self {
        Self { client }
    }

    async fn run_all(&self, results: &mut TestResults) -> Result<(), Box<dyn std::error::Error>> {
        OutputFormatter::section_header("Catalog Client Tests");

        let test_categories = [
            ("Catalog Lifecycle", "test_catalog_lifecycle"),
            ("Catalog List Operations", "test_catalog_list_operations"),
            (
                "Catalog Update Operations",
                "test_catalog_update_operations",
            ),
            ("Catalog Properties", "test_catalog_properties"),
            ("Catalog Sharing", "test_catalog_sharing"),
            ("Error Handling", "test_catalog_error_handling"),
            ("Schema Integration", "test_catalog_with_schemas"),
        ];

        for (category_name, test_method) in test_categories {
            OutputFormatter::test_category_start(category_name);

            let category_start = Instant::now();
            let result = match test_method {
                "test_catalog_lifecycle" => self.test_catalog_lifecycle().await,
                "test_catalog_list_operations" => self.test_catalog_list_operations().await,
                "test_catalog_update_operations" => self.test_catalog_update_operations().await,
                "test_catalog_properties" => self.test_catalog_properties().await,
                "test_catalog_sharing" => self.test_catalog_sharing().await,
                "test_catalog_error_handling" => self.test_catalog_error_handling().await,
                "test_catalog_with_schemas" => self.test_catalog_with_schemas().await,
                _ => unreachable!(),
            };
            let duration = category_start.elapsed();

            let success = result.is_ok();

            if let Err(e) = result {
                OutputFormatter::error(&format!("Test failed: {}", e));
            }

            OutputFormatter::info(&format!(
                "Category completed in {:.2}s",
                duration.as_secs_f64()
            ));
            OutputFormatter::test_category_end(category_name, success);

            results.add_result(category_name.to_string(), success);
        }

        Ok(())
    }

    /// Test basic catalog lifecycle: create, get, delete
    async fn test_catalog_lifecycle(&self) -> Result<(), Box<dyn std::error::Error>> {
        let catalog_name = "test_lifecycle_catalog";

        // Create catalog
        let spinner =
            OutputFormatter::operation_start("Creating catalog with storage root and comment");
        let created_catalog = self
            .client
            .create_catalog(
                catalog_name,
                Some("s3://test-bucket/catalog-root"),
                Some("Test catalog for lifecycle testing"),
                None::<HashMap<String, String>>,
            )
            .await?;
        OutputFormatter::operation_success(&spinner, "Catalog created successfully");

        // Validate creation
        OutputFormatter::step("Validating catalog properties");
        OutputFormatter::validation_result(
            "Name",
            catalog_name,
            &created_catalog.name,
            created_catalog.name == catalog_name,
        );
        OutputFormatter::validation_result(
            "Comment",
            "Test catalog for lifecycle testing",
            created_catalog.comment.as_deref().unwrap_or(""),
            created_catalog.comment.as_deref() == Some("Test catalog for lifecycle testing"),
        );
        OutputFormatter::validation_result(
            "Storage Root",
            "s3://test-bucket/catalog-root",
            created_catalog.storage_root.as_deref().unwrap_or(""),
            created_catalog.storage_root.as_deref() == Some("s3://test-bucket/catalog-root"),
        );

        OutputFormatter::key_value_pairs(&[
            (
                "Catalog ID",
                created_catalog.id.as_deref().unwrap_or("unknown"),
            ),
            (
                "Created At",
                created_catalog
                    .created_at
                    .map(|t| t.to_string())
                    .as_deref()
                    .unwrap_or("unknown"),
            ),
        ]);

        // Get catalog
        let spinner = OutputFormatter::operation_start("Retrieving catalog by name");
        let retrieved_catalog = self.client.catalog(catalog_name).get().await?;
        OutputFormatter::operation_success(&spinner, "Catalog retrieved successfully");

        OutputFormatter::validation_result(
            "Retrieved Name",
            &created_catalog.name,
            &retrieved_catalog.name,
            retrieved_catalog.name == created_catalog.name,
        );
        OutputFormatter::validation_result(
            "Retrieved ID",
            created_catalog.id.as_deref().unwrap_or(""),
            retrieved_catalog.id.as_deref().unwrap_or(""),
            retrieved_catalog.id == created_catalog.id,
        );

        // Delete catalog
        let spinner = OutputFormatter::operation_start("Deleting catalog");
        self.client.catalog(catalog_name).delete(None).await?;
        OutputFormatter::operation_success(&spinner, "Catalog deleted successfully");

        // Verify deletion
        let spinner = OutputFormatter::operation_start("Verifying catalog deletion");
        let get_result = self.client.catalog(catalog_name).get().await;
        if get_result.is_err() {
            OutputFormatter::operation_success(
                &spinner,
                "Catalog deletion verified (get failed as expected)",
            );
        } else {
            OutputFormatter::operation_failed(&spinner, "Catalog still exists after deletion");
            return Err("Catalog deletion verification failed".into());
        }

        Ok(())
    }

    /// Test catalog listing functionality
    async fn test_catalog_list_operations(&self) -> Result<(), Box<dyn std::error::Error>> {
        let catalog_names = vec![
            "list_test_catalog_1",
            "list_test_catalog_2",
            "list_test_catalog_3",
        ];

        OutputFormatter::step(&format!("Creating {} test catalogs", catalog_names.len()));

        // Create multiple test catalogs with progress
        for name in &catalog_names {
            let spinner = OutputFormatter::operation_start(&format!("Creating catalog: {}", name));
            self.client
                .create_catalog(
                    name,
                    None::<String>,
                    Some(format!("Test catalog {}", name)),
                    None::<HashMap<String, String>>,
                )
                .await?;
            OutputFormatter::operation_success(&spinner, &format!("Created {}", name));
        }

        // Test listing all catalogs
        let spinner = OutputFormatter::operation_start("Listing all catalogs");
        let all_catalogs: Vec<_> = self.client.list_catalogs(None).try_collect().await?;
        OutputFormatter::operation_success(
            &spinner,
            &format!("Retrieved {} catalogs", all_catalogs.len()),
        );

        // Display catalog table
        OutputFormatter::step("Displaying catalog information");
        OutputFormatter::catalog_table(&all_catalogs);

        // Verify our test catalogs are in the list
        OutputFormatter::step("Verifying test catalogs in results");
        for name in &catalog_names {
            let found = all_catalogs.iter().any(|c| c.name == *name);
            OutputFormatter::validation_result(
                &format!("Catalog {}", name),
                "present",
                if found { "present" } else { "missing" },
                found,
            );
            if !found {
                return Err(format!("Catalog {} not found in list", name).into());
            }
        }

        // Test pagination
        let spinner = OutputFormatter::operation_start("Testing pagination (max_results=2)");
        let paginated_catalogs: Vec<_> = self.client.list_catalogs(Some(2)).try_collect().await?;
        OutputFormatter::operation_success(
            &spinner,
            &format!(
                "Retrieved {} catalogs with pagination",
                paginated_catalogs.len()
            ),
        );

        // Clean up
        OutputFormatter::step("Cleaning up test catalogs");
        for name in &catalog_names {
            let spinner = OutputFormatter::operation_start(&format!("Deleting {}", name));
            self.client.catalog(name).delete(None).await?;
            OutputFormatter::operation_success(&spinner, &format!("Deleted {}", name));
        }

        Ok(())
    }

    /// Test catalog update operations
    async fn test_catalog_update_operations(&self) -> Result<(), Box<dyn std::error::Error>> {
        let catalog_name = "test_update_catalog";

        // Create initial catalog
        let spinner = OutputFormatter::operation_start("Creating catalog for update testing");
        self.client
            .create_catalog(
                catalog_name,
                Some("s3://initial-bucket/root"),
                Some("Initial comment"),
                None::<HashMap<String, String>>,
            )
            .await?;
        OutputFormatter::operation_success(&spinner, "Initial catalog created");

        // Test comment update
        let spinner = OutputFormatter::operation_start("Updating catalog comment");
        let updated_catalog = self
            .client
            .catalog(catalog_name)
            .update(
                None::<String>,
                Some("Updated comment for testing"),
                None::<String>,
                None::<HashMap<String, String>>,
            )
            .await?;
        OutputFormatter::operation_success(&spinner, "Comment updated successfully");

        OutputFormatter::validation_result(
            "Updated Comment",
            "Updated comment for testing",
            updated_catalog.comment.as_deref().unwrap_or(""),
            updated_catalog.comment.as_deref() == Some("Updated comment for testing"),
        );

        // Test properties update
        let spinner = OutputFormatter::operation_start("Updating catalog properties");
        let mut properties = HashMap::new();
        properties.insert("test_key".to_string(), "test_value".to_string());
        properties.insert("environment".to_string(), "integration_test".to_string());

        let updated_catalog_props = self
            .client
            .catalog(catalog_name)
            .update(
                None::<String>,
                None::<String>,
                None::<String>,
                Some(properties.clone()),
            )
            .await?;
        OutputFormatter::operation_success(&spinner, "Properties updated successfully");

        OutputFormatter::validation_result(
            "Property test_key",
            "test_value",
            updated_catalog_props
                .properties
                .get("test_key")
                .unwrap_or(&"".to_string()),
            updated_catalog_props.properties.get("test_key") == Some(&"test_value".to_string()),
        );

        // Test owner update
        let spinner = OutputFormatter::operation_start("Updating catalog owner");
        let updated_catalog_owner = self
            .client
            .catalog(catalog_name)
            .update(
                None::<String>,
                None::<String>,
                Some("test_owner@example.com"),
                None::<HashMap<String, String>>,
            )
            .await?;
        OutputFormatter::operation_success(&spinner, "Owner updated successfully");

        OutputFormatter::validation_result(
            "Owner",
            "test_owner@example.com",
            updated_catalog_owner.owner.as_deref().unwrap_or(""),
            updated_catalog_owner.owner.as_deref() == Some("test_owner@example.com"),
        );

        // Clean up
        let spinner = OutputFormatter::operation_start("Cleaning up test catalog");
        self.client.catalog(catalog_name).delete(None).await?;
        OutputFormatter::operation_success(&spinner, "Test catalog cleaned up");

        Ok(())
    }

    /// Test catalog with various properties configurations
    async fn test_catalog_properties(&self) -> Result<(), Box<dyn std::error::Error>> {
        let catalog_name = "test_properties_catalog";

        // Create catalog with initial properties
        let mut initial_properties = HashMap::new();
        initial_properties.insert("created_by".to_string(), "integration_test".to_string());
        initial_properties.insert("version".to_string(), "1.0".to_string());
        initial_properties.insert("environment".to_string(), "test".to_string());

        let spinner = OutputFormatter::operation_start("Creating catalog with initial properties");
        let catalog = self
            .client
            .create_catalog(
                catalog_name,
                None::<String>,
                Some("Catalog for properties testing"),
                Some(initial_properties.clone()),
            )
            .await?;
        OutputFormatter::operation_success(&spinner, "Catalog created with properties");

        // Verify initial properties
        OutputFormatter::step("Validating initial properties");
        for (key, expected_value) in &initial_properties {
            let empty_string = "".to_string();
            let actual_value = catalog.properties.get(key).unwrap_or(&empty_string);
            OutputFormatter::validation_result(
                &format!("Property {}", key),
                expected_value,
                actual_value,
                actual_value == expected_value,
            );
        }

        // Test property updates
        let spinner = OutputFormatter::operation_start("Updating and adding properties");
        let mut updated_properties = initial_properties.clone();
        updated_properties.insert("updated_at".to_string(), "2024-01-01".to_string());
        updated_properties.insert("version".to_string(), "1.1".to_string());

        let updated_catalog = self
            .client
            .catalog(catalog_name)
            .update(
                None::<String>,
                None::<String>,
                None::<String>,
                Some(updated_properties.clone()),
            )
            .await?;
        OutputFormatter::operation_success(&spinner, "Properties updated successfully");

        // Verify updated properties
        OutputFormatter::step("Validating updated properties");
        for (key, expected_value) in &updated_properties {
            let empty_string = "".to_string();
            let actual_value = updated_catalog.properties.get(key).unwrap_or(&empty_string);
            OutputFormatter::validation_result(
                &format!("Updated Property {}", key),
                expected_value,
                actual_value,
                actual_value == expected_value,
            );
        }

        // Test clearing properties
        let spinner = OutputFormatter::operation_start("Clearing all properties");
        let cleared_catalog = self
            .client
            .catalog(catalog_name)
            .update(
                None::<String>,
                None::<String>,
                None::<String>,
                Some(HashMap::new()),
            )
            .await?;
        OutputFormatter::operation_success(&spinner, "Properties cleared successfully");

        OutputFormatter::validation_result(
            "Properties Count",
            "0",
            &cleared_catalog.properties.len().to_string(),
            cleared_catalog.properties.is_empty(),
        );

        // Clean up
        let spinner = OutputFormatter::operation_start("Cleaning up properties test catalog");
        self.client.catalog(catalog_name).delete(None).await?;
        OutputFormatter::operation_success(&spinner, "Properties test catalog cleaned up");

        Ok(())
    }

    /// Test sharing catalog creation
    async fn test_catalog_sharing(&self) -> Result<(), Box<dyn std::error::Error>> {
        let catalog_name = "test_sharing_catalog";
        let provider_name = "test_provider";
        let share_name = "test_share";

        let spinner = OutputFormatter::operation_start(&format!(
            "Creating sharing catalog (provider: {}, share: {})",
            provider_name, share_name
        ));

        let sharing_catalog = self
            .client
            .create_sharing_catalog(
                catalog_name,
                provider_name,
                share_name,
                Some("Catalog for sharing testing"),
                None::<HashMap<String, String>>,
            )
            .await?;
        OutputFormatter::operation_success(&spinner, "Sharing catalog created successfully");

        // Validate sharing catalog properties
        OutputFormatter::step("Validating sharing catalog properties");
        OutputFormatter::validation_result(
            "Catalog Name",
            catalog_name,
            &sharing_catalog.name,
            sharing_catalog.name == catalog_name,
        );
        OutputFormatter::validation_result(
            "Share Name",
            share_name,
            sharing_catalog.share_name.as_deref().unwrap_or(""),
            sharing_catalog.share_name.as_deref() == Some(share_name),
        );
        OutputFormatter::validation_result(
            "Provider Name",
            provider_name,
            sharing_catalog.provider_name.as_deref().unwrap_or(""),
            sharing_catalog.provider_name.as_deref() == Some(provider_name),
        );

        // Verify retrieval
        let spinner = OutputFormatter::operation_start("Retrieving sharing catalog");
        let retrieved_sharing_catalog = self.client.catalog(catalog_name).get().await?;
        OutputFormatter::operation_success(&spinner, "Sharing catalog retrieved successfully");

        OutputFormatter::validation_result(
            "Retrieved Share Name",
            share_name,
            retrieved_sharing_catalog
                .share_name
                .as_deref()
                .unwrap_or(""),
            retrieved_sharing_catalog.share_name.as_deref() == Some(share_name),
        );

        // Clean up
        let spinner = OutputFormatter::operation_start("Cleaning up sharing test catalog");
        self.client.catalog(catalog_name).delete(None).await?;
        OutputFormatter::operation_success(&spinner, "Sharing test catalog cleaned up");

        Ok(())
    }

    /// Test error handling scenarios
    async fn test_catalog_error_handling(&self) -> Result<(), Box<dyn std::error::Error>> {
        let catalog_name = "duplicate_test_catalog";

        // Create first catalog
        let spinner = OutputFormatter::operation_start("Creating first catalog");
        self.client
            .create_catalog(
                catalog_name,
                None::<String>,
                Some("First catalog"),
                None::<HashMap<String, String>>,
            )
            .await?;
        OutputFormatter::operation_success(&spinner, "First catalog created");

        // Test duplicate creation
        let spinner = OutputFormatter::operation_start("Attempting to create duplicate catalog");
        let duplicate_result = self
            .client
            .create_catalog(
                catalog_name,
                None::<String>,
                Some("Duplicate catalog"),
                None::<HashMap<String, String>>,
            )
            .await;

        if duplicate_result.is_err() {
            OutputFormatter::operation_success(&spinner, "Duplicate creation properly rejected");
        } else {
            OutputFormatter::operation_failed(&spinner, "Duplicate creation should have failed");
            return Err("Expected error when creating duplicate catalog".into());
        }

        // Test getting non-existent catalog
        let spinner = OutputFormatter::operation_start("Attempting to get non-existent catalog");
        let nonexistent_result = self.client.catalog("nonexistent_catalog_12345").get().await;
        if nonexistent_result.is_err() {
            OutputFormatter::operation_success(&spinner, "Non-existent catalog properly rejected");
        } else {
            OutputFormatter::operation_failed(
                &spinner,
                "Non-existent catalog get should have failed",
            );
            return Err("Expected error when getting non-existent catalog".into());
        }

        // Test updating non-existent catalog
        let spinner = OutputFormatter::operation_start("Attempting to update non-existent catalog");
        let update_nonexistent_result = self
            .client
            .catalog("nonexistent_catalog_12345")
            .update(
                None::<String>,
                Some("Should fail"),
                None::<String>,
                None::<HashMap<String, String>>,
            )
            .await;
        if update_nonexistent_result.is_err() {
            OutputFormatter::operation_success(
                &spinner,
                "Non-existent catalog update properly rejected",
            );
        } else {
            OutputFormatter::operation_failed(
                &spinner,
                "Non-existent catalog update should have failed",
            );
        }

        // Test catalog with schemas
        OutputFormatter::step("Creating schema in catalog for delete testing");
        let spinner = OutputFormatter::operation_start("Creating test schema");
        self.client
            .create_schema(catalog_name, "test_schema", Some("Test schema"))
            .await?;
        OutputFormatter::operation_success(&spinner, "Test schema created");

        let spinner = OutputFormatter::operation_start("Testing delete without force");
        let delete_result = self.client.catalog(catalog_name).delete(Some(false)).await;
        if delete_result.is_ok() {
            OutputFormatter::operation_success(&spinner, "Delete without force succeeded");
        } else {
            OutputFormatter::operation_success(&spinner, "Delete without force failed as expected");
        }

        // Clean up with force
        let spinner = OutputFormatter::operation_start("Cleaning up with force delete");
        self.client.catalog(catalog_name).delete(Some(true)).await?;
        OutputFormatter::operation_success(&spinner, "Force delete successful");

        Ok(())
    }

    /// Test catalog operations with schemas
    async fn test_catalog_with_schemas(&self) -> Result<(), Box<dyn std::error::Error>> {
        let catalog_name = "catalog_with_schemas_test";

        // Create catalog
        let spinner = OutputFormatter::operation_start("Creating catalog for schema testing");
        let catalog = self
            .client
            .create_catalog(
                catalog_name,
                Some("s3://test-bucket/schemas-test"),
                Some("Catalog for schema testing"),
                None::<HashMap<String, String>>,
            )
            .await?;
        OutputFormatter::operation_success(&spinner, "Test catalog created");

        OutputFormatter::key_value_pairs(&[
            ("Catalog Name", &catalog.name),
            ("Catalog ID", catalog.id.as_deref().unwrap_or("unknown")),
        ]);

        // Create multiple schemas
        let schema_names = vec!["schema_1", "schema_2", "schema_3"];

        OutputFormatter::step(&format!("Creating {} schemas", schema_names.len()));
        for schema_name in &schema_names {
            let spinner =
                OutputFormatter::operation_start(&format!("Creating schema: {}", schema_name));
            let schema = self
                .client
                .create_schema(
                    catalog_name,
                    schema_name,
                    Some(format!("Test schema {}", schema_name)),
                )
                .await?;
            OutputFormatter::operation_success(
                &spinner,
                &format!("Schema {} created", schema_name),
            );

            OutputFormatter::validation_result(
                "Schema Name",
                schema_name,
                &schema.name,
                schema.name == *schema_name,
            );
            OutputFormatter::validation_result(
                "Catalog Name",
                catalog_name,
                &schema.catalog_name,
                schema.catalog_name == catalog_name,
            );
        }

        // List schemas in catalog
        let spinner = OutputFormatter::operation_start("Listing schemas in catalog");
        let schemas: Vec<_> = self
            .client
            .list_schemas(catalog_name, None)
            .try_collect()
            .await?;
        OutputFormatter::operation_success(
            &spinner,
            &format!("Retrieved {} schemas", schemas.len()),
        );

        // Display schema table
        OutputFormatter::step("Displaying schema information");
        OutputFormatter::schema_table(&schemas);

        // Verify all schemas are present
        OutputFormatter::step("Verifying schemas in catalog");
        for schema_name in &schema_names {
            let found = schemas.iter().any(|s| s.name == *schema_name);
            OutputFormatter::validation_result(
                &format!("Schema {}", schema_name),
                "present",
                if found { "present" } else { "missing" },
                found,
            );
        }

        // Test schema operations through catalog
        OutputFormatter::step("Testing schema operations through catalog client");
        let schema_client = self.client.catalog(catalog_name).schema("schema_1");

        let spinner = OutputFormatter::operation_start("Getting schema through catalog client");
        let _schema = schema_client.get().await?;
        OutputFormatter::operation_success(&spinner, "Schema retrieved successfully");

        let spinner = OutputFormatter::operation_start("Updating schema through catalog client");
        let updated_schema = schema_client
            .update(
                None::<String>,
                Some("Updated schema comment"),
                None::<HashMap<String, String>>,
            )
            .await?;
        OutputFormatter::operation_success(&spinner, "Schema updated successfully");

        OutputFormatter::validation_result(
            "Updated Comment",
            "Updated schema comment",
            updated_schema.comment.as_deref().unwrap_or(""),
            updated_schema.comment.as_deref() == Some("Updated schema comment"),
        );

        // Clean up schemas first
        OutputFormatter::step("Cleaning up schemas");
        for schema_name in &schema_names {
            let spinner =
                OutputFormatter::operation_start(&format!("Deleting schema: {}", schema_name));
            self.client
                .schema(catalog_name, schema_name)
                .delete(None)
                .await?;
            OutputFormatter::operation_success(
                &spinner,
                &format!("Schema {} deleted", schema_name),
            );
        }

        // Clean up catalog
        let spinner = OutputFormatter::operation_start("Cleaning up test catalog");
        self.client.catalog(catalog_name).delete(None).await?;
        OutputFormatter::operation_success(&spinner, "Test catalog cleaned up");

        Ok(())
    }
}
