//! Temporary Table Credentials Journey
//!
//! Tests generating temporary credentials for a managed table:
//! create catalog+schema+table → generate temp table creds → verify structure
//!
//! NOTE: Pending recording against managed Databricks UC.
//! Run with UC_INTEGRATION_RECORD=true to record.

use async_trait::async_trait;
use unitycatalog_client::{TableOperation, UnityCatalogClient};
use unitycatalog_common::tables::v1::{DataSourceFormat, TableType};

use crate::execution::{
    ImplementationTag, JourneyMetadata, JourneyState, JourneyTier, ResourceTag, UserJourney,
};
use crate::{AcceptanceError, AcceptanceResult};

pub struct TemporaryTableCredentialsJourney {
    catalog_name: String,
    schema_name: String,
    table_name: String,
}

impl TemporaryTableCredentialsJourney {
    pub fn new() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            catalog_name: format!("tmp_tbl_cred_catalog_{}", timestamp),
            schema_name: format!("tmp_tbl_cred_schema_{}", timestamp),
            table_name: format!("tmp_tbl_cred_{}", timestamp),
        }
    }
}

impl Default for TemporaryTableCredentialsJourney {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserJourney for TemporaryTableCredentialsJourney {
    fn name(&self) -> &str {
        "temporary_table_credentials"
    }

    fn description(&self) -> &str {
        "Temporary table credentials: create managed table, generate read/write temp creds, verify structure"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![
                ResourceTag::TemporaryCredentials,
                ResourceTag::Tables,
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
        state.set_string("table_name", self.table_name.clone());
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
        Ok(())
    }

    async fn execute(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        let full_name = format!(
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
        println!("  ✓ Managed table created: {}", full_name);

        // Step 2: Generate temporary read credentials
        println!(
            "  🔐 Generating temporary read credentials for '{}'",
            full_name
        );
        let (read_cred, _table_id) = client
            .temporary_credentials()
            .temporary_table_credential(full_name.clone(), TableOperation::Read)
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
        let (rw_cred, _table_id) = client
            .temporary_credentials()
            .temporary_table_credential(full_name.clone(), TableOperation::ReadWrite)
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

    async fn cleanup(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        let full_name = format!(
            "{}.{}.{}",
            self.catalog_name, self.schema_name, self.table_name
        );
        let _ = client.table(&full_name).delete().await;
        let _ = client
            .schema(&self.catalog_name, &self.schema_name)
            .delete()
            .await;
        let _ = client.catalog(&self.catalog_name).delete().await;
        Ok(())
    }
}
