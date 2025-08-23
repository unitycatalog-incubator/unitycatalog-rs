use std::sync::Arc;

use delta_kernel::{Snapshot, Version};

use session::*;
use unitycatalog_common::models::tables::v1::DataSourceFormat;

use self::location::StorageLocationUrl;
use self::secrets::{ProvidesSecretManager, SecretManager};
use crate::Result;
use crate::api::tables::TableManager;
use crate::policy::{Policy, ProvidesPolicy};
use crate::store::{ProvidesResourceStore, ResourceStore};

pub mod location;
mod object_store;
pub mod secrets;
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
