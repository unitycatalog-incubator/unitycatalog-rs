use std::sync::Arc;

use delta_kernel::{Snapshot, Version};

use session::*;
use unitycatalog_common::models::tables::v1::DataSourceFormat;

use self::location::StorageLocationUrl;
use self::secrets::{ProvidesSecretManager, SecretManager};
use crate::Result;
use crate::api::tables::TableManager;
use crate::policy::{Decision, Permission, Policy, ProvidesPolicy};
use crate::store::{ProvidesResourceStore, ResourceStore};
use unitycatalog_common::models::ResourceIdent;

pub mod location;
mod object_store;
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
    ServerHandlerInner<Cx>: deltalake_datafusion::ObjectStoreFactory,
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
            secrets,
        }
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
