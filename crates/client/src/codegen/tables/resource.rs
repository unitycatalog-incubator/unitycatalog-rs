// @generated — do not edit by hand.
use super::builders::*;
use super::client::TableServiceClient;
/// A client scoped to a single `table`.
#[derive(Clone)]
pub struct TableClient {
    pub(crate) catalog_name: String,
    pub(crate) schema_name: String,
    pub(crate) table_name: String,
    pub(crate) client: TableServiceClient,
}
impl TableClient {
    /// Create a client bound to the resource's name components.
    pub fn new(
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        table_name: impl Into<String>,
        client: TableServiceClient,
    ) -> Self {
        Self {
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            table_name: table_name.into(),
            client,
        }
    }
    /// Create a `table` client from its dot-joined full name (e.g. `"catalog_name.schema_name.table_name"`).
    pub fn from_full_name(full_name: impl Into<String>, client: TableServiceClient) -> Self {
        let full_name = full_name.into();
        let mut parts = full_name.splitn(3usize, '.');
        let catalog_name = parts.next().unwrap_or_default();
        let schema_name = parts.next().unwrap_or_default();
        let table_name = parts.next().unwrap_or_default();
        Self::new(catalog_name, schema_name, table_name, client)
    }
    /// Get a table
    pub fn get(&self) -> GetTableBuilder {
        GetTableBuilder::new(
            self.client.clone(),
            format!(
                "{}.{}.{}",
                self.catalog_name, self.schema_name, self.table_name
            ),
        )
    }
    /// Get boolean reflecting if table exists
    pub fn get_table_exists(&self) -> GetTableExistsBuilder {
        GetTableExistsBuilder::new(
            self.client.clone(),
            format!(
                "{}.{}.{}",
                self.catalog_name, self.schema_name, self.table_name
            ),
        )
    }
    /// Delete a table
    pub fn delete(&self) -> DeleteTableBuilder {
        DeleteTableBuilder::new(
            self.client.clone(),
            format!(
                "{}.{}.{}",
                self.catalog_name, self.schema_name, self.table_name
            ),
        )
    }
}
