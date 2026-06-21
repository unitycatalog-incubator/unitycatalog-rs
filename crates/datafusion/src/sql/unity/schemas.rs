use datafusion::arrow::array::RecordBatch;
use datafusion::common::{DataFusionError, Result};
use datafusion::sql::sqlparser::ast::{ObjectName, Value};
use unitycatalog_client::UnityCatalogClient;
use url::Url;

use crate::sql::unity::{create_response_to_batch, drop_response_to_batch};

/// Split a schema [`ObjectName`] into `(catalog, schema)`.
///
/// Unity Catalog schemas are always two-part (`catalog.schema`); unlike a
/// Databricks SQL session, hydrofoil has no notion of a "current catalog" to
/// resolve a bare schema name against, so a one-part name is rejected with a
/// clear error directing the caller to qualify it.
fn split_schema_name(name: &ObjectName) -> Result<(String, String)> {
    match name.0.as_slice() {
        [catalog, schema] => Ok((catalog.to_string(), schema.to_string())),
        [_] => Err(DataFusionError::Execution(format!(
            "Schema name '{name}' must be qualified with a catalog (<catalog>.<schema>)"
        ))),
        _ => Err(DataFusionError::Execution(format!(
            "Expected schema name to be a two-part identifier (<catalog>.<schema>), got '{name}'"
        ))),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct CreateSchemaStatement {
    pub name: ObjectName,
    pub if_not_exists: bool,
    pub managed_location: Option<Url>,
    pub comment: Option<String>,
    pub properties: Option<Vec<(String, Value)>>,
}

impl CreateSchemaStatement {
    pub(crate) async fn execute(&self, client: UnityCatalogClient) -> Result<RecordBatch> {
        let (catalog, schema) = split_schema_name(&self.name)?;

        // The UC `CreateSchema` API has no managed-location field (unlike
        // catalogs), so `MANAGED LOCATION` is not supported for schemas yet.
        if self.managed_location.is_some() {
            return Err(DataFusionError::NotImplemented(
                "MANAGED LOCATION is not supported for CREATE SCHEMA".to_string(),
            ));
        }

        let mut request = client
            .create_schema(&catalog, &schema)
            .with_comment(self.comment.clone());
        if let Some(properties) = self.properties.as_ref() {
            request = request.with_properties(
                properties
                    .iter()
                    .map(|(k, v)| (k.clone(), value_to_string(v))),
            );
        }
        let schema_info = request
            .await
            .map_err(|e| DataFusionError::External(Box::new(e)))?;
        create_response_to_batch(self.name.to_string(), "Schema", schema_info)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct DropSchemaStatement {
    pub name: ObjectName,
    pub if_exists: bool,
    pub cascade: bool,
}

impl DropSchemaStatement {
    pub(crate) async fn execute(&self, client: UnityCatalogClient) -> Result<RecordBatch> {
        let (catalog, schema) = split_schema_name(&self.name)?;
        client
            .schema(&catalog, &schema)
            .delete()
            .with_force(self.cascade)
            .await
            .map_err(|e| DataFusionError::External(Box::new(e)))?;
        drop_response_to_batch(self.name.to_string(), "Schema", "success")
    }
}

/// Render a parsed option [`Value`] as the plain string UC properties expect.
fn value_to_string(value: &Value) -> String {
    match value {
        Value::SingleQuotedString(s)
        | Value::DoubleQuotedString(s)
        | Value::EscapedStringLiteral(s) => s.clone(),
        Value::Number(n, _) => n.clone(),
        other => other.to_string(),
    }
}
