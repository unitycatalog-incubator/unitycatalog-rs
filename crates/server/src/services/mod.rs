use std::sync::Arc;

use delta_kernel::{Snapshot, Version};

use session::*;
use unitycatalog_common::models::tables::v1::DataSourceFormat;

use self::location::StorageLocationUrl;
use self::secrets::{ProvidesSecretManager, SecretManager};
use crate::Result;
use crate::api::tables::{TableHandler, TableManager};
use crate::api::volumes::VolumeHandler;
use crate::policy::{Decision, Permission, Policy, ProvidesPolicy};
use crate::store::{ProvidesObjectStore, ProvidesResourceStore, ResourceStore};
use unitycatalog_common::ObjectLabel;
use unitycatalog_common::models::ResourceIdent;
use unitycatalog_common::services::commit_coordinator::{
    CommitCoordinator, InMemoryCommitCoordinator, ProvidesCommitCoordinator,
};

pub mod credential_vending;
pub(crate) mod kernel;
pub mod location;
pub mod managed_delta_contract;
pub(crate) mod object_store;
pub mod secrets;
mod session;
mod sharing;

#[derive(Clone)]
pub struct ServerHandler<Cx> {
    handler: Arc<ServerHandlerInner<Cx>>,
    session: Arc<KernelSession>,
    /// Optional source for resolving shared *table* primitives.
    ///
    /// Delta Sharing resolves a shared table's storage location by looking up
    /// the underlying Table primitive. In the self-contained topology that
    /// primitive lives in this server's own store, so the default
    /// (`None`) resolves it locally. In the side-by-side topology the Table
    /// primitive lives in an upstream Unity Catalog, so the hybrid wiring
    /// injects an [`UpstreamTableHandler`](crate::handlers::upstream::UpstreamTableHandler)
    /// here and resolution follows the same routing as every other surface.
    table_source: Option<Arc<dyn TableHandler<Cx>>>,
    /// Optional source for resolving shared *volume* primitives.
    ///
    /// The Open Sharing volume and agent-skill surfaces resolve a shared
    /// asset's storage location by looking up the underlying Volume primitive.
    /// As with [`table_source`](Self::table_source), the default (`None`)
    /// resolves it from this server's own store (self-contained topology), while
    /// the hybrid wiring injects an
    /// [`UpstreamVolumeHandler`](crate::handlers::upstream::UpstreamVolumeHandler)
    /// to resolve it from an upstream Unity Catalog (side-by-side topology).
    volume_source: Option<Arc<dyn VolumeHandler<Cx>>>,
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
        Self::try_new_tokio_with_coordinator(
            policy,
            store,
            secrets,
            Arc::new(InMemoryCommitCoordinator::default()),
        )
    }

    /// Construct a handler backed by a specific [`CommitCoordinator`].
    ///
    /// Use this to wire a persistent coordinator (e.g. the Postgres-backed
    /// `GraphStore`) instead of the default in-memory one.
    pub fn try_new_tokio_with_coordinator(
        policy: Arc<dyn Policy<Cx>>,
        store: Arc<dyn ResourceStore>,
        secrets: Arc<dyn SecretManager>,
        commit_coordinator: Arc<dyn CommitCoordinator>,
    ) -> Result<Self> {
        let handler = Arc::new(
            ServerHandlerInner::new(policy.clone(), store.clone(), secrets.clone())
                .with_commit_coordinator(commit_coordinator),
        );
        let session = Arc::new(KernelSession::new(handler.clone())?);
        Ok(Self {
            handler,
            session,
            table_source: None,
            volume_source: None,
        })
    }
}

impl<Cx: Send + Sync + 'static> ServerHandler<Cx> {
    /// Route shared-table resolution through `table_source` instead of the
    /// local store.
    ///
    /// Used by the hybrid topology so that Delta Sharing reads resolve their
    /// backing Table primitive from the same place every other table surface is
    /// served (e.g. an upstream Unity Catalog), rather than the local store.
    pub fn with_table_source(mut self, table_source: Arc<dyn TableHandler<Cx>>) -> Self {
        self.table_source = Some(table_source);
        self
    }

    /// The configured table source, if any.
    pub(crate) fn table_source(&self) -> Option<&Arc<dyn TableHandler<Cx>>> {
        self.table_source.as_ref()
    }

    /// Route shared-volume resolution through `volume_source` instead of the
    /// local store.
    ///
    /// Used by the hybrid topology so that Open Sharing volume and agent-skill
    /// reads resolve their backing Volume primitive from the same place every
    /// other volume surface is served (e.g. an upstream Unity Catalog), rather
    /// than the local store.
    pub fn with_volume_source(mut self, volume_source: Arc<dyn VolumeHandler<Cx>>) -> Self {
        self.volume_source = Some(volume_source);
        self
    }

    /// The configured volume source, if any.
    pub(crate) fn volume_source(&self) -> Option<&Arc<dyn VolumeHandler<Cx>>> {
        self.volume_source.as_ref()
    }
}

#[derive(Clone)]
pub struct ServerHandlerInner<Cx> {
    policy: Arc<dyn Policy<Cx>>,
    store: Arc<dyn ResourceStore>,
    object_store: Option<Arc<dyn olai_store::ObjectStore<ObjectLabel>>>,
    secrets: Arc<dyn SecretManager>,
    /// Delta catalog-managed commit coordinator (in-memory by default).
    commit_coordinator: Arc<dyn CommitCoordinator>,
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
            commit_coordinator: Arc::new(InMemoryCommitCoordinator::default()),
        }
    }

    /// Override the Delta commit coordinator (e.g. a Postgres-backed one, or a
    /// custom unbackfilled cap).
    pub fn with_commit_coordinator(mut self, coordinator: Arc<dyn CommitCoordinator>) -> Self {
        self.commit_coordinator = coordinator;
        self
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

impl<Cx: Send + Sync + 'static> ProvidesCommitCoordinator for ServerHandlerInner<Cx> {
    fn commit_coordinator(&self) -> &dyn CommitCoordinator {
        self.commit_coordinator.as_ref()
    }
}

impl<Cx: Send + Sync + 'static> ProvidesCommitCoordinator for ServerHandler<Cx> {
    fn commit_coordinator(&self) -> &dyn CommitCoordinator {
        self.handler.commit_coordinator.as_ref()
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
