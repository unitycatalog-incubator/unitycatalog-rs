use url::Url;

use crate::error::Result;
use crate::models::tables::v1::TableInfo;
use crate::resources::ResourceRef;

/// Resolver for the storage location of a table.
#[async_trait::async_trait]
pub trait TableLocationResolver: Send + Sync {
    /// Resolve the storage location of a table.
    async fn resolve_location(&self, table: &ResourceRef) -> Result<Url>;
}

#[async_trait::async_trait]
pub trait TableInfoResolver: Send + Sync {
    /// Read table info from the storage location.
    async fn read_table_info(&self, table: &ResourceRef) -> Result<TableInfo>;
}
