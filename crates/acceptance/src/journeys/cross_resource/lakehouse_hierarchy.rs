//! Lakehouse Hierarchy Journey
//!
//! Tests a realistic multi-resource lakehouse setup:
//! catalog → 2 schemas → managed tables in each → managed volumes → list all → verify
//!
//! NOTE: Pending recording against managed Databricks UC.
//! Run with UC_INTEGRATION_RECORD=true to record.

use async_trait::async_trait;
use futures::StreamExt;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::models::volumes::v1::VolumeType;
use unitycatalog_common::tables::v1::{DataSourceFormat, TableType};

use crate::execution::{
    ImplementationTag, JourneyMetadata, JourneyState, JourneyTier, ResourceTag, UserJourney,
};
use crate::{AcceptanceError, AcceptanceResult};

pub struct LakehouseHierarchyJourney {
    catalog_name: String,
    schema_names: Vec<String>,
}

impl LakehouseHierarchyJourney {
    pub fn new() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            catalog_name: format!("lakehouse_catalog_{}", timestamp),
            schema_names: vec![
                format!("bronze_{}", timestamp),
                format!("silver_{}", timestamp),
            ],
        }
    }
}

impl Default for LakehouseHierarchyJourney {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserJourney for LakehouseHierarchyJourney {
    fn name(&self) -> &str {
        "lakehouse_hierarchy"
    }

    fn description(&self) -> &str {
        "Lakehouse hierarchy: catalog → 2 schemas (bronze/silver) → managed tables + volumes in each"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![
                ResourceTag::Catalogs,
                ResourceTag::Schemas,
                ResourceTag::Tables,
                ResourceTag::Volumes,
            ],
            implementations: vec![ImplementationTag::All],
            tier: JourneyTier::Tier4Advanced,
            requires_external_storage: false,
        }
    }

    fn save_state(&self) -> AcceptanceResult<JourneyState> {
        let mut state = JourneyState::empty();
        state.set_string("catalog_name", self.catalog_name.clone());
        let schema_json = serde_json::to_string(&self.schema_names)
            .map_err(crate::AcceptanceError::JsonParsing)?;
        state.set_string("schema_names", schema_json);
        Ok(state)
    }

    fn load_state(&mut self, state: &JourneyState) -> AcceptanceResult<()> {
        if let Some(v) = state.get_string("catalog_name") {
            self.catalog_name = v;
        }
        if let Some(json) = state.get_string("schema_names") {
            self.schema_names = serde_json::from_str(&json)?;
        }
        Ok(())
    }

    async fn execute(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        // Step 1: Create catalog
        println!("  📁 Creating catalog '{}'", self.catalog_name);
        client
            .create_catalog(&self.catalog_name)
            .with_comment("Lakehouse hierarchy test".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create catalog: {}", e))
            })?;

        // Step 2: Create schemas and in each create a table + volume
        for schema_name in &self.schema_names {
            println!(
                "  📂 Creating schema '{}.{}'",
                self.catalog_name, schema_name
            );
            client
                .create_schema(&self.catalog_name, schema_name)
                .with_comment(format!("{} layer", schema_name))
                .await
                .map_err(|e| {
                    AcceptanceError::JourneyExecution(format!("Failed to create schema: {}", e))
                })?;

            let table_name = format!("{}_events", schema_name);
            let volume_name = format!("{}_files", schema_name);

            // Create managed table
            client
                .create_table(
                    &table_name,
                    schema_name,
                    &self.catalog_name,
                    TableType::Managed,
                    DataSourceFormat::Delta,
                )
                .with_comment(format!("{} events table", schema_name))
                .await
                .map_err(|e| {
                    AcceptanceError::JourneyExecution(format!("Failed to create table: {}", e))
                })?;
            println!("    ✓ Table '{}'", table_name);

            // Create managed volume
            client
                .create_volume(
                    &self.catalog_name,
                    schema_name,
                    &volume_name,
                    VolumeType::Managed,
                )
                .with_comment(format!("{} files volume", schema_name))
                .await
                .map_err(|e| {
                    AcceptanceError::JourneyExecution(format!("Failed to create volume: {}", e))
                })?;
            println!("    ✓ Volume '{}'", volume_name);
        }

        // Step 3: Verify — list schemas and check count
        let schemas: Vec<_> = client
            .list_schemas(&self.catalog_name)
            .into_stream()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to list schemas: {}", e))
            })?;
        assert_eq!(
            schemas.len(),
            self.schema_names.len(),
            "Unexpected number of schemas"
        );
        println!("  ✓ Verified {} schemas", schemas.len());

        // Verify tables and volumes in each schema
        for schema_name in &self.schema_names {
            let tables: Vec<_> = client
                .list_tables(&self.catalog_name, schema_name)
                .into_stream()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| {
                    AcceptanceError::JourneyExecution(format!("Failed to list tables: {}", e))
                })?;
            assert_eq!(tables.len(), 1, "Expected 1 table in {}", schema_name);

            let volumes: Vec<_> = client
                .list_volumes(&self.catalog_name, schema_name)
                .into_stream()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| {
                    AcceptanceError::JourneyExecution(format!("Failed to list volumes: {}", e))
                })?;
            assert_eq!(volumes.len(), 1, "Expected 1 volume in {}", schema_name);
        }
        println!("  ✓ All tables and volumes verified");

        Ok(())
    }

    async fn cleanup(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        for schema_name in &self.schema_names {
            let table_name = format!("{}_events", schema_name);
            let volume_name = format!("{}_files", schema_name);
            let full_table = format!("{}.{}.{}", self.catalog_name, schema_name, table_name);

            let _ = client.table(&full_table).delete().await;
            let _ = client
                .volume(&self.catalog_name, schema_name, &volume_name)
                .delete()
                .await;
            let _ = client
                .schema(&self.catalog_name, schema_name)
                .delete()
                .await;
        }
        let _ = client.catalog(&self.catalog_name).delete().await;
        Ok(())
    }
}
