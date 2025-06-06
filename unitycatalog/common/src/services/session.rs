use std::sync::{Arc, LazyLock};

use bytes::Bytes;
use datafusion::arrow::array::{AsArray, RecordBatch};
use datafusion::arrow::json::LineDelimitedWriter;
use datafusion::logical_expr::ColumnarValue;
use datafusion::physical_plan::PhysicalExpr;
use datafusion::prelude::SessionContext;
use datafusion::prelude::{Expr, col, lit, named_struct};
use datafusion_catalog::{CatalogProvider, MemoryCatalogProvider, MemorySchemaProvider};
use datafusion_common::TableReference as DfTableReference;
use datafusion_functions::core::expr_ext::FieldAccessor;
use delta_kernel::{Table, Version};
use deltalake_datafusion::{
    DeltaLogReplayProvider, KernelContextExt as _, KernelExtensionConfig, ObjectStoreFactory,
    TableSnapshot,
};
use itertools::Itertools;

use super::kernel::TableManager;
use super::sharing::{SharingExt, SharingTableReference};
use crate::services::location::StorageLocationUrl;
use crate::tables::v1::DataSourceFormat;
use crate::{Error, Result};

const UC_RS_SYSTEM_CATALOG_NAME: &str = "uc_rs_system";
const UC_RS_LOG_REPLAY_SCHEMA_NAME: &str = "uc_rs_log_replay";

static PQ_FILE_EXTRACT: LazyLock<Expr> = LazyLock::new(|| {
    named_struct(vec![
        lit("file"),
        named_struct(vec![
            lit("path"),
            col("path"),
            lit("partitionValues"),
            col("\"fileConstantValues\"").field("partitionValues"),
            lit("size"),
            col("size"),
        ]),
    ])
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableReference {
    Sharing(SharingTableReference),
    Datafusion(DfTableReference),
}

struct Extractors {
    sharing_pq_files: Arc<dyn PhysicalExpr>,
}

impl Extractors {
    pub fn new(ctx: &SessionContext) -> Result<Self> {
        let df_schema = DeltaLogReplayProvider::scan_row_schema().try_into()?;
        let sharing_pq_files = ctx.create_physical_expr(PQ_FILE_EXTRACT.clone(), &df_schema)?;
        Ok(Self { sharing_pq_files })
    }
}

pub struct KernelSession {
    ctx: SessionContext,
    extractors: Extractors,
}

impl KernelSession {
    pub fn new(object_store_factory: Arc<dyn ObjectStoreFactory>) -> Result<Self> {
        let config =
            KernelExtensionConfig::default().with_object_store_factory(object_store_factory);
        let ctx = SessionContext::new().enable_delta_kernel(config);
        let catalog = Arc::new(MemoryCatalogProvider::new());
        catalog.register_schema(
            UC_RS_LOG_REPLAY_SCHEMA_NAME,
            Arc::new(MemorySchemaProvider::new()),
        )?;
        ctx.register_catalog(UC_RS_SYSTEM_CATALOG_NAME, catalog);

        Ok(Self {
            extractors: Extractors::new(&ctx)?,
            ctx,
        })
    }

    pub fn ctx(&self) -> &SessionContext {
        &self.ctx
    }

    pub fn system_catalog(&self) -> Arc<dyn CatalogProvider> {
        self.ctx
            .catalog(UC_RS_SYSTEM_CATALOG_NAME)
            .expect("system catalog should be registered in kernel session")
    }

    pub(super) async fn extract_sharing_query_response(
        &self,
        table_ref: &SharingTableReference,
        sharing_ext: &dyn SharingExt,
    ) -> Result<Bytes> {
        let log_replay_table_name = table_ref.system_table_name();
        let inner_ref = DfTableReference::full(
            UC_RS_SYSTEM_CATALOG_NAME,
            UC_RS_LOG_REPLAY_SCHEMA_NAME,
            log_replay_table_name,
        );
        if !self.ctx.table_exist(inner_ref.clone())? {
            let location = sharing_ext.table_location(table_ref).await?;
            let table = Table::try_from_uri(location.location())?;
            self.ctx.register_table(
                inner_ref.clone(),
                Arc::new(DeltaLogReplayProvider::new(table.into())?),
            )?;
        }
        let table = self.ctx.table(inner_ref).await?.collect().await?;
        let results: Vec<_> = table
            .iter()
            .map(|batch| {
                let res = match self.extractors.sharing_pq_files.evaluate(batch)? {
                    ColumnarValue::Array(arr) => arr,
                    ColumnarValue::Scalar(scalar) => scalar.to_array_of_size(batch.num_rows())?,
                };
                Ok::<_, Error>(RecordBatch::from(res.as_struct()))
            })
            .try_collect()?;
        encode_nd_json(&results) // spellchecker:disable-line
    }
}

#[async_trait::async_trait]
impl TableManager for KernelSession {
    async fn read_snapshot(
        &self,
        location: &StorageLocationUrl,
        format: &DataSourceFormat,
        version: Option<Version>,
    ) -> Result<Arc<dyn TableSnapshot>> {
        match format {
            DataSourceFormat::Delta => Ok(self
                .ctx
                .read_delta_snapshot(location.location(), version)
                .await?),
            _ => Err(Error::InvalidArgument(format!(
                "unsupported data source format in kernel session: {:?}",
                format
            ))),
        }
    }
}

// spellchecker:ignore-next-line
pub fn encode_nd_json(data: &[RecordBatch]) -> Result<Bytes> {
    let mut writer = LineDelimitedWriter::new(Vec::new());
    for batch in data {
        writer.write(batch)?;
    }
    Ok(Bytes::from(writer.into_inner()))
}
