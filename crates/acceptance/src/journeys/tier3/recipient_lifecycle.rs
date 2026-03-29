//! Recipient Lifecycle Journey
//!
//! Tests the full lifecycle of a Delta Sharing recipient:
//! create → get → list → delete
//!
//! NOTE: Pending recording against managed Databricks UC.
//! Run with UC_INTEGRATION_RECORD=true to record.

use async_trait::async_trait;
use futures::StreamExt;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::recipients::v1::AuthenticationType;

use crate::execution::{
    ImplementationTag, JourneyMetadata, JourneyState, JourneyTier, ResourceTag, UserJourney,
};
use crate::{AcceptanceError, AcceptanceResult};

pub struct RecipientLifecycleJourney {
    recipient_name: String,
}

impl RecipientLifecycleJourney {
    pub fn new() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            recipient_name: format!("recipient_lc_{}", timestamp),
        }
    }
}

impl Default for RecipientLifecycleJourney {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserJourney for RecipientLifecycleJourney {
    fn name(&self) -> &str {
        "recipient_lifecycle"
    }

    fn description(&self) -> &str {
        "Recipient lifecycle: create TOKEN recipient, get, list, update comment, delete"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![ResourceTag::Recipients],
            implementations: vec![
                ImplementationTag::ManagedDatabricks,
                ImplementationTag::OssRust,
            ],
            tier: JourneyTier::Tier3Sharing,
            requires_external_storage: false,
        }
    }

    fn save_state(&self) -> AcceptanceResult<JourneyState> {
        let mut state = JourneyState::empty();
        state.set_string("recipient_name", self.recipient_name.clone());
        Ok(state)
    }

    fn load_state(&mut self, state: &JourneyState) -> AcceptanceResult<()> {
        if let Some(v) = state.get_string("recipient_name") {
            self.recipient_name = v;
        }
        Ok(())
    }

    async fn execute(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        // Step 1: Create recipient with TOKEN authentication
        println!("  👤 Creating recipient '{}'", self.recipient_name);
        let recipient = client
            .create_recipient(
                &self.recipient_name,
                AuthenticationType::Token,
                "acceptance-test",
            )
            .with_comment("Recipient lifecycle test".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create recipient: {}", e))
            })?;
        assert_eq!(recipient.name, self.recipient_name);
        println!("  ✓ Recipient created: {}", recipient.name);

        // Step 2: Get recipient
        let fetched = client
            .recipient(&self.recipient_name)
            .get()
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to get recipient: {}", e))
            })?;
        assert_eq!(fetched.name, self.recipient_name);
        println!("  ✓ Recipient fetched: {}", fetched.name);

        // Step 3: List recipients
        let recipients: Vec<_> = client
            .list_recipients()
            .into_stream()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to list recipients: {}", e))
            })?;
        assert!(
            recipients.iter().any(|r| r.name == self.recipient_name),
            "Created recipient not found in list"
        );
        println!("  ✓ Listed {} recipient(s)", recipients.len());

        Ok(())
    }

    async fn cleanup(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        let _ = client.recipient(&self.recipient_name).delete().await;
        Ok(())
    }
}
