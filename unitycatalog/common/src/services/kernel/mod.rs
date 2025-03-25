use delta_kernel::snapshot::Snapshot;
use delta_kernel::{Engine, Table, Version};
use url::Url;

use crate::Result;
use crate::services::TableLocationResolver;
use crate::tables::v1::DataSourceFormat;

pub use predicate::json_predicate_to_expression;

mod conversion;
pub(crate) mod engine;
mod predicate;

pub trait TableManager: Send + Sync + 'static {
    fn read_snapshot(
        &self,
        location: &Url,
        format: &DataSourceFormat,
        version: Option<Version>,
    ) -> Result<Snapshot>;
}

pub trait ProvidesEngine: Send + Sync + 'static {
    fn engine(&self) -> &dyn Engine;
}

impl<T: ProvidesEngine + TableLocationResolver> TableManager for T {
    fn read_snapshot(
        &self,
        location: &Url,
        format: &DataSourceFormat,
        version: Option<Version>,
    ) -> Result<Snapshot> {
        if format != &DataSourceFormat::Delta {
            return Err(crate::Error::Generic(format!(
                "Unsupported format: {:?}",
                format
            )));
        }
        let table = Table::new(location.clone());
        let snapshot = table.snapshot(self.engine(), version)?;
        Ok(snapshot)
    }
}
