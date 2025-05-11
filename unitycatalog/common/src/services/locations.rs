use crate::error::Result;
use crate::models::tables::v1::TableInfo;
use crate::resources::ResourceRef;

#[async_trait::async_trait]
pub trait TableInfoResolver: Send + Sync {
    /// Read table info from the storage location.
    async fn read_table_info(&self, table: &ResourceRef) -> Result<TableInfo>;
}
