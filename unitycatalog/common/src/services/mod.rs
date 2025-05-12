use std::sync::Arc;

use delta_kernel_datafusion::{TableSnapshot, Version};

use self::kernel::TableManager;
use crate::api::{RequestContext, SharingQueryHandler};
use crate::models::sharing::v1::*;
use crate::models::tables::v1::{DataSourceFormat, TableInfo};
use crate::resources::ResourceStore;
use crate::{ProvidesResourceStore, ResourceIdent, ResourceName, Result, ShareInfo};

pub mod kernel;
mod location;
mod object_store;
pub mod policy;
pub mod secrets;
pub mod session;

pub use location::*;
pub use policy::*;
pub use secrets::*;
pub use session::*;

#[derive(Clone)]
pub struct ServerHandler {
    handler: Arc<ServerHandlerInner>,
    session: Arc<KernelSession>,
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
        let session = Arc::new(KernelSession::new(handler.clone()));
        Ok(Self { handler, session })
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
impl TableManager for ServerHandler {
    async fn read_snapshot(
        &self,
        location: &StorageLocationUrl,
        format: &DataSourceFormat,
        version: Option<Version>,
    ) -> Result<Arc<dyn TableSnapshot>> {
        self.session.read_snapshot(location, format, version).await
    }
}

#[async_trait::async_trait]
trait SharingExt {
    async fn get_snapshot(
        &self,
        share: &str,
        schema: &str,
        table: &str,
    ) -> Result<Arc<dyn TableSnapshot>>;
}

#[async_trait::async_trait]
impl<T: TableManager + ResourceStore> SharingExt for T {
    async fn get_snapshot(
        &self,
        share: &str,
        schema: &str,
        table: &str,
    ) -> Result<Arc<dyn TableSnapshot>> {
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
        let location = StorageLocationUrl::parse(&location)?;
        self.read_snapshot(&location, &DataSourceFormat::Delta, None)
            .await
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

    async fn query_table(
        &self,
        request: QueryTableRequest,
        context: RequestContext,
    ) -> Result<QueryResponse> {
        todo!()
    }
}
