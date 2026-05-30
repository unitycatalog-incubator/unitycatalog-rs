//! Credential Lifecycle Journey
//!
//! Tests the full lifecycle of a storage credential:
//! create → get → list → update comment → delete
//!
//! NOTE: Pending recording against managed Databricks UC.
//! Run with UC_INTEGRATION_RECORD=true to record.

use async_trait::async_trait;
use futures::StreamExt;
use unitycatalog_common::credentials::v1::{AwsIamRoleConfig, AzureManagedIdentity, Purpose};

use crate::execution::{
    ImplementationTag, JourneyContext, JourneyMetadata, JourneyState, JourneyTier, ResourceTag,
    UserJourney,
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

    async fn execute(&self, ctx: &JourneyContext) -> AcceptanceResult<()> {
        // A storage credential must reference a real cloud identity, so we read the
        // config from the environment. If none is configured we skip the journey
        // (it cannot succeed against a real server without it).
        //   UC_TEST_AWS_ROLE_ARN               — AWS IAM role ARN
        //   UC_TEST_AZURE_ACCESS_CONNECTOR_ID   — Azure Databricks Access Connector resource ID
        let aws_role_arn = std::env::var("UC_TEST_AWS_ROLE_ARN").ok();
        let azure_connector_id = std::env::var("UC_TEST_AZURE_ACCESS_CONNECTOR_ID").ok();
        if aws_role_arn.is_none() && azure_connector_id.is_none() {
            println!(
                "  ⏭️  Skipping credential_lifecycle — set UC_TEST_AWS_ROLE_ARN or \
                 UC_TEST_AZURE_ACCESS_CONNECTOR_ID to exercise storage credentials"
            );
            return Ok(());
        }

        // Step 1: Create credential with the configured cloud identity
        println!("  🔑 Creating credential '{}'", self.credential_name);
        let mut builder = ctx
            .client()
            .create_credential(&self.credential_name, Purpose::Storage)
            .with_comment("Credential lifecycle test".to_string());
        if let Some(role_arn) = aws_role_arn {
            builder = builder.with_aws_iam_role(AwsIamRoleConfig {
                role_arn,
                ..Default::default()
            });
        } else if let Some(access_connector_id) = azure_connector_id {
            builder = builder.with_azure_managed_identity(AzureManagedIdentity {
                access_connector_id,
                ..Default::default()
            });
        }
        let credential = builder.await.map_err(|e| {
            AcceptanceError::JourneyExecution(format!("Failed to create credential: {}", e))
        })?;
        println!("  ✓ Credential created: {}", credential.name);

        // Step 2: Get credential
        let fetched = ctx
            .client()
            .credential(&self.credential_name)
            .get()
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to get credential: {}", e))
            })?;
        assert_eq!(fetched.name, self.credential_name);
        println!("  ✓ Credential fetched: {}", fetched.name);

        // Step 3: List credentials
        let credentials: Vec<_> = ctx
            .client()
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
        ctx.client()
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

    async fn cleanup(&self, ctx: &JourneyContext) -> AcceptanceResult<()> {
        let _ = ctx
            .client()
            .credential(&self.credential_name)
            .delete()
            .await;
        Ok(())
    }
}
