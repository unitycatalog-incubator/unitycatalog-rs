use std::sync::Arc;

use crate::api::{RequestContext, SharingQueryHandler};
use crate::kernel::KernelQueryHandler;
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
    handler: Arc<ServerHandlerInner>,
    query: Arc<dyn SharingQueryHandler>,
}

impl ServerHandler {
    #[cfg(feature = "tokio")]
    pub fn try_new_tokio(
        policy: Arc<dyn Policy>,
        store: Arc<dyn ResourceStore>,
        secrets: Arc<dyn SecretManager>,
    ) -> Result<Self> {
        let handler = Arc::new(ServerHandlerInner::new(
            policy.clone(),
            store.clone(),
            secrets.clone(),
        ));
        let query =
            KernelQueryHandler::try_new_tokio(handler.clone(), handler.clone(), policy.clone())?;
        Ok(Self { handler, query })
    }
}

#[derive(Clone)]
pub struct ServerHandlerInner {
    policy: Arc<dyn Policy>,
    store: Arc<dyn ResourceStore>,
    secrets: Arc<dyn SecretManager>,
}

impl ServerHandlerInner {
    pub fn new(
        policy: Arc<dyn Policy>,
        store: Arc<dyn ResourceStore>,
        secrets: Arc<dyn SecretManager>,
    ) -> Self {
        Self {
            policy,
            store,
            secrets,
        }
    }
}

impl ProvidesPolicy for ServerHandlerInner {
    fn policy(&self) -> &Arc<dyn Policy> {
        &self.policy
    }
}

impl ProvidesPolicy for ServerHandler {
    fn policy(&self) -> &Arc<dyn Policy> {
        &self.handler.policy
    }
}

impl ProvidesResourceStore for ServerHandlerInner {
    fn store(&self) -> &dyn ResourceStore {
        self.store.as_ref()
    }
}

impl ProvidesResourceStore for ServerHandler {
    fn store(&self) -> &dyn ResourceStore {
        self.handler.store.as_ref()
    }
}

impl ProvidesSecretManager for ServerHandlerInner {
    fn secret_manager(&self) -> &dyn SecretManager {
        self.secrets.as_ref()
    }
}

impl ProvidesSecretManager for ServerHandler {
    fn secret_manager(&self) -> &dyn SecretManager {
        self.handler.secrets.as_ref()
    }
}

#[async_trait::async_trait]
impl<T: ResourceStore> TableLocationResolver for T {
    async fn resolve(&self, table: &ResourceRef) -> Result<url::Url> {
        let (table, _) = self.get(&ResourceIdent::Table(table.clone())).await?;
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
