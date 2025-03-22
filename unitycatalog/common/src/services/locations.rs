use std::sync::Arc;

use crate::error::Result;
use crate::resources::ResourceRef;

/// Resolver for the storage location of a table.
#[async_trait::async_trait]
pub trait TableLocationResolver: Send + Sync {
    async fn resolve(&self, table: &ResourceRef) -> Result<url::Url>;
}

#[async_trait::async_trait]
impl<T: TableLocationResolver> TableLocationResolver for Arc<T> {
    async fn resolve(&self, table: &ResourceRef) -> Result<url::Url> {
        T::resolve(self, table).await
    }
}
