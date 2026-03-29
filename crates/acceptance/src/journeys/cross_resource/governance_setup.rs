//! Governance Setup Journey
//!
//! Tests the full governance chain: catalog → schema → credential → external location → external table.
//! This represents the typical workflow a data engineer performs when setting up a governed
//! external data asset in Unity Catalog.
//!
//! NOTE: Pending recording against managed Databricks UC.
//! Run with UC_INTEGRATION_RECORD=true to record.

use async_trait::async_trait;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::credentials::v1::Purpose;
use unitycatalog_common::tables::v1::{DataSourceFormat, TableType};

use crate::execution::{
    ImplementationTag, JourneyMetadata, JourneyState, JourneyTier, ResourceTag, UserJourney,
};
use crate::{AcceptanceError, AcceptanceResult};

pub struct GovernanceSetupJourney {
    catalog_name: String,
    schema_name: String,
    table_name: String,
    credential_name: String,
    external_location_name: String,
    storage_root: String,
}

impl GovernanceSetupJourney {
    pub fn new(storage_root: impl Into<String>) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            catalog_name: format!("gov_catalog_{}", timestamp),
            schema_name: format!("gov_schema_{}", timestamp),
            table_name: format!("gov_table_{}", timestamp),
            credential_name: format!("gov_cred_{}", timestamp),
            external_location_name: format!("gov_loc_{}", timestamp),
            storage_root: storage_root.into(),
        }
    }
}

#[async_trait]
impl UserJourney for GovernanceSetupJourney {
    fn name(&self) -> &str {
        "governance_setup"
    }

    fn description(&self) -> &str {
        "Full governance chain: catalog → schema → storage credential → external location → external table"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![
                ResourceTag::Catalogs,
                ResourceTag::Schemas,
                ResourceTag::Credentials,
                ResourceTag::ExternalLocations,
                ResourceTag::Tables,
            ],
            implementations: vec![ImplementationTag::ManagedDatabricks],
            tier: JourneyTier::Tier4Advanced,
            requires_external_storage: true,
        }
    }

    fn save_state(&self) -> AcceptanceResult<JourneyState> {
        let mut state = JourneyState::empty();
        state.set_string("catalog_name", self.catalog_name.clone());
        state.set_string("schema_name", self.schema_name.clone());
        state.set_string("table_name", self.table_name.clone());
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
        if let Some(v) = state.get_string("table_name") {
            self.table_name = v;
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
        let table_location = format!("{}/tables/{}/", self.storage_root, self.table_name);
        let full_name = format!(
            "{}.{}.{}",
            self.catalog_name, self.schema_name, self.table_name
        );

        // Step 1: Create catalog
        println!("  📁 [1/5] Creating catalog '{}'", self.catalog_name);
        client
            .create_catalog(&self.catalog_name)
            .with_comment("Governance test catalog".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create catalog: {}", e))
            })?;

        // Step 2: Create schema
        println!(
            "  📂 [2/5] Creating schema '{}.{}'",
            self.catalog_name, self.schema_name
        );
        client
            .create_schema(&self.catalog_name, &self.schema_name)
            .with_comment("Governance test schema".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create schema: {}", e))
            })?;

        // Step 3: Create storage credential
        println!(
            "  🔑 [3/5] Creating storage credential '{}'",
            self.credential_name
        );
        client
            .create_credential(&self.credential_name, Purpose::Storage)
            .with_comment("Governance test storage credential".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create credential: {}", e))
            })?;

        // Step 4: Create external location
        println!(
            "  🌍 [4/5] Creating external location '{}'",
            self.external_location_name
        );
        client
            .create_external_location(
                &self.external_location_name,
                &table_location,
                &self.credential_name,
            )
            .with_comment("Governance test external location".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!(
                    "Failed to create external location: {}",
                    e
                ))
            })?;

        // Step 5: Create external table
        println!("  🗃️  [5/5] Creating external table '{}'", full_name);
        let table = client
            .create_table(
                &self.table_name,
                &self.schema_name,
                &self.catalog_name,
                TableType::External,
                DataSourceFormat::Delta,
            )
            .with_storage_location(table_location.clone())
            .with_comment("Governance test external table".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create external table: {}", e))
            })?;
        assert_eq!(table.name, self.table_name);
        println!("  ✓ Full governance chain established: {}", table.full_name);

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
        let _ = client
            .external_location(&self.external_location_name)
            .delete()
            .await;
        let _ = client.credential(&self.credential_name).delete().await;
        Ok(())
    }
}
