//! Enhanced Catalog Journey Example
//!
//! This example demonstrates how to use the new reporting utilities to create
//! rich, well-structured journey logs with progress tracking, performance metrics,
//! and condensed output.

use async_trait::async_trait;
use futures::StreamExt;
use unitycatalog_client::UnityCatalogClient;

use crate::init_journey;
use crate::journey::{JourneyState, UserJourney};
use crate::journey_helpers::{JourneyLogger, cleanup_step};
use crate::reporting::ReportingConfig;
use crate::{AcceptanceError, AcceptanceResult};

/// Enhanced catalog journey with rich reporting
pub struct EnhancedCatalogJourney {
    catalog_name: String,
    storage_root: String,
    logger: Option<JourneyLogger>,
}

impl EnhancedCatalogJourney {
    /// Create a new enhanced catalog journey
    pub fn new() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        let catalog_name = format!("enhanced_catalog_{}", timestamp);

        Self {
            catalog_name,
            storage_root: "s3://open-lakehouse-dev/".to_string(),
            logger: None,
        }
    }

    /// Create with a specific catalog name
    pub fn with_catalog_name(catalog_name: impl Into<String>) -> Self {
        Self {
            catalog_name: catalog_name.into(),
            storage_root: "s3://open-lakehouse-dev/".to_string(),
            logger: None,
        }
    }

    /// Create with custom configuration
    pub fn with_config(catalog_name: impl Into<String>, config: ReportingConfig) -> Self {
        let logger = JourneyLogger::with_config("enhanced_catalog", config);

        Self {
            catalog_name: catalog_name.into(),
            storage_root: "s3://open-lakehouse-dev/".to_string(),
            logger: Some(logger),
        }
    }

    /// Get the catalog name for this journey
    pub fn catalog_name(&self) -> &str {
        &self.catalog_name
    }
}

#[async_trait]
impl UserJourney for EnhancedCatalogJourney {
    fn name(&self) -> &str {
        "enhanced_catalog"
    }

    fn description(&self) -> &str {
        "Enhanced catalog lifecycle with rich reporting: create, list, inspect, delete"
    }

    fn tags(&self) -> Vec<&str> {
        vec!["catalog", "enhanced", "reporting", "example", "demo"]
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
        // Create logger directly since we can't mutate self
        let logger = if let Some(ref logger) = self.logger {
            logger.clone()
        } else {
            init_journey!(
                "enhanced_catalog",
                &format!("Enhanced catalog lifecycle for '{}'", self.catalog_name)
            )
        };

        // Step 1: Create catalog
        logger.info("📁 Creating catalog")?;
        let created_catalog = logger
            .step("create_catalog", async {
                client
                    .create_catalog(&self.catalog_name)
                    .with_storage_root(self.storage_root.clone())
                    .with_comment(Some(
                        "Enhanced test catalog with rich reporting".to_string(),
                    ))
                    .await
                    .map_err(|e| {
                        AcceptanceError::UnityCatalog(format!("Failed to create catalog: {}", e))
                    })
            })
            .await?;

        // Step 2: Verify creation by checking the returned catalog
        logger.info("🔍 Verifying catalog creation")?;
        logger
            .step("verify_creation", async {
                if created_catalog.name != self.catalog_name {
                    return Err(AcceptanceError::JourneyValidation(format!(
                        "Created catalog name '{}' doesn't match expected '{}'",
                        created_catalog.name, self.catalog_name
                    )));
                }
                logger.info(&format!(
                    "Catalog created with ID: {}",
                    created_catalog.name
                ))?;
                Ok(())
            })
            .await?;

        // Step 3: List catalogs and find ours
        logger.info("📋 Listing all catalogs")?;
        let _catalog_found = logger
            .step("list_catalogs", async {
                let catalogs = client.list_catalogs(Some(50));
                let found = catalogs
                    .any(|c| async {
                        c.ok()
                            .map(|catalog| catalog.name == self.catalog_name)
                            .unwrap_or(false)
                    })
                    .await;

                if !found {
                    return Err(AcceptanceError::JourneyExecution(format!(
                        "Created catalog '{}' was not found in the catalog list",
                        self.catalog_name
                    )));
                }

                logger.info(&format!("Found catalog '{}' in listing", self.catalog_name))?;
                Ok(found)
            })
            .await?;

        // Step 4: Get detailed catalog information
        logger.info("🔍 Getting catalog details")?;
        let catalog_info = logger
            .step("inspect_catalog", async {
                client.catalog(&self.catalog_name).get().await.map_err(|e| {
                    AcceptanceError::UnityCatalog(format!("Failed to get catalog info: {}", e))
                })
            })
            .await?;

        // Step 5: Validate catalog properties
        logger.info("🔍 Validating catalog properties")?;
        logger
            .step("validate_properties", async {
                // Validate name
                if catalog_info.name != self.catalog_name {
                    return Err(AcceptanceError::JourneyValidation(format!(
                        "Catalog name mismatch: expected '{}', got '{}'",
                        self.catalog_name, catalog_info.name
                    )));
                }

                // Validate storage root if provided
                if let Some(storage_root) = &catalog_info.storage_root {
                    if storage_root != &self.storage_root {
                        logger.warn(&format!(
                            "Storage root mismatch: expected '{}', got '{}'",
                            self.storage_root, storage_root
                        ))?;
                    }
                }

                // Validate comment
                if let Some(comment) = &catalog_info.comment {
                    if !comment.contains("Enhanced test catalog") {
                        logger.warn("Catalog comment doesn't match expected content")?;
                    }
                }

                // Log detailed information
                logger.info("Catalog validation successful:")?;
                logger.info(&format!("  • Name: {}", catalog_info.name))?;
                logger.info(&format!(
                    "  • ID: {}",
                    catalog_info.id.unwrap_or_else(|| "N/A".to_string())
                ))?;
                if let Some(storage_root) = &catalog_info.storage_root {
                    logger.info(&format!("  • Storage Root: {}", storage_root))?;
                }
                if let Some(comment) = &catalog_info.comment {
                    logger.info(&format!("  • Comment: {}", comment))?;
                }

                Ok(())
            })
            .await?;

        // Step 6: Clean up by deleting the catalog
        logger.info("🗑️ Deleting catalog")?;
        logger
            .step_with_description("cleanup_catalog", "Delete test artifact.", async {
                client
                    .catalog(&self.catalog_name)
                    .delete(Some(true))
                    .await
                    .map_err(|e| {
                        AcceptanceError::UnityCatalog(format!("Failed to delete catalog: {}", e))
                    })
            })
            .await?;

        // Note: Performance summary removed due to immutability constraints
        logger.info("Journey completed successfully")?;

        // Finish successfully
        logger.finish(true)?;

        Ok(())
    }

