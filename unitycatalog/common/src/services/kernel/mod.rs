use std::sync::Arc;

use delta_kernel::Version;
use delta_kernel_datafusion::TableSnapshot;

use crate::Result;
use crate::services::location::StorageLocationUrl;
use crate::tables::v1::DataSourceFormat;

pub use predicate::json_predicate_to_expression;

mod conversion;
mod predicate;

#[async_trait::async_trait]
pub trait TableManager: Send + Sync + 'static {
    async fn read_snapshot(
        &self,
        location: &StorageLocationUrl,
        format: &DataSourceFormat,
        version: Option<Version>,
    ) -> Result<Arc<dyn TableSnapshot>>;
}
