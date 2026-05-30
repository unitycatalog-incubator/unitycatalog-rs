//! Temporary Volume Credentials Journey
//!
//! Tests generating temporary credentials for a managed volume:
//! create catalog+schema+managed volume → generate read/read-write temp creds → verify structure
//!
//! NOTE: Pending recording against managed Databricks UC.
//! Run with UC_INTEGRATION_RECORD=true to record.

use async_trait::async_trait;
use unitycatalog_client::VolumeOperation;
use unitycatalog_common::volumes::v1::VolumeType;

use crate::execution::{
    ImplementationTag, JourneyContext, JourneyMetadata, JourneyState, JourneyTier, ResourceTag,
    UserJourney,
};
use crate::{AcceptanceError, AcceptanceResult};

pub struct TemporaryVolumeCredentialsJourney {
    catalog_name: String,
    schema_name: String,
    volume_name: String,
}

impl TemporaryVolumeCredentialsJourney {
    pub fn new() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            catalog_name: format!("tmp_vol_cred_catalog_{}", timestamp),
            schema_name: format!("tmp_vol_cred_schema_{}", timestamp),
            volume_name: format!("tmp_vol_cred_{}", timestamp),
        }
    }
}

impl Default for TemporaryVolumeCredentialsJourney {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserJourney for TemporaryVolumeCredentialsJourney {
    fn name(&self) -> &str {
        "temporary_volume_credentials"
    }

    fn description(&self) -> &str {
        "Temporary volume credentials: create managed volume, generate read/read-write temp creds, verify structure"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![
                ResourceTag::TemporaryCredentials,
                ResourceTag::Volumes,
                ResourceTag::Catalogs,
                ResourceTag::Schemas,
            ],
            implementations: vec![ImplementationTag::ManagedDatabricks],
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
        let full_name = format!(
            "{}.{}.{}",
            self.catalog_name, self.schema_name, self.volume_name
        );

        // Step 1: Create catalog, schema, and managed volume
        ctx.client()
            .create_catalog(&self.catalog_name)
            .with_storage_root(ctx.storage_root.clone())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create catalog: {}", e))
            })?;
        ctx.client()
            .create_schema(&self.catalog_name, &self.schema_name)
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create schema: {}", e))
            })?;
        ctx.client()
            .create_volume(
                &self.catalog_name,
                &self.schema_name,
                &self.volume_name,
                VolumeType::Managed,
            )
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create volume: {}", e))
            })?;
        println!("  ✓ Managed volume created: {}", full_name);

        // Step 2: Generate temporary read credentials
        println!(
            "  🔐 Generating temporary read credentials for '{}'",
            full_name
        );
        let (read_cred, _volume_id) = ctx
            .client()
            .temporary_credentials()
            .temporary_volume_credential(full_name.clone(), VolumeOperation::Read)
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!(
                    "Failed to generate temporary read credentials: {}",
                    e
                ))
            })?;
        assert!(
            read_cred.credentials.is_some(),
            "No temporary credentials returned"
        );
        println!("  ✓ Temporary read credentials generated");

        // Step 3: Generate temporary read-write credentials
        println!(
            "  🔐 Generating temporary read-write credentials for '{}'",
            full_name
        );
        let (rw_cred, _volume_id) = ctx
            .client()
            .temporary_credentials()
            .temporary_volume_credential(full_name.clone(), VolumeOperation::ReadWrite)
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!(
                    "Failed to generate temporary read-write credentials: {}",
                    e
                ))
            })?;
        assert!(
            rw_cred.credentials.is_some(),
            "No temporary credentials returned"
        );
        println!("  ✓ Temporary read-write credentials generated");

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
