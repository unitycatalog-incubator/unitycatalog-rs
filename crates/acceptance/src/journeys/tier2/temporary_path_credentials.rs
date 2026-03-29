//! Temporary Path Credentials Journey
//!
//! Tests generating temporary credentials for an external storage path:
//! create credential → create external location → generate temp path creds → verify structure
//!
//! NOTE: Pending recording against managed Databricks UC.
//! Run with UC_INTEGRATION_RECORD=true to record.

use async_trait::async_trait;
use unitycatalog_client::{PathOperation, UnityCatalogClient};
use unitycatalog_common::credentials::v1::Purpose;

use crate::execution::{
    ImplementationTag, JourneyMetadata, JourneyState, JourneyTier, ResourceTag, UserJourney,
};
use crate::{AcceptanceError, AcceptanceResult};

pub struct TemporaryPathCredentialsJourney {
    credential_name: String,
    external_location_name: String,
    storage_root: String,
}

impl TemporaryPathCredentialsJourney {
    pub fn new(storage_root: impl Into<String>) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            credential_name: format!("tmp_path_cred_{}", timestamp),
            external_location_name: format!("tmp_path_loc_{}", timestamp),
            storage_root: storage_root.into(),
        }
    }
}

#[async_trait]
impl UserJourney for TemporaryPathCredentialsJourney {
    fn name(&self) -> &str {
        "temporary_path_credentials"
    }

    fn description(&self) -> &str {
        "Temporary path credentials: create external location, generate temp path creds, verify"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![
                ResourceTag::TemporaryCredentials,
                ResourceTag::ExternalLocations,
                ResourceTag::Credentials,
            ],
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
        // Step 1: Create credential
        client
            .create_credential(&self.credential_name, Purpose::Storage)
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create credential: {}", e))
            })?;

        // Step 2: Create external location
        client
            .create_external_location(
                &self.external_location_name,
                &self.storage_root,
                &self.credential_name,
            )
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!(
                    "Failed to create external location: {}",
                    e
                ))
            })?;
        println!(
            "  ✓ External location created: {}",
            self.external_location_name
        );

        // Step 3: Generate temporary read credentials for the path
        println!(
            "  🔐 Generating temporary path read credentials for '{}'",
            self.storage_root
        );
        let (read_cred, _url) = client
            .temporary_credentials()
            .temporary_path_credential(&self.storage_root, PathOperation::Read, None)
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!(
                    "Failed to generate temporary path credentials: {}",
                    e
                ))
            })?;
        assert!(
            read_cred.credentials.is_some(),
            "No temporary credentials returned"
        );
        println!("  ✓ Temporary path read credentials generated");

        // Step 4: Generate temporary read-write credentials
        let (rw_cred, _url) = client
            .temporary_credentials()
            .temporary_path_credential(&self.storage_root, PathOperation::ReadWrite, None)
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!(
                    "Failed to generate temporary read-write path credentials: {}",
                    e
                ))
            })?;
        assert!(
            rw_cred.credentials.is_some(),
            "No temporary credentials returned"
        );
        println!("  ✓ Temporary path read-write credentials generated");

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
