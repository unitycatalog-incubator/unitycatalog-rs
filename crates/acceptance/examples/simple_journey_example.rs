//! Example demonstrating the simplified journey framework
//!
//! This example shows how to:
//! - Use the new trait-based journey system
//! - Execute journeys against Unity Catalog
//! - Record responses automatically
//! - Run multiple journeys in sequence
//!
//! To run this example:
//! ```bash
//! # Set up environment
//! export UC_SERVER_URL="http://localhost:8080"
//! export UC_AUTH_TOKEN="your-token"  # Optional
//! export RECORD_JOURNEY_RESPONSES="true"
//! export JOURNEY_RECORDING_DIR="./test_recordings"
//!
//! # Run the example
//! cargo run --example simple_journey_example
//! ```

use unitycatalog_acceptance::{
    AcceptanceResult,
    journeys::SimpleCatalogJourney,
    simple_journey::{JourneyConfig, UserJourney},
};

#[tokio::main]
async fn main() -> AcceptanceResult<()> {
    println!("🚀 Unity Catalog Simplified Journey Framework Example");
    println!("====================================================");

    // Load configuration from environment
    let config = JourneyConfig::default();
    println!("📋 Configuration:");
    println!("  Server URL: {}", config.server_url);
    println!("  Recording: {}", config.recording_enabled);
    println!("  Output Dir: {}", config.output_dir.display());
    println!();

    // Create executor
    let executor = config.create_executor()?;

    // Example 1: Run a single journey
    println!("📝 Example 1: Single Journey Execution");
    println!("--------------------------------------");

    let catalog_journey = SimpleCatalogJourney::new();
    println!("Executing journey: {}", catalog_journey.name());
    println!("Description: {}", catalog_journey.description());
    println!("Tags: {:?}", catalog_journey.tags());
    println!();

    let result = executor.execute_journey(&catalog_journey).await?;
    println!("Result: {}", result.summary());
    println!();

    // Example 2: Run multiple journeys
    println!("📝 Example 2: Multiple Journey Execution");
    println!("----------------------------------------");

    let journeys: Vec<Box<dyn UserJourney>> = vec![
        Box::new(SimpleCatalogJourney::new()),
        Box::new(SimpleCatalogJourney::with_catalog_name(
            "second_catalog_example",
        )),
        // Note: Other journeys may have compilation issues with current client API
        // Box::new(CatalogLifecycleJourney::new()),
        // Box::new(SchemaOperationsJourney::new()),
        // Box::new(TableOperationsJourney::new()),
    ];

    println!("Executing {} journeys...", journeys.len());

    let journey_refs: Vec<&dyn UserJourney> = journeys.iter().map(|j| j.as_ref()).collect();
    let results = executor.execute_journeys(journey_refs).await?;

    // Print summary
    println!("\n📊 Journey Execution Summary");
    println!("============================");
    let mut successful = 0;
    let mut failed = 0;

    for result in &results {
        println!("{}", result.summary());
        if result.is_success() {
            successful += 1;
        } else {
            failed += 1;
        }
    }

    println!();
    println!("✅ Successful: {}", successful);
    println!("❌ Failed: {}", failed);
    println!("📊 Total: {}", results.len());

    // Example 3: Custom journey
    println!("\n📝 Example 3: Custom Journey");
    println!("----------------------------");

    let custom_journey = CustomExampleJourney::new();
    let custom_result = executor.execute_journey(&custom_journey).await?;
    println!("Custom journey result: {}", custom_result.summary());

    if config.recording_enabled {
        println!(
            "\n📁 Recorded responses saved to: {}",
            config.output_dir.display()
        );
        println!("You can find numbered response files for each journey step.");
    }

    println!("\n🎉 Example completed successfully!");
    Ok(())
}

/// Custom journey example showing how to implement your own journey
struct CustomExampleJourney {
    catalog_name: String,
}

impl CustomExampleJourney {
    fn new() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            catalog_name: format!("custom_example_{}", timestamp),
        }
    }
}

#[async_trait::async_trait]
impl UserJourney for CustomExampleJourney {
    fn name(&self) -> &str {
        "custom_example"
    }

    fn description(&self) -> &str {
        "Custom journey demonstrating how to implement your own journey"
    }

    fn tags(&self) -> Vec<&str> {
        vec!["custom", "example", "smoke"]
    }

    async fn execute(
        &self,
        client: &unitycatalog_client::UnityCatalogClient,
        recorder: &mut unitycatalog_acceptance::simple_journey::JourneyRecorder,
    ) -> AcceptanceResult<()> {
        println!("🔧 Executing custom journey: {}", self.name());

        // Step 1: Create a catalog
        println!("  📁 Creating catalog '{}'", self.catalog_name);
        let catalog = client
            .create_catalog(&self.catalog_name)
            .with_comment(Some("Custom example catalog".to_string()))
            .await
            .map_err(|e| {
                unitycatalog_acceptance::AcceptanceError::UnityCatalog(format!(
                    "Failed to create catalog: {}",
                    e
                ))
            })?;

        recorder
            .record_step(
                "create_catalog",
                format!("Create example catalog '{}'", self.catalog_name),
                &catalog,
            )
            .await?;

        // Step 2: List catalogs to verify
        println!("  📋 Listing catalogs to verify creation");
        let mut catalogs = client.list_catalogs(Some(10));
        let mut catalog_names = Vec::new();

        use futures::StreamExt;
        while let Some(catalog_result) = catalogs.next().await {
            let catalog = catalog_result.map_err(|e| {
                unitycatalog_acceptance::AcceptanceError::UnityCatalog(format!(
                    "Failed to list catalogs: {}",
                    e
                ))
            })?;
            catalog_names.push(catalog.name.clone());
        }

        recorder
            .record_step(
                "list_catalogs",
                "List catalogs for verification",
                &serde_json::json!({
                    "catalog_names": catalog_names,
                    "found_our_catalog": catalog_names.contains(&self.catalog_name)
                }),
            )
            .await?;

        println!("  ✅ Custom journey steps completed");
        Ok(())
    }

    async fn cleanup(
        &self,
        client: &unitycatalog_client::UnityCatalogClient,
        recorder: &mut unitycatalog_acceptance::simple_journey::JourneyRecorder,
    ) -> AcceptanceResult<()> {
        println!("  🧹 Cleaning up custom journey");

        match client.catalog(&self.catalog_name).delete(Some(false)).await {
            Ok(()) => {
                recorder
                    .record_step(
                        "cleanup_delete_catalog",
                        format!("Cleanup: Delete catalog '{}'", self.catalog_name),
                        &serde_json::json!({
                            "deleted": true,
                            "catalog_name": self.catalog_name
                        }),
                    )
                    .await?;
                println!("  🗑️ Successfully deleted catalog '{}'", self.catalog_name);
            }
            Err(e) => {
                eprintln!(
                    "  ⚠️ Failed to delete catalog '{}': {}",
                    self.catalog_name, e
                );
                recorder
                    .record_step(
                        "cleanup_delete_catalog_failed",
                        format!("Cleanup: Failed to delete catalog '{}'", self.catalog_name),
                        &serde_json::json!({
                            "deleted": false,
                            "catalog_name": self.catalog_name,
                            "error": e.to_string()
                        }),
                    )
                    .await?;
            }
        }

        Ok(())
    }
}
