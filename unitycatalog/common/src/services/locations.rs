use crate::error::Result;
use crate::resources::ResourceRef;

/// Resolver for the storage location of a table.
#[async_trait::async_trait]
pub trait TableLocationResolver: Send + Sync {
    async fn resolve(&self, table: &ResourceRef) -> Result<url::Url>;
}
