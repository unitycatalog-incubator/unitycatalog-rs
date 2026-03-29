//! External Location Lifecycle Journey
//!
//! Tests the full lifecycle of an external location:
//! create credential → create external location → list → delete location → delete credential
//!
//! NOTE: Pending recording against managed Databricks UC.
//! Run with UC_INTEGRATION_RECORD=true to record.

use async_trait::async_trait;
use futures::StreamExt;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::credentials::v1::Purpose;

use crate::execution::{
    ImplementationTag, JourneyMetadata, JourneyState, JourneyTier, ResourceTag, UserJourney,
};
use crate::{AcceptanceError, AcceptanceResult};

pub struct ExternalLocationLifecycleJourney {
    credential_name: String,
    external_location_name: String,
    storage_root: String,
}

impl ExternalLocationLifecycleJourney {
    pub fn new(storage_root: impl Into<String>) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            credential_name: format!("ext_loc_cred_{}", timestamp),
            external_location_name: format!("ext_loc_{}", timestamp),
            storage_root: storage_root.into(),
        }
    }
}

#[async_trait]
impl UserJourney for ExternalLocationLifecycleJourney {
    fn name(&self) -> &str {
        "external_location_lifecycle"
    }

    fn description(&self) -> &str {
        "External location lifecycle: create credential, create external location, list, delete"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![ResourceTag::ExternalLocations, ResourceTag::Credentials],
            implementations: vec![ImplementationTag::ManagedDatabricks],
            tier: JourneyTier::Tier2Governance,
            requires_external_storage: true,
        }
    }

    fn save_state(&self) -> AcceptanceResult<JourneyState> {
        let mut state = JourneyState::empty();
        state.set_string("credential_name", self.credential_name.clone());
        state.set_string(
            "external_location_name",
            self.external_location_name.clone(),
        );
        state.set_string("storage_root", self.storage_root.clone());
        Ok(state)
    }

    fn load_state(&mut self, state: &JourneyState) -> AcceptanceResult<()> {
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
        // Step 1: Create storage credential
        println!("  🔑 Creating credential '{}'", self.credential_name);
        client
            .create_credential(&self.credential_name, Purpose::Storage)
            .with_comment("External location lifecycle test credential".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create credential: {}", e))
            })?;

        // Step 2: Create external location
        println!(
            "  🌍 Creating external location '{}'",
            self.external_location_name
        );
        let ext_loc = client
            .create_external_location(
                &self.external_location_name,
                &self.storage_root,
                &self.credential_name,
            )
            .with_comment("External location lifecycle test".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!(
                    "Failed to create external location: {}",
                    e
                ))
            })?;
        println!("  ✓ External location created: {}", ext_loc.name);

        // Step 3: List external locations
        let locations: Vec<_> = client
            .list_external_locations()
            .into_stream()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!(
                    "Failed to list external locations: {}",
                    e
                ))
            })?;
        assert!(
            locations
                .iter()
                .any(|l| l.name == self.external_location_name),
            "Created external location not found in list"
        );
        println!("  ✓ Listed {} external location(s)", locations.len());

        Ok(())
    }

    async fn cleanup(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        let _ = client
            .external_location(&self.external_location_name)
            .delete()
            .await;
        let _ = client.credential(&self.credential_name).delete().await;
        Ok(())
    }
}
