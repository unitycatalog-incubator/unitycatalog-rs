//! Function (UDF) Lifecycle Journey
//!
//! Tests the full lifecycle of a SQL user-defined function:
//! create catalog+schema → create SQL UDF → get → list → delete
//!
//! NOTE: Pending recording against managed Databricks UC.
//! Run with UC_INTEGRATION_RECORD=true to record.

use async_trait::async_trait;
use futures::StreamExt;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::models::functions::v1::{
    ParameterStyle, RoutineBody, SecurityType, SqlDataAccess,
};

use crate::execution::{
    ImplementationTag, JourneyMetadata, JourneyState, JourneyTier, ResourceTag, UserJourney,
};
use crate::{AcceptanceError, AcceptanceResult};

pub struct FunctionLifecycleJourney {
    catalog_name: String,
    schema_name: String,
    function_name: String,
}

impl FunctionLifecycleJourney {
    pub fn new() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            catalog_name: format!("fn_lc_catalog_{}", timestamp),
            schema_name: format!("fn_lc_schema_{}", timestamp),
            function_name: format!("fn_lc_{}", timestamp),
        }
    }
}

impl Default for FunctionLifecycleJourney {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserJourney for FunctionLifecycleJourney {
    fn name(&self) -> &str {
        "function_lifecycle"
    }

    fn description(&self) -> &str {
        "Function lifecycle: create catalog+schema, create SQL UDF (returns int), get, list, delete"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![
                ResourceTag::Functions,
                ResourceTag::Catalogs,
                ResourceTag::Schemas,
            ],
            implementations: vec![
                ImplementationTag::OssRust,
                ImplementationTag::ManagedDatabricks,
            ],
            tier: JourneyTier::Tier4Advanced,
            requires_external_storage: false,
        }
    }

    fn save_state(&self) -> AcceptanceResult<JourneyState> {
        let mut state = JourneyState::empty();
        state.set_string("catalog_name", self.catalog_name.clone());
        state.set_string("schema_name", self.schema_name.clone());
        state.set_string("function_name", self.function_name.clone());
        Ok(state)
    }

    fn load_state(&mut self, state: &JourneyState) -> AcceptanceResult<()> {
        if let Some(v) = state.get_string("catalog_name") {
            self.catalog_name = v;
        }
        if let Some(v) = state.get_string("schema_name") {
            self.schema_name = v;
        }
        if let Some(v) = state.get_string("function_name") {
            self.function_name = v;
        }
        Ok(())
    }

    async fn execute(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        // Step 1: Create catalog and schema
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

        // Step 2: Create a simple SQL scalar UDF that returns INT
        println!(
            "  ⚙️  Creating SQL UDF '{}.{}.{}'",
            self.catalog_name, self.schema_name, self.function_name
        );
        let function = client
            .create_function(
                &self.function_name,
                &self.catalog_name,
                &self.schema_name,
                "INT",
                "INT",
                ParameterStyle::S,
                true,
                SqlDataAccess::ContainsSql,
                true,
                SecurityType::Definer,
                RoutineBody::Sql,
            )
            .with_routine_definition("SELECT 42".to_string())
            .with_comment("Function lifecycle test: returns 42".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create function: {}", e))
            })?;
        assert_eq!(function.name, self.function_name);
        println!("  ✓ Function created: {}", function.full_name);

        // Step 3: Get function
        let fetched = client
            .function(&self.catalog_name, &self.schema_name, &self.function_name)
            .get()
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to get function: {}", e))
            })?;
        assert_eq!(fetched.name, self.function_name);
        println!("  ✓ Function fetched: {}", fetched.full_name);

        // Step 4: List functions
        let functions: Vec<_> = client
            .list_functions(&self.catalog_name, &self.schema_name)
            .into_stream()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to list functions: {}", e))
            })?;
        assert!(
            functions.iter().any(|f| f.name == self.function_name),
            "Created function not found in list"
        );
        println!("  ✓ Listed {} function(s)", functions.len());

        Ok(())
    }

    async fn cleanup(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        let _ = client
            .function(&self.catalog_name, &self.schema_name, &self.function_name)
            .delete()
            .await;
        let _ = client
            .schema(&self.catalog_name, &self.schema_name)
            .delete()
            .await;
        let _ = client.catalog(&self.catalog_name).delete().await;
        Ok(())
    }
}
