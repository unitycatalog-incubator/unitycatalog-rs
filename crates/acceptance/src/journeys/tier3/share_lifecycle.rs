//! Share Lifecycle Journey
//!
//! Tests the full lifecycle of a Delta Share:
//! create catalog+schema+table → create share → get → list → delete
//!
//! NOTE: Pending recording against managed Databricks UC.
//! Run with UC_INTEGRATION_RECORD=true to record.

use async_trait::async_trait;
use futures::StreamExt;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::tables::v1::{DataSourceFormat, TableType};

use crate::execution::{
    ImplementationTag, JourneyMetadata, JourneyState, JourneyTier, ResourceTag, UserJourney,
};
use crate::{AcceptanceError, AcceptanceResult};

pub struct ShareLifecycleJourney {
    catalog_name: String,
    schema_name: String,
    table_name: String,
    share_name: String,
}

impl ShareLifecycleJourney {
    pub fn new() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            catalog_name: format!("share_lc_catalog_{}", timestamp),
            schema_name: format!("share_lc_schema_{}", timestamp),
            table_name: format!("share_lc_table_{}", timestamp),
            share_name: format!("share_lc_{}", timestamp),
        }
    }
}

impl Default for ShareLifecycleJourney {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserJourney for ShareLifecycleJourney {
    fn name(&self) -> &str {
        "share_lifecycle"
    }

    fn description(&self) -> &str {
        "Share lifecycle: create table, create share, get, list, delete"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![
                ResourceTag::Shares,
                ResourceTag::Tables,
                ResourceTag::Catalogs,
                ResourceTag::Schemas,
            ],
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
        state.set_string("catalog_name", self.catalog_name.clone());
        state.set_string("schema_name", self.schema_name.clone());
        state.set_string("table_name", self.table_name.clone());
        state.set_string("share_name", self.share_name.clone());
        Ok(state)
    }

    fn load_state(&mut self, state: &JourneyState) -> AcceptanceResult<()> {
        if let Some(v) = state.get_string("catalog_name") {
            self.catalog_name = v;
        }
        if let Some(v) = state.get_string("schema_name") {
            self.schema_name = v;
        }
        if let Some(v) = state.get_string("table_name") {
            self.table_name = v;
        }
        if let Some(v) = state.get_string("share_name") {
            self.share_name = v;
        }
        Ok(())
    }

    async fn execute(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        let table_full_name = format!(
            "{}.{}.{}",
            self.catalog_name, self.schema_name, self.table_name
        );

        // Step 1: Create catalog, schema, and managed table
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
        client
            .create_table(
                &self.table_name,
                &self.schema_name,
                &self.catalog_name,
                TableType::Managed,
                DataSourceFormat::Delta,
            )
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create table: {}", e))
            })?;
        println!("  ✓ Table created: {}", table_full_name);

        // Step 2: Create share
        println!("  🤝 Creating share '{}'", self.share_name);
        let share = client
            .create_share(&self.share_name)
            .with_comment("Share lifecycle test".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create share: {}", e))
            })?;
        assert_eq!(share.name, self.share_name);
        println!("  ✓ Share created: {}", share.name);

        // Step 3: Get share
        let fetched = client.share(&self.share_name).get().await.map_err(|e| {
            AcceptanceError::JourneyExecution(format!("Failed to get share: {}", e))
        })?;
        assert_eq!(fetched.name, self.share_name);
        println!("  ✓ Share fetched: {}", fetched.name);

        // Step 4: List shares
        let shares: Vec<_> = client
            .list_shares()
            .into_stream()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to list shares: {}", e))
            })?;
        assert!(
            shares.iter().any(|s| s.name == self.share_name),
            "Created share not found in list"
        );
        println!("  ✓ Listed {} share(s)", shares.len());

        Ok(())
    }

    async fn cleanup(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        let table_full_name = format!(
            "{}.{}.{}",
            self.catalog_name, self.schema_name, self.table_name
        );
        let _ = client.share(&self.share_name).delete().await;
        let _ = client.table(&table_full_name).delete().await;
        let _ = client
            .schema(&self.catalog_name, &self.schema_name)
            .delete()
            .await;
        let _ = client.catalog(&self.catalog_name).delete().await;
        Ok(())
    }
}
