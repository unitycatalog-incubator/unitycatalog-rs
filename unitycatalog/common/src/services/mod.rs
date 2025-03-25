use std::sync::Arc;

use delta_kernel::Engine;
use delta_kernel::snapshot::Snapshot;

use self::kernel::{ProvidesEngine, TableManager};
use crate::api::{RequestContext, SharingQueryHandler};
use crate::models::sharing::v1::*;
use crate::models::tables::v1::{DataSourceFormat, TableInfo};
use crate::resources::ResourceStore;
use crate::{
    ProvidesResourceStore, Resource, ResourceIdent, ResourceName, ResourceRef, Result, ShareInfo,
};

pub mod kernel;
pub mod locations;
pub mod policy;
pub mod secrets;

pub use locations::*;
pub use policy::*;
pub use secrets::*;

#[derive(Clone)]
pub struct ServerHandler {
    handler: Arc<ServerHandlerInner>,
    engine: Arc<dyn Engine>,
}

impl ServerHandler {
    #[cfg(feature = "tokio")]
    pub fn try_new_tokio(
        policy: Arc<dyn Policy>,
        store: Arc<dyn ResourceStore>,
        secrets: Arc<dyn SecretManager>,
    ) -> Result<Self> {
        use delta_kernel::engine::default::executor::tokio::TokioBackgroundExecutor;
        use delta_kernel::engine::default::executor::tokio::TokioMultiThreadExecutor;

        let handler = Arc::new(ServerHandlerInner::new(
            policy.clone(),
            store.clone(),
            secrets.clone(),
        ));

        let handle = tokio::runtime::Handle::try_current()
            .map_err(|e| crate::Error::generic(e.to_string()))?;
        let engine: Arc<dyn Engine> = match handle.runtime_flavor() {
            tokio::runtime::RuntimeFlavor::MultiThread => kernel::engine::get_engine(
                handler.clone(),
                Arc::new(TokioMultiThreadExecutor::new(handle)),
            )?,
            tokio::runtime::RuntimeFlavor::CurrentThread => kernel::engine::get_engine(
                handler.clone(),
                Arc::new(TokioBackgroundExecutor::new()),
            )?,
            _ => {
                return Err(crate::Error::generic("Unsupported runtime flavor"));
            }
        };
        Ok(Self { handler, engine })
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

impl ProvidesEngine for ServerHandler {
    fn engine(&self) -> &dyn Engine {
        self.engine.as_ref()
    }
}

#[async_trait::async_trait]
impl<T: ResourceStore> TableLocationResolver for T {
    async fn resolve_location(&self, table: &ResourceRef) -> Result<url::Url> {
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
trait SharingExt {
    async fn get_snapshot(&self, share: &str, schema: &str, table: &str) -> Result<Snapshot>;
}

#[async_trait::async_trait]
impl<T: TableManager + ResourceStore> SharingExt for T {
    async fn get_snapshot(&self, share: &str, schema: &str, table: &str) -> Result<Snapshot> {
        let share_ident = ResourceIdent::share(ResourceName::new([share]));
        let share_info: ShareInfo = self.get(&share_ident).await?.0.try_into()?;
        let Some(table_object) = share_info
            .data_objects
            .iter()
            .find(|o| o.shared_as() == &format!("{}.{}", schema, table))
        else {
            return Err(crate::Error::NotFound);
        };
        let table_ident = ResourceIdent::table(ResourceName::new(table_object.name.split(".")));
        let table_info: TableInfo = self.get(&table_ident).await?.0.try_into()?;
        let location = table_info.storage_location.ok_or(crate::Error::NotFound)?;
        let location = url::Url::parse(&location)?;
        self.read_snapshot(&location, &DataSourceFormat::Delta, None)
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
        let snapshot = self
            .get_snapshot(&request.share, &request.schema, &request.name)
            .await?;
        Ok(GetTableVersionResponse {
            version: snapshot.version() as i64,
        })
    }

    async fn get_table_metadata(
        &self,
        request: GetTableMetadataRequest,
        context: RequestContext,
    ) -> Result<QueryResponse> {
        self.check_required(&request, context.recipient()).await?;
        let snapshot = self
            .get_snapshot(&request.share, &request.schema, &request.name)
            .await?;
        Ok([snapshot.metadata().into(), snapshot.protocol().into()].into())
    }
}