    async fn cleanup(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        if let Some(logger) = &self.logger {
            logger.info("Starting cleanup operations")?;

            // Try to delete the catalog if it still exists
            cleanup_step(
                logger,
                "cleanup_delete_catalog",
                client.catalog(&self.catalog_name).delete(Some(true)),
            )
            .await?;

            logger.info("Cleanup completed")?;
        }
        Ok(())
    }
}

impl Default for EnhancedCatalogJourney {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporting::ReportingConfig;

    #[test]
    fn test_enhanced_catalog_journey_properties() {
        let journey = EnhancedCatalogJourney::new();

        assert_eq!(journey.name(), "enhanced_catalog");
        assert!(!journey.description().is_empty());
        assert!(journey.tags().contains(&"catalog"));
        assert!(journey.tags().contains(&"enhanced"));
        assert!(journey.tags().contains(&"reporting"));
    }

    #[test]
    fn test_custom_catalog_name() {
        let journey = EnhancedCatalogJourney::with_catalog_name("my_enhanced_catalog");
        assert_eq!(journey.catalog_name, "my_enhanced_catalog");
    }

    #[test]
    fn test_with_custom_config() {
        let config = ReportingConfig {
            verbosity: 2,
            ..ReportingConfig::default()
        };

        let journey = EnhancedCatalogJourney::with_config("test_catalog", config);
        assert_eq!(journey.catalog_name, "test_catalog");
        assert!(journey.logger.is_some());
    }

    #[test]
    fn test_default_journey() {
        let journey = EnhancedCatalogJourney::default();
        assert!(journey.catalog_name.starts_with("enhanced_catalog_"));
    }

    #[test]
    fn test_state_management() {
        let journey = EnhancedCatalogJourney::new();

        // Save state
        let state = journey.save_state().unwrap();
        assert!(state.get_string("catalog_name").is_some());
        assert!(state.get_string("storage_root").is_some());

        // Create new journey and load state
        let mut new_journey = EnhancedCatalogJourney::with_catalog_name("different_name");
        new_journey.load_state(&state).unwrap();

        assert_eq!(new_journey.catalog_name, journey.catalog_name);
        assert_eq!(new_journey.storage_root, journey.storage_root);
    }

    #[test]
    fn test_journey_initialization() {
        let journey = EnhancedCatalogJourney::new();
        assert!(journey.catalog_name.starts_with("enhanced_catalog_"));
        assert_eq!(journey.storage_root, "s3://open-lakehouse-dev/");
        assert!(journey.logger.is_none());
    }
}
