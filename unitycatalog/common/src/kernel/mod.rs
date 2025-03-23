use std::sync::Arc;

use delta_kernel::snapshot::Snapshot;
use delta_kernel::{Engine, Table};

use crate::api::{RequestContext, SecuredAction, SharingQueryHandler};
use crate::models::sharing::v1::{
    GetTableMetadataRequest, GetTableVersionRequest, GetTableVersionResponse, QueryResponse,
};
use crate::services::{Policy, TableLocationResolver};
use crate::{ResourceRef, Result};

pub use predicate::json_predicate_to_expression;

mod conversion;
mod engine;
mod predicate;

pub struct KernelQueryHandler {
    engine: Arc<dyn Engine>,
    location_resolver: Arc<dyn TableLocationResolver>,
    policy: Arc<dyn Policy>,
}

impl KernelQueryHandler {
    /// Create a new instance of [`KernelQueryHandler`].
    pub fn new(
        engine: Arc<dyn Engine>,
        location_resolver: Arc<dyn TableLocationResolver>,
        policy: Arc<dyn Policy>,
    ) -> Self {
        Self {
            engine,
            location_resolver,
            policy,
        }
    }

    /// Create a new instance of [`KernelQueryHandler`] with a background executor.
    #[cfg(feature = "tokio")]
    pub fn try_new_tokio(
        handler: Arc<dyn engine::RegistryHandler>,
        location_resolver: Arc<dyn TableLocationResolver>,
        policy: Arc<dyn Policy>,
    ) -> Result<Arc<Self>> {
        use delta_kernel::engine::default::executor::tokio::TokioBackgroundExecutor;
        use delta_kernel::engine::default::executor::tokio::TokioMultiThreadExecutor;

        let handle = tokio::runtime::Handle::try_current()
            .map_err(|e| crate::Error::generic(e.to_string()))?;
        let engine: Arc<dyn Engine> = match handle.runtime_flavor() {
            tokio::runtime::RuntimeFlavor::MultiThread => {
                engine::get_engine(handler, Arc::new(TokioMultiThreadExecutor::new(handle)))?
            }
            tokio::runtime::RuntimeFlavor::CurrentThread => {
                engine::get_engine(handler, Arc::new(TokioBackgroundExecutor::new()))?
            }
            _ => {
                return Err(crate::Error::generic("Unsupported runtime flavor"));
            }
        };

        Ok(Arc::new(Self::new(engine, location_resolver, policy)))
    }

    async fn get_snapshot(&self, table_ref: &ResourceRef) -> Result<Snapshot> {
        let location = self.location_resolver.resolve(table_ref).await?;
        let table = Table::new(location);
        let snapshot = table.snapshot(self.engine.as_ref(), None)?;
        Ok(snapshot)
    }
}

#[async_trait::async_trait]
impl SharingQueryHandler for KernelQueryHandler {
    async fn get_table_version(
        &self,
        request: GetTableVersionRequest,
        context: RequestContext,
    ) -> Result<GetTableVersionResponse> {
        self.policy
            .check_required(&request, context.as_ref())
            .await?;
        let res = request.resource();
        // TODO handle optional timestamp
        let snapshot = self.get_snapshot(res.as_ref()).await?;
        let version = snapshot.version();
        Ok(GetTableVersionResponse {
            version: version as i64,
        })
    }

    async fn get_table_metadata(
        &self,
        request: GetTableMetadataRequest,
        context: RequestContext,
    ) -> Result<QueryResponse> {
        self.policy
            .check_required(&request, context.as_ref())
            .await?;
        let res = request.resource();
        let snapshot = self.get_snapshot(res.as_ref()).await?;
        Ok([snapshot.metadata().into(), snapshot.protocol().into()].into())
    }
}
