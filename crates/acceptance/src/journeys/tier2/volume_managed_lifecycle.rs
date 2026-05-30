//! Managed Volume Lifecycle Journey
//!
//! Tests the full lifecycle of a managed volume:
//! create catalog → create schema → create MANAGED volume → get → list → delete
//!
//! NOTE: Pending recording against managed Databricks UC.
//! Run with UC_INTEGRATION_RECORD=true to record.

use async_trait::async_trait;
use futures::StreamExt;
use unitycatalog_common::models::volumes::v1::VolumeType;

use crate::execution::{
    ImplementationTag, JourneyContext, JourneyMetadata, JourneyState, JourneyTier, ResourceTag,
    UserJourney,
};
use crate::{AcceptanceError, AcceptanceResult};

pub struct VolumeManagedLifecycleJourney {
    catalog_name: String,
    schema_name: String,
    volume_name: String,
}

impl VolumeManagedLifecycleJourney {
    pub fn new() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            catalog_name: format!("vol_lc_catalog_{}", timestamp),
            schema_name: format!("vol_lc_schema_{}", timestamp),
            volume_name: format!("vol_lc_{}", timestamp),
        }
    }
}

impl Default for VolumeManagedLifecycleJourney {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserJourney for VolumeManagedLifecycleJourney {
    fn name(&self) -> &str {
        "volume_managed_lifecycle"
    }

    fn description(&self) -> &str {
        "Managed volume lifecycle: create catalog+schema, create MANAGED volume, get, list, delete"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![
                ResourceTag::Catalogs,
                ResourceTag::Schemas,
                ResourceTag::Volumes,
            ],
            // The Java OSS server cannot deserialize our `VOLUME_TYPE_MANAGED`
            // enum value (it expects `MANAGED`), so volume creation 500s there.
            // Tracked as a client/server wire-format follow-up.
            implementations: vec![
                ImplementationTag::OssRust,
                ImplementationTag::ManagedDatabricks,
            ],
            tier: JourneyTier::Tier2Governance,
            requires_external_storage: false,
        }
    }

    fn save_state(&self) -> AcceptanceResult<JourneyState> {
        let mut state = JourneyState::empty();
        state.set_string("catalog_name", self.catalog_name.clone());
        state.set_string("schema_name", self.schema_name.clone());
        state.set_string("volume_name", self.volume_name.clone());
        Ok(state)
    }

    fn load_state(&mut self, state: &JourneyState) -> AcceptanceResult<()> {
        if let Some(v) = state.get_string("catalog_name") {
            self.catalog_name = v;
        }
        if let Some(v) = state.get_string("schema_name") {
            self.schema_name = v;
        }
        if let Some(v) = state.get_string("volume_name") {
            self.volume_name = v;
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

        // Step 2: Create schema
        println!(
            "  📂 Creating schema '{}.{}'",
            self.catalog_name, self.schema_name
        );
        ctx.client()
            .create_schema(&self.catalog_name, &self.schema_name)
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create schema: {}", e))
            })?;

        // Step 3: Create managed volume
        println!(
            "  📦 Creating managed volume '{}.{}.{}'",
            self.catalog_name, self.schema_name, self.volume_name
        );
        let volume = ctx
            .client()
            .create_volume(
                &self.catalog_name,
                &self.schema_name,
                &self.volume_name,
                VolumeType::Managed,
            )
            .with_comment("Managed volume lifecycle test".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create volume: {}", e))
            })?;
        assert_eq!(volume.name, self.volume_name);
        println!("  ✓ Volume created: {}", volume.full_name);

        // Step 4: Get volume
        let fetched = ctx
            .client()
            .volume(&self.catalog_name, &self.schema_name, &self.volume_name)
            .get()
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to get volume: {}", e))
            })?;
        assert_eq!(fetched.name, self.volume_name);
        println!("  ✓ Volume fetched: {}", fetched.full_name);

        // Step 5: List volumes
        let volumes: Vec<_> = ctx
            .client()
            .list_volumes(&self.catalog_name, &self.schema_name)
            .into_stream()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to list volumes: {}", e))
            })?;
        assert!(
            volumes.iter().any(|v| v.name == self.volume_name),
            "Created volume not found in list"
        );
        println!("  ✓ Listed {} volume(s)", volumes.len());

        Ok(())
    }

    async fn cleanup(&self, ctx: &JourneyContext) -> AcceptanceResult<()> {
        let _ = ctx
            .client()
            .volume(&self.catalog_name, &self.schema_name, &self.volume_name)
            .delete()
            .await;
        let _ = ctx
            .client()
            .schema(&self.catalog_name, &self.schema_name)
            .delete()
            .await;
        let _ = ctx.client().catalog(&self.catalog_name).delete().await;
        Ok(())
    }
}
