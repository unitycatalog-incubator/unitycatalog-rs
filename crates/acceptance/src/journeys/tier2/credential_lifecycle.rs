//! Credential Lifecycle Journey
//!
//! Tests the full lifecycle of a storage credential:
//! create → get → list → update comment → delete
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

pub struct CredentialLifecycleJourney {
    credential_name: String,
}

impl CredentialLifecycleJourney {
    pub fn new() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            credential_name: format!("cred_lc_{}", timestamp),
        }
    }
}

impl Default for CredentialLifecycleJourney {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserJourney for CredentialLifecycleJourney {
    fn name(&self) -> &str {
        "credential_lifecycle"
    }

    fn description(&self) -> &str {
        "Storage credential lifecycle: create, get, list, update comment, delete"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![ResourceTag::Credentials],
            implementations: vec![ImplementationTag::ManagedDatabricks],
            tier: JourneyTier::Tier2Governance,
            requires_external_storage: false,
        }
    }

    fn save_state(&self) -> AcceptanceResult<JourneyState> {
        let mut state = JourneyState::empty();
        state.set_string("credential_name", self.credential_name.clone());
        Ok(state)
    }

    fn load_state(&mut self, state: &JourneyState) -> AcceptanceResult<()> {
        if let Some(v) = state.get_string("credential_name") {
            self.credential_name = v;
        }
        Ok(())
    }

    async fn execute(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        // Step 1: Create credential
        println!("  🔑 Creating credential '{}'", self.credential_name);
        let credential = client
            .create_credential(&self.credential_name, Purpose::Storage)
            .with_comment("Credential lifecycle test".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create credential: {}", e))
            })?;
        println!("  ✓ Credential created: {}", credential.name);

        // Step 2: Get credential
        let fetched = client
            .credential(&self.credential_name)
            .get()
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to get credential: {}", e))
            })?;
        assert_eq!(fetched.name, self.credential_name);
        println!("  ✓ Credential fetched: {}", fetched.name);

        // Step 3: List credentials
        let credentials: Vec<_> = client
            .list_credentials()
            .into_stream()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to list credentials: {}", e))
            })?;
        assert!(
            credentials.iter().any(|c| c.name == self.credential_name),
            "Created credential not found in list"
        );
        println!("  ✓ Listed {} credential(s)", credentials.len());

        // Step 4: Update comment
        client
            .credential(&self.credential_name)
            .update()
            .with_comment("Updated comment".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to update credential: {}", e))
            })?;
        println!("  ✓ Credential comment updated");

        Ok(())
    }

    async fn cleanup(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        let _ = client.credential(&self.credential_name).delete().await;
        Ok(())
    }
}
