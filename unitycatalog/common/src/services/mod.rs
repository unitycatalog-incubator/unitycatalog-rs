use std::sync::Arc;

use delta_kernel::{Engine, Table};

use crate::api::{RequestContext, SharingQueryHandler};
use crate::kernel::KernelEngineFactroy;
use crate::models::sharing::v1::*;
use crate::resources::ResourceStore;
use crate::{ProvidesResourceStore, Resource, ResourceIdent, ResourceRef, Result};

pub mod locations;
pub mod policy;
pub mod secrets;

pub use locations::*;
pub use policy::*;
pub use secrets::*;

#[derive(Clone)]
pub struct ServerHandler {
    pub policy: Arc<dyn Policy>,
    pub store: Arc<dyn ResourceStore>,
    pub query: Arc<dyn SharingQueryHandler>,
    pub secrets: Arc<dyn SecretManager>,
}

impl ServerHandler {
    pub fn new(
        policy: Arc<dyn Policy>,
        store: Arc<dyn ResourceStore>,
        query: Arc<dyn SharingQueryHandler>,
        secrets: Arc<dyn SecretManager>,
    ) -> Self {
        Self {
            policy,
            store,
            query,
            secrets,
        }
    }
}

impl ProvidesPolicy for ServerHandler {
    fn policy(&self) -> &Arc<dyn Policy> {
        &self.policy
    }
}

impl ProvidesResourceStore for ServerHandler {
    fn store(&self) -> &dyn ResourceStore {
        self.store.as_ref()
    }
}

impl ProvidesSecretManager for ServerHandler {
    fn secret_manager(&self) -> &dyn SecretManager {
        self.secrets.as_ref()
    }
}

#[async_trait::async_trait]
impl TableLocationResolver for ServerHandler {
    async fn resolve(&self, table: &ResourceRef) -> Result<url::Url> {
        let (table, _) = self
            .store()
            .get(&ResourceIdent::Table(table.clone()))
            .await?;
        let table = match table {
            Resource::TableInfo(t) => t,
            _ => return Err(crate::Error::NotFound),
        };
        table
            .storage_location
            .as_ref()
            .ok_or(crate::Error::NotFound)
            .and_then(|l| Ok(url::Url::parse(l)?))
    }
}

#[async_trait::async_trait]
impl KernelEngineFactroy for ServerHandler {
    async fn create(&self, _table: &Table) -> Result<Arc<dyn Engine>> {
        todo!("create engine")
    }
}

#[async_trait::async_trait]
impl SharingQueryHandler for ServerHandler {
    async fn get_table_version(
        &self,
        request: GetTableVersionRequest,
        context: RequestContext,
    ) -> Result<GetTableVersionResponse> {
        self.check_required(&request, context.recipient()).await?;
        self.query.get_table_version(request, context).await
    }

    async fn get_table_metadata(
        &self,
        request: GetTableMetadataRequest,
        context: RequestContext,
    ) -> Result<QueryResponse> {
        self.check_required(&request, context.recipient()).await?;
        self.query.get_table_metadata(request, context).await
    }
}
