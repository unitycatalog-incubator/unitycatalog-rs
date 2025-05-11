use std::sync::Arc;

use datafusion::prelude::SessionContext;
use delta_kernel::Version;
use delta_kernel_datafusion::{
    KernelContextExt as _, KernelExtensionConfig, ObjectStoreFactory, TableSnapshot,
};
use url::Url;

use super::kernel::TableManager;
use crate::tables::v1::DataSourceFormat;
use crate::{Error, Result};

pub struct KernelSession {
    ctx: SessionContext,
}

impl KernelSession {
    pub fn new(object_store_factory: Arc<dyn ObjectStoreFactory>) -> Self {
        let config =
            KernelExtensionConfig::default().with_object_store_factory(object_store_factory);
        let ctx = SessionContext::new().enable_delta_kernel(config);
        Self { ctx }
    }
}

#[async_trait::async_trait]
impl TableManager for KernelSession {
    async fn read_snapshot(
        &self,
        location: &Url,
        format: &DataSourceFormat,
        version: Option<Version>,
    ) -> Result<Arc<dyn TableSnapshot>> {
        match format {
            DataSourceFormat::Delta => Ok(self.ctx.read_delta_snapshot(location, version).await?),
            _ => Err(Error::InvalidArgument(format!(
                "unsupported data source format in kernel session: {:?}",
                format
            ))),
        }
    }
}
