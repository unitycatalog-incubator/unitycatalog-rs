use delta_kernel::Version;
use delta_kernel::snapshot::Snapshot;
use url::Url;

use crate::Result;
use crate::tables::v1::DataSourceFormat;

pub use predicate::json_predicate_to_expression;

mod conversion;
pub(crate) mod engine;
mod predicate;

#[async_trait::async_trait]
pub trait TableManager: Send + Sync + 'static {
    async fn read_snapshot(
        &self,
        location: &Url,
        format: &DataSourceFormat,
        version: Option<Version>,
    ) -> Result<Snapshot>;
}
