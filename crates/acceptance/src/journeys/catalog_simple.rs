//! Simple Catalog Example Journey
//!
//! A minimal, working example of the simplified journey framework that demonstrates
//! basic catalog operations without complex API interactions.

use async_trait::async_trait;
use futures::StreamExt;
use unitycatalog_client::UnityCatalogClient;

use crate::AcceptanceResult;
use crate::journey::{JourneyState, UserJourney};

/// A simple catalog journey that creates, reads, and deletes a catalog
pub struct SimpleCatalogJourney {
    catalog_name: String,
    storage_root: String,
}

impl SimpleCatalogJourney {
    /// Create a new simple catalog journey
    pub fn new() -> Self {
        // Always use timestamp-based name initially; state management will handle replay
        let timestamp = chrono::Utc::now().timestamp();
        let catalog_name = format!("simple_catalog_{}", timestamp);

        Self {
            catalog_name,
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
        "simple_catalog"
    }

    fn description(&self) -> &str {
        "Simple catalog lifecycle: create, list, delete"
    }

    fn tags(&self) -> Vec<&str> {
        vec!["catalog", "simple", "example", "smoke"]
    }

    fn save_state(&self) -> AcceptanceResult<JourneyState> {
        let mut state = JourneyState::empty();
        state.set_string("catalog_name", self.catalog_name.clone());
        state.set_string("storage_root", self.storage_root.clone());
        Ok(state)
    }

    fn load_state(&mut self, state: &JourneyState) -> AcceptanceResult<()> {
        if let Some(catalog_name) = state.get_string("catalog_name") {
            self.catalog_name = catalog_name;
        }
        if let Some(storage_root) = state.get_string("storage_root") {
            self.storage_root = storage_root;
        }
        Ok(())
    }

    async fn execute(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
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

        println!("✅ Created catalog: {}", created_catalog.name);

        // Step 2: List catalogs to verify our catalog exists
        println!("📋 Listing catalogs to verify creation");
        let catalogs = client.list_catalogs(Some(10)); // Limit to 10 for demo
        let found_our_catalog = catalogs
            .any(|c| async {
                c.ok()
                    .map(|catalog| catalog.name == self.catalog_name)
                    .unwrap_or(false)
            })
            .await;

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

        client
            .catalog(&self.catalog_name)
            .delete(Some(true))
            .await?;
        println!("🗑️ Successfully deleted catalog '{}'", self.catalog_name);

        // Verify catalog properties
        assert_eq!(catalog_info.name, self.catalog_name);
        println!("✅ Verified catalog properties");

        println!("🎉 Simple catalog journey completed successfully!");
        Ok(())
    }

    async fn cleanup(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        println!("🧹 Cleaning up simple catalog journey");
        // Delete the catalog just in case it wasn't deleted
        // TODO: once we have proper error types in the client this should only allow not-found errors
        let _ = client.catalog(&self.catalog_name).delete(Some(true)).await;
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

        assert_eq!(journey.name(), "simple_catalog");
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
