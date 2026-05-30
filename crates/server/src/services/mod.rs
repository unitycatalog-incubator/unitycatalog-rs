use std::sync::Arc;

use delta_kernel::{Snapshot, Version};

use session::*;
use unitycatalog_common::models::tables::v1::DataSourceFormat;

use self::location::StorageLocationUrl;
use self::secrets::{ProvidesSecretManager, SecretManager};
use crate::Result;
use crate::api::tables::TableManager;
use crate::policy::{Decision, Permission, Policy, ProvidesPolicy};
use crate::store::{ProvidesObjectStore, ProvidesResourceStore, ResourceStore};
use unitycatalog_common::ObjectLabel;
use unitycatalog_common::models::ResourceIdent;

pub mod credential_vending;
pub(crate) mod kernel;
pub mod location;
pub(crate) mod object_store;
pub mod secrets;
mod session;
mod sharing;

#[derive(Clone)]
pub struct ServerHandler<Cx> {
    handler: Arc<ServerHandlerInner<Cx>>,
    session: Arc<KernelSession>,
}

impl<Cx: Send + Sync + 'static> ServerHandler<Cx>
where
    ServerHandlerInner<Cx>: kernel::ObjectStoreFactory,
{
    pub fn try_new_tokio(
        policy: Arc<dyn Policy<Cx>>,
        store: Arc<dyn ResourceStore>,
        secrets: Arc<dyn SecretManager>,
    ) -> Result<Self> {
        let handler = Arc::new(ServerHandlerInner::new(
            policy.clone(),
            store.clone(),
            secrets.clone(),
        ));
        let session = Arc::new(KernelSession::new(handler.clone())?);
        Ok(Self { handler, session })
    }
}

#[derive(Clone)]
pub struct ServerHandlerInner<Cx> {
    policy: Arc<dyn Policy<Cx>>,
    store: Arc<dyn ResourceStore>,
    object_store: Option<Arc<dyn olai_store::ObjectStore<ObjectLabel>>>,
    secrets: Arc<dyn SecretManager>,
}

impl<Cx: Send + Sync + 'static> ServerHandlerInner<Cx> {
    pub fn new(
        policy: Arc<dyn Policy<Cx>>,
        store: Arc<dyn ResourceStore>,
        secrets: Arc<dyn SecretManager>,
    ) -> Self {
        Self {
            policy,
            store,
            object_store: None,
            secrets,
        }
    }

    /// Set the generic object store.
    ///
    /// When provided, the server exposes the untyped `ObjectStore<ObjectLabel>`
    /// interface alongside the typed `ResourceStore` interface.
    pub fn with_object_store(
        mut self,
        object_store: Arc<dyn olai_store::ObjectStore<ObjectLabel>>,
    ) -> Self {
        self.object_store = Some(object_store);
        self
    }
}

impl<Cx: Send + Sync + 'static> ProvidesPolicy<Cx> for ServerHandlerInner<Cx> {
    fn policy(&self) -> &Arc<dyn Policy<Cx>> {
        &self.policy
    }
}

impl<Cx: Send + Sync + 'static> ProvidesPolicy<Cx> for ServerHandler<Cx> {
    fn policy(&self) -> &Arc<dyn Policy<Cx>> {
        &self.handler.policy
    }
}

#[async_trait::async_trait]
impl<Cx: Send + Sync + 'static> Policy<Cx> for ServerHandlerInner<Cx> {
    async fn authorize(
        &self,
        resource: &ResourceIdent,
        permission: &Permission,
        context: &Cx,
    ) -> Result<Decision> {
        self.policy().authorize(resource, permission, context).await
    }

    async fn authorize_many(
        &self,
        resources: &[ResourceIdent],
        permission: &Permission,
        context: &Cx,
    ) -> Result<Vec<Decision>> {
        self.policy()
            .authorize_many(resources, permission, context)
            .await
    }
}

#[async_trait::async_trait]
impl<Cx: Send + Sync + 'static> Policy<Cx> for ServerHandler<Cx> {
    async fn authorize(
        &self,
        resource: &ResourceIdent,
        permission: &Permission,
        context: &Cx,
    ) -> Result<Decision> {
        self.handler
            .policy
            .authorize(resource, permission, context)
            .await
    }

    async fn authorize_many(
        &self,
        resources: &[ResourceIdent],
        permission: &Permission,
        context: &Cx,
    ) -> Result<Vec<Decision>> {
        self.handler
            .policy
            .authorize_many(resources, permission, context)
            .await
    }
}

impl<Cx: Send + Sync + 'static> ProvidesResourceStore for ServerHandlerInner<Cx> {
    fn store(&self) -> &dyn ResourceStore {
        self.store.as_ref()
    }
}

impl<Cx: Send + Sync + 'static> ProvidesResourceStore for ServerHandler<Cx> {
    fn store(&self) -> &dyn ResourceStore {
        self.handler.store.as_ref()
    }
}

impl<Cx: Send + Sync + 'static> ProvidesObjectStore for ServerHandlerInner<Cx> {
    fn object_store(&self) -> &dyn olai_store::ObjectStore<ObjectLabel> {
        self.object_store
            .as_ref()
            .expect("ObjectStore not configured on ServerHandler")
            .as_ref()
    }
}

impl<Cx: Send + Sync + 'static> ProvidesObjectStore for ServerHandler<Cx> {
    fn object_store(&self) -> &dyn olai_store::ObjectStore<ObjectLabel> {
        self.handler.object_store()
    }
}

impl<Cx: Send + Sync + 'static> ProvidesSecretManager for ServerHandlerInner<Cx> {
    fn secret_manager(&self) -> &dyn SecretManager {
        self.secrets.as_ref()
    }
}

impl<Cx: Send + Sync + 'static> ProvidesSecretManager for ServerHandler<Cx> {
    fn secret_manager(&self) -> &dyn SecretManager {
        self.handler.secrets.as_ref()
    }
}

#[async_trait::async_trait]
impl<Cx: Send + Sync + 'static> TableManager for ServerHandler<Cx> {
    async fn read_snapshot(
        &self,
        location: &StorageLocationUrl,
        format: &DataSourceFormat,
        version: Option<Version>,
    ) -> Result<Arc<Snapshot>> {
        self.session.read_snapshot(location, format, version).await
    }
}
