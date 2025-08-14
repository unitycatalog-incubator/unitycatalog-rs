use std::sync::Arc;

use delta_kernel::{Snapshot, Version};

use session::*;
use unitycatalog_common::api::tables::TableManager;
use unitycatalog_common::models::tables::v1::DataSourceFormat;
use unitycatalog_common::resources::ResourceStore;
use unitycatalog_common::services::{
    Policy, ProvidesPolicy, ProvidesSecretManager, SecretManager, StorageLocationUrl,
};
use unitycatalog_common::{ProvidesResourceStore, Result};

mod object_store;
mod session;
mod sharing;

#[derive(Clone)]
pub struct ServerHandler {
    handler: Arc<ServerHandlerInner>,
    session: Arc<KernelSession>,
}

impl ServerHandler {
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
        let session = Arc::new(KernelSession::new(handler.clone())?);
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
    ) -> Result<Arc<Snapshot>> {
        self.session.read_snapshot(location, format, version).await
    }
}
