//! External Volume Lifecycle Journey
//!
//! Tests the lifecycle of an external volume backed by cloud storage:
//! create credential → create external location → create EXTERNAL volume → get → delete
//!
//! NOTE: Pending recording against managed Databricks UC.
//! Run with UC_INTEGRATION_RECORD=true to record.

use async_trait::async_trait;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::credentials::v1::Purpose;
use unitycatalog_common::models::volumes::v1::VolumeType;

use crate::execution::{
    ImplementationTag, JourneyMetadata, JourneyState, JourneyTier, ResourceTag, UserJourney,
};
use crate::{AcceptanceError, AcceptanceResult};

pub struct VolumeExternalLifecycleJourney {
    catalog_name: String,
    schema_name: String,
    volume_name: String,
    credential_name: String,
    external_location_name: String,
    storage_root: String,
}

impl VolumeExternalLifecycleJourney {
    pub fn new(storage_root: impl Into<String>) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            catalog_name: format!("ext_vol_catalog_{}", timestamp),
            schema_name: format!("ext_vol_schema_{}", timestamp),
            volume_name: format!("ext_vol_{}", timestamp),
            credential_name: format!("ext_vol_cred_{}", timestamp),
            external_location_name: format!("ext_vol_loc_{}", timestamp),
            storage_root: storage_root.into(),
        }
    }
}

#[async_trait]
impl UserJourney for VolumeExternalLifecycleJourney {
    fn name(&self) -> &str {
        "volume_external_lifecycle"
    }

    fn description(&self) -> &str {
        "External volume lifecycle: credential + external location → create EXTERNAL volume → get → delete"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![
                ResourceTag::Volumes,
                ResourceTag::ExternalLocations,
                ResourceTag::Credentials,
                ResourceTag::Catalogs,
                ResourceTag::Schemas,
            ],
            implementations: vec![ImplementationTag::ManagedDatabricks],
            tier: JourneyTier::Tier2Governance,
            requires_external_storage: true,
        }
    }

    fn save_state(&self) -> AcceptanceResult<JourneyState> {
        let mut state = JourneyState::empty();
        state.set_string("catalog_name", self.catalog_name.clone());
        state.set_string("schema_name", self.schema_name.clone());
        state.set_string("volume_name", self.volume_name.clone());
        state.set_string("credential_name", self.credential_name.clone());
        state.set_string(
            "external_location_name",
            self.external_location_name.clone(),
        );
        state.set_string("storage_root", self.storage_root.clone());
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
        if let Some(v) = state.get_string("credential_name") {
            self.credential_name = v;
        }
        if let Some(v) = state.get_string("external_location_name") {
            self.external_location_name = v;
        }
        if let Some(v) = state.get_string("storage_root") {
            self.storage_root = v;
        }
        Ok(())
    }

    async fn execute(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        // Step 1: Create catalog and schema
        client
            .create_catalog(&self.catalog_name)
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create catalog: {}", e))
            })?;
        client
            .create_schema(&self.catalog_name, &self.schema_name)
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create schema: {}", e))
            })?;

        // Step 2: Create credential
        client
            .create_credential(&self.credential_name, Purpose::Storage)
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create credential: {}", e))
            })?;

        // Step 3: Create external location
        let volume_storage_path = format!("{}/volumes/{}/", self.storage_root, self.volume_name);
        client
            .create_external_location(
                &self.external_location_name,
                &volume_storage_path,
                &self.credential_name,
            )
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!(
                    "Failed to create external location: {}",
                    e
                ))
            })?;

        // Step 4: Create external volume
        println!(
            "  📦 Creating external volume '{}.{}.{}'",
            self.catalog_name, self.schema_name, self.volume_name
        );
        let volume = client
            .create_volume(
                &self.catalog_name,
                &self.schema_name,
                &self.volume_name,
                VolumeType::External,
            )
            .with_storage_location(volume_storage_path.clone())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!(
                    "Failed to create external volume: {}",
                    e
                ))
            })?;
        assert_eq!(volume.name, self.volume_name);
        println!("  ✓ External volume created: {}", volume.full_name);

        // Step 5: Get volume and verify it is external
        let fetched = client
            .volume(&self.catalog_name, &self.schema_name, &self.volume_name)
            .get()
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to get volume: {}", e))
            })?;
        assert_eq!(fetched.volume_type(), VolumeType::External);
        println!("  ✓ Volume type confirmed: External");

        Ok(())
    }

    async fn cleanup(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        let _ = client
            .volume(&self.catalog_name, &self.schema_name, &self.volume_name)
            .delete()
            .await;
        let _ = client
            .schema(&self.catalog_name, &self.schema_name)
            .delete()
            .await;
        let _ = client.catalog(&self.catalog_name).delete().await;
        let _ = client
            .external_location(&self.external_location_name)
            .delete()
            .await;
        let _ = client.credential(&self.credential_name).delete().await;
        Ok(())
    }
}
