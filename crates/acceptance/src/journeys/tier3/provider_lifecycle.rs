//! Provider Lifecycle Journey
//!
//! Tests the full lifecycle of a Delta Sharing provider:
//! create → get → list → update comment → delete
//!
//! NOTE: Pending recording against managed Databricks UC.
//! Run with UC_INTEGRATION_RECORD=true to record.

use async_trait::async_trait;
use futures::StreamExt;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::providers::v1::ProviderAuthenticationType;

use crate::execution::{
    ImplementationTag, JourneyMetadata, JourneyState, JourneyTier, ResourceTag, UserJourney,
};
use crate::{AcceptanceError, AcceptanceResult};

/// A minimal open-sharing recipient profile, required when creating a TOKEN provider.
const RECIPIENT_PROFILE: &str = r#"{"shareCredentialsVersion":1,"endpoint":"https://sharing.example.com/delta-sharing/","bearerToken":"acceptance-test-token","expirationTime":"2099-01-01T00:00:00.000Z"}"#;

pub struct ProviderLifecycleJourney {
    provider_name: String,
}

impl ProviderLifecycleJourney {
    pub fn new() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            provider_name: format!("provider_lc_{}", timestamp),
        }
    }
}

impl Default for ProviderLifecycleJourney {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserJourney for ProviderLifecycleJourney {
    fn name(&self) -> &str {
        "provider_lifecycle"
    }

    fn description(&self) -> &str {
        "Provider lifecycle: create TOKEN provider, get, list, update comment, delete"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![ResourceTag::Providers],
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
        state.set_string("provider_name", self.provider_name.clone());
        Ok(state)
    }

    fn load_state(&mut self, state: &JourneyState) -> AcceptanceResult<()> {
        if let Some(v) = state.get_string("provider_name") {
            self.provider_name = v;
        }
        Ok(())
    }

    async fn execute(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        // Step 1: Create provider with TOKEN authentication
        println!("  🔗 Creating provider '{}'", self.provider_name);
        let provider = client
            .create_provider(&self.provider_name, ProviderAuthenticationType::Token)
            .with_recipient_profile_str(RECIPIENT_PROFILE.to_string())
            .with_comment("Provider lifecycle test".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create provider: {}", e))
            })?;
        assert_eq!(provider.name, self.provider_name);
        println!("  ✓ Provider created: {}", provider.name);

        // Step 2: Get provider
        let fetched = client
            .provider(&self.provider_name)
            .get()
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to get provider: {}", e))
            })?;
        assert_eq!(fetched.name, self.provider_name);
        println!("  ✓ Provider fetched: {}", fetched.name);

        // Step 3: List providers
        let providers: Vec<_> = client
            .list_providers()
            .into_stream()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to list providers: {}", e))
            })?;
        assert!(
            providers.iter().any(|p| p.name == self.provider_name),
            "Created provider not found in list"
        );
        println!("  ✓ Listed {} provider(s)", providers.len());

        // Step 4: Update provider comment (exercises Provider UPDATE / PATCH)
        client
            .provider(&self.provider_name)
            .update()
            .with_comment("Updated provider comment".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to update provider: {}", e))
            })?;
        println!("  ✓ Provider comment updated");

        Ok(())
    }

    async fn cleanup(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        let _ = client.provider(&self.provider_name).delete().await;
        Ok(())
    }
}
