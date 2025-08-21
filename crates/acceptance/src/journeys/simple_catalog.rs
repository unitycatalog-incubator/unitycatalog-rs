//! Simple Catalog Example Journey
//!
//! A minimal, working example of the simplified journey framework that demonstrates
//! basic catalog operations without complex API interactions.

use async_trait::async_trait;
use futures::StreamExt;
use unitycatalog_client::UnityCatalogClient;

use crate::AcceptanceResult;
use crate::simple_journey::{JourneyRecorder, UserJourney};

/// A simple catalog journey that creates, reads, and deletes a catalog
pub struct SimpleCatalogJourney {
    catalog_name: String,
    storage_root: String,
}

impl SimpleCatalogJourney {
    /// Create a new simple catalog journey
    pub fn new() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            catalog_name: format!("simple_catalog_{}", timestamp),
            storage_root: "s3://open-lakehouse-dev/".to_string(),
        }
    }

    /// Create with a specific catalog name
    pub fn with_catalog_name(catalog_name: impl Into<String>) -> Self {
        Self {
            catalog_name: catalog_name.into(),
            storage_root: "s3://open-lakehouse-dev/".to_string(),
        }
    }

    /// Get the catalog name for this journey
    pub fn catalog_name(&self) -> &str {
        &self.catalog_name
    }
}

#[async_trait]
impl UserJourney for SimpleCatalogJourney {
    fn name(&self) -> &str {
        "simple_catalog_example"
    }

    fn description(&self) -> &str {
        "Simple catalog lifecycle: create, list, delete"
    }

    fn tags(&self) -> Vec<&str> {
        vec!["catalog", "simple", "example", "smoke"]
    }

    async fn execute(
        &self,
        client: &UnityCatalogClient,
        recorder: &mut JourneyRecorder,
    ) -> AcceptanceResult<()> {
        println!(
            "🚀 Starting simple catalog journey for '{}'",
            self.catalog_name
        );

        // Step 1: Create catalog with minimal configuration
        println!("📁 Creating catalog '{}'", self.catalog_name);
        let created_catalog = client
            .create_catalog(&self.catalog_name)
            .with_storage_root(self.storage_root.clone())
            .with_comment(Some("Simple test catalog".to_string()))
            .await
            .map_err(|e| {
                crate::AcceptanceError::UnityCatalog(format!("Failed to create catalog: {}", e))
            })?;

        recorder
            .record_step(
                "create_catalog",
                format!("Create simple catalog '{}'", self.catalog_name),
                &created_catalog,
            )
            .await?;

        println!("✅ Created catalog: {}", created_catalog.name);

        // Step 2: List catalogs to verify our catalog exists
        println!("📋 Listing catalogs to verify creation");
        let mut catalogs = client.list_catalogs(Some(10)); // Limit to 10 for demo
        let mut catalog_list = Vec::new();
        let mut found_our_catalog = false;

        while let Some(catalog_result) = catalogs.next().await {
            let catalog = catalog_result.map_err(|e| {
                crate::AcceptanceError::UnityCatalog(format!("Failed to list catalogs: {}", e))
            })?;

            if catalog.name == self.catalog_name {
                found_our_catalog = true;
                println!("✅ Found our catalog in the list: {}", catalog.name);
            }

            catalog_list.push(catalog);
        }

        recorder
            .record_step(
                "list_catalogs",
                "List catalogs for verification",
                &serde_json::json!({
                    "total_catalogs": catalog_list.len(),
                    "found_our_catalog": found_our_catalog,
                    "our_catalog_name": self.catalog_name,
                    "catalog_names": catalog_list.iter().map(|c| &c.name).collect::<Vec<_>>()
                }),
            )
            .await?;

        // Verify our catalog is in the list
        if !found_our_catalog {
            return Err(crate::AcceptanceError::JourneyExecution(format!(
                "Created catalog '{}' was not found in the catalog list",
                self.catalog_name
            )));
        }

        // Step 3: Get specific catalog information
        println!("🔍 Getting detailed catalog information");
        let catalog_info = client
            .catalog(&self.catalog_name)
            .get()
            .await
            .map_err(|e| {
                crate::AcceptanceError::UnityCatalog(format!("Failed to get catalog info: {}", e))
            })?;

        recorder
            .record_step(
                "get_catalog_info",
                format!("Get detailed info for catalog '{}'", self.catalog_name),
                &catalog_info,
            )
            .await?;

        // Verify catalog properties
        assert_eq!(catalog_info.name, self.catalog_name);
        println!("✅ Verified catalog properties");

        println!("🎉 Simple catalog journey completed successfully!");
        Ok(())
    }

    async fn cleanup(
        &self,
        client: &UnityCatalogClient,
        recorder: &mut JourneyRecorder,
    ) -> AcceptanceResult<()> {
        println!("🧹 Cleaning up simple catalog journey");

        // Delete the catalog
        match client.catalog(&self.catalog_name).delete(Some(true)).await {
            Ok(()) => {
                recorder
                    .record_step(
                        "cleanup_delete_catalog",
                        format!("Cleanup: Delete catalog '{}'", self.catalog_name),
                        &serde_json::json!({
                            "deleted": true,
                            "catalog_name": self.catalog_name,
                            "force": true
                        }),
                    )
                    .await?;
                println!("🗑️ Successfully deleted catalog '{}'", self.catalog_name);
            }
            Err(e) => {
                // Log the error but don't fail cleanup
                eprintln!("⚠️ Failed to delete catalog '{}': {}", self.catalog_name, e);
                recorder
                    .record_step(
                        "cleanup_delete_catalog_failed",
                        format!("Cleanup: Failed to delete catalog '{}'", self.catalog_name),
                        &serde_json::json!({
                            "deleted": false,
                            "catalog_name": self.catalog_name,
                            "error": e.to_string(),
                            "error_type": "cleanup_error"
                        }),
                    )
                    .await?;
            }
        }

        Ok(())
    }
}

impl Default for SimpleCatalogJourney {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_catalog_journey_properties() {
        let journey = SimpleCatalogJourney::new();

        assert_eq!(journey.name(), "simple_catalog_example");
        assert!(!journey.description().is_empty());
        assert!(journey.tags().contains(&"catalog"));
        assert!(journey.tags().contains(&"simple"));
        assert!(journey.tags().contains(&"example"));
        assert!(journey.tags().contains(&"smoke"));
    }

    #[test]
    fn test_custom_catalog_name() {
        let journey = SimpleCatalogJourney::with_catalog_name("my_custom_catalog");
        assert_eq!(journey.catalog_name, "my_custom_catalog");
    }

    #[test]
    fn test_default_journey() {
        let journey = SimpleCatalogJourney::default();
        assert!(journey.catalog_name.starts_with("simple_catalog_"));
    }
}
