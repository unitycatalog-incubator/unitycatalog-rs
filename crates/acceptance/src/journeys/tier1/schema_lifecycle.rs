//! Schema Lifecycle Journey
//!
//! Tests the full lifecycle of a Unity Catalog schema:
//! create catalog → create schema → get → list → update comment → delete schema → delete catalog

use async_trait::async_trait;
use futures::StreamExt;

use crate::execution::{
    ImplementationTag, JourneyContext, JourneyMetadata, JourneyState, JourneyTier, ResourceTag,
    UserJourney,
};
use crate::{AcceptanceError, AcceptanceResult};

pub struct SchemaLifecycleJourney {
    catalog_name: String,
    schema_name: String,
}

impl SchemaLifecycleJourney {
    pub fn new() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            catalog_name: format!("schema_lc_catalog_{}", timestamp),
            schema_name: format!("schema_lc_{}", timestamp),
        }
    }
}

impl Default for SchemaLifecycleJourney {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserJourney for SchemaLifecycleJourney {
    fn name(&self) -> &str {
        "schema_lifecycle"
    }

    fn description(&self) -> &str {
        "Schema lifecycle: create catalog, update catalog comment, create schema, get, list, update comment, delete"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![ResourceTag::Catalogs, ResourceTag::Schemas],
            implementations: vec![ImplementationTag::All],
            tier: JourneyTier::Tier1Crud,
            requires_external_storage: false,
        }
    }

    fn save_state(&self) -> AcceptanceResult<JourneyState> {
        let mut state = JourneyState::empty();
        state.set_string("catalog_name", self.catalog_name.clone());
        state.set_string("schema_name", self.schema_name.clone());
        Ok(state)
    }

    fn load_state(&mut self, state: &JourneyState) -> AcceptanceResult<()> {
        if let Some(v) = state.get_string("catalog_name") {
            self.catalog_name = v;
        }
        if let Some(v) = state.get_string("schema_name") {
            self.schema_name = v;
        }
        Ok(())
    }

    async fn execute(&self, ctx: &JourneyContext) -> AcceptanceResult<()> {
        // Step 1: Create catalog
        println!("  📁 Creating catalog '{}'", self.catalog_name);
        ctx.client()
            .create_catalog(&self.catalog_name)
            .with_storage_root(ctx.storage_root.clone())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create catalog: {}", e))
            })?;

        // Step 1b: Update catalog comment (exercises Catalog UPDATE / PATCH)
        ctx.client()
            .catalog(&self.catalog_name)
            .update()
            .with_comment("Updated catalog comment".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to update catalog: {}", e))
            })?;
        println!("  ✓ Catalog comment updated");

        // Step 2: Create schema
        println!(
            "  📂 Creating schema '{}.{}'",
            self.catalog_name, self.schema_name
        );
        let schema = ctx
            .client()
            .create_schema(&self.catalog_name, &self.schema_name)
            .with_comment("Schema lifecycle test".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create schema: {}", e))
            })?;

        assert_eq!(schema.name, self.schema_name);
        assert_eq!(schema.catalog_name, self.catalog_name);
        println!("  ✓ Schema created: {}", schema.full_name);

        // Step 3: Get schema
        let fetched = ctx
            .client()
            .schema(&self.catalog_name, &self.schema_name)
            .get()
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to get schema: {}", e))
            })?;
        assert_eq!(fetched.name, self.schema_name);
        println!("  ✓ Schema fetched: {}", fetched.full_name);

        // Step 4: List schemas
        let schemas: Vec<_> = ctx
            .client()
            .list_schemas(&self.catalog_name)
            .into_stream()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to list schemas: {}", e))
            })?;
        assert!(
            schemas.iter().any(|s| s.name == self.schema_name),
            "Created schema not found in list"
        );
        println!("  ✓ Listed {} schema(s)", schemas.len());

        // Step 5: Update comment
        ctx.client()
            .schema(&self.catalog_name, &self.schema_name)
            .update()
            .with_comment("Updated comment".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to update schema: {}", e))
            })?;
        println!("  ✓ Schema comment updated");

        Ok(())
    }

    async fn cleanup(&self, ctx: &JourneyContext) -> AcceptanceResult<()> {
        // Delete schema
        let _ = ctx
            .client()
            .schema(&self.catalog_name, &self.schema_name)
            .delete()
            .await;

        // Delete catalog
        let _ = ctx.client().catalog(&self.catalog_name).delete().await;

        Ok(())
    }
}
