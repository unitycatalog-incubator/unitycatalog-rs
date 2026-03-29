//! Table Managed Lifecycle Journey
//!
//! Tests the full lifecycle of a managed Delta table:
//! create catalog → create schema → create managed DELTA table → get → list → delete

use async_trait::async_trait;
use futures::StreamExt;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::tables::v1::{DataSourceFormat, TableType};

use crate::execution::{
    ImplementationTag, JourneyMetadata, JourneyState, JourneyTier, ResourceTag, UserJourney,
};
use crate::{AcceptanceError, AcceptanceResult};

pub struct TableManagedLifecycleJourney {
    catalog_name: String,
    schema_name: String,
    table_name: String,
}

impl TableManagedLifecycleJourney {
    pub fn new() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            catalog_name: format!("table_lc_catalog_{}", timestamp),
            schema_name: format!("table_lc_schema_{}", timestamp),
            table_name: format!("table_lc_{}", timestamp),
        }
    }
}

impl Default for TableManagedLifecycleJourney {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserJourney for TableManagedLifecycleJourney {
    fn name(&self) -> &str {
        "table_managed_lifecycle"
    }

    fn description(&self) -> &str {
        "Managed table lifecycle: create catalog+schema, create MANAGED DELTA table, get, list, delete"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![
                ResourceTag::Catalogs,
                ResourceTag::Schemas,
                ResourceTag::Tables,
            ],
            implementations: vec![ImplementationTag::All],
            tier: JourneyTier::Tier1Crud,
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

        // Step 1: Create catalog
        println!("  📁 Creating catalog '{}'", self.catalog_name);
        client
            .create_catalog(&self.catalog_name)
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create catalog: {}", e))
            })?;

        // Step 2: Create schema
        println!(
            "  📂 Creating schema '{}.{}'",
            self.catalog_name, self.schema_name
        );
        client
            .create_schema(&self.catalog_name, &self.schema_name)
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create schema: {}", e))
            })?;

        // Step 3: Create managed Delta table
        println!("  🗃️  Creating managed table '{}'", full_name);
        let table = client
            .create_table(
                &self.table_name,
                &self.schema_name,
                &self.catalog_name,
                TableType::Managed,
                DataSourceFormat::Delta,
            )
            .with_comment("Managed table lifecycle test".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create table: {}", e))
            })?;

        assert_eq!(table.name, self.table_name);
        println!("  ✓ Table created: {}", table.full_name);

        // Step 4: Get table
        let fetched = client.table(&full_name).get().await.map_err(|e| {
            AcceptanceError::JourneyExecution(format!("Failed to get table: {}", e))
        })?;
        assert_eq!(fetched.name, self.table_name);
        println!("  ✓ Table fetched: {}", fetched.full_name);

        // Step 5: List tables
        let tables: Vec<_> = client
            .list_tables(&self.catalog_name, &self.schema_name)
            .into_stream()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to list tables: {}", e))
            })?;
        assert!(
            tables.iter().any(|t| t.name == self.table_name),
            "Created table not found in list"
        );
        println!("  ✓ Listed {} table(s)", tables.len());

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
