use delta_kernel::schema::{DataType, PrimitiveType, Schema, StructField};
use itertools::Itertools;
use unitycatalog_derive::rest_handlers;

use super::{RequestContext, SecuredAction};
use crate::models::ObjectLabel;
use crate::models::tables::v1::*;
use crate::resources::{ResourceIdent, ResourceName, ResourceRef, ResourceStore};
use crate::services::StorageLocationUrl;
use crate::services::kernel::TableManager;
use crate::services::policy::{Permission, Policy, Recipient, process_resources};
use crate::{Error, Result};

const MAX_RESULTS_TABLES: usize = 50;

rest_handlers!(
    TablesHandler, "tables", [
        ListTableSummariesRequest, Table, Read, ListTableSummariesResponse with [
            catalog_name: query as String,
            schema_name_pattern: query as Option<String>,
            table_name_pattern: query as Option<String>,
            include_manifest_capabilities: query as Option<bool>,
        ];
        ListTablesRequest, Table, Read, ListTablesResponse with [
            catalog_name: query as String,
            schema_name: query as String,
            include_delta_metadata: query as Option<bool>,
            omit_columns: query as Option<bool>,
            omit_properties: query as Option<bool>,
            omit_username: query as Option<bool>,
            include_browse: query as Option<bool>,
            include_manifest_capabilities: query as Option<bool>,
        ];
        CreateTableRequest, Table, Create, TableInfo;
        GetTableRequest, Table, Read, TableInfo with [
            full_name: path as String,
            include_delta_metadata: query as Option<bool>,
            include_browse: query as Option<bool>,
            include_manifest_capabilities: query as Option<bool>,
        ];
        GetTableExistsRequest, Table, Read, GetTableExistsResponse with [
            full_name: path as String,
        ];
        DeleteTableRequest, Table, Write with [
            full_name: path as String,
        ];
    ]
);

#[async_trait::async_trait]
pub trait TablesHandler: Send + Sync + 'static {
    /// List table summaries.
    async fn list_table_summaries(
        &self,
        request: ListTableSummariesRequest,
        context: RequestContext,
    ) -> Result<ListTableSummariesResponse>;

    /// List tables.
    async fn list_tables(
        &self,
        request: ListTablesRequest,
        context: RequestContext,
    ) -> Result<ListTablesResponse>;

    /// Create a table.
    async fn create_table(
        &self,
        request: CreateTableRequest,
        context: RequestContext,
    ) -> Result<TableInfo>;

    /// Get a table.
    async fn get_table(
        &self,
        request: GetTableRequest,
        context: RequestContext,
    ) -> Result<TableInfo>;

    /// Check if a table exists.
    async fn get_table_exists(
        &self,
        request: GetTableExistsRequest,
        context: RequestContext,
    ) -> Result<GetTableExistsResponse>;

    /// Delete a table.
    async fn delete_table(
        &self,
        request: DeleteTableRequest,
        context: RequestContext,
    ) -> Result<()>;
}

#[async_trait::async_trait]
impl<T: ResourceStore + Policy + TableManager> TablesHandler for T {
    async fn list_table_summaries(
        &self,
        request: ListTableSummariesRequest,
        context: RequestContext,
    ) -> Result<ListTableSummariesResponse> {
        self.check_required(&request, context.as_ref()).await?;
        // TODO: handle like operators for schema and table name
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::TableInfo,
                Some(&ResourceName::new([&request.catalog_name])),
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, context.as_ref(), &Permission::Read, &mut resources).await?;
        let infos: Vec<TableInfo> = resources.into_iter().map(|r| r.try_into()).try_collect()?;
        Ok(ListTableSummariesResponse {
            tables: infos.into_iter().map(|r| r.into()).collect(),
            next_page_token,
        })
    }

    async fn list_tables(
        &self,
        request: ListTablesRequest,
        context: RequestContext,
    ) -> Result<ListTablesResponse> {
        // TODO: assert max_results is within bounds <= 50
        self.check_required(&request, context.as_ref()).await?;
        // TODO: handle like operators for schema and table name
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::TableInfo,
                Some(&ResourceName::new([
                    &request.catalog_name,
                    &request.schema_name,
                ])),
                request
                    .max_results
                    .map(|v| usize::min(v as usize, MAX_RESULTS_TABLES)),
                request.page_token,
            )
            .await?;
        process_resources(self, context.as_ref(), &Permission::Read, &mut resources).await?;
        Ok(ListTablesResponse {
            tables: resources.into_iter().map(|r| r.try_into()).try_collect()?,
            next_page_token,
        })
    }

    async fn create_table(
        &self,
        request: CreateTableRequest,
        context: RequestContext,
    ) -> Result<TableInfo> {
        self.check_required(&request, context.as_ref()).await?;
        let info = if request.table_type == TableType::External as i32 {
            let Some(location) = request.storage_location.as_ref() else {
                return Err(Error::invalid_argument("missing storage location"));
            };
            let location = StorageLocationUrl::parse(location)?;
            let snapshot = self
                .read_snapshot(&location, &request.data_source_format(), None)
                .await?;
            TableInfo {
                name: request.name,
                catalog_name: request.catalog_name,
                schema_name: request.schema_name,
                table_type: request.table_type,
                data_source_format: request.data_source_format,
                properties: request.properties,
                storage_location: request.storage_location,
                comment: request.comment,
                columns: schema_to_columns(
                    snapshot.schema().as_ref(),
                    snapshot.metadata().partition_columns(),
                )?,
                ..Default::default()
            }
        } else {
            TableInfo {
                name: request.name,
                catalog_name: request.catalog_name,
                schema_name: request.schema_name,
                table_type: request.table_type,
                data_source_format: request.data_source_format,
                properties: request.properties,
                storage_location: request.storage_location,
                comment: request.comment,
                columns: request.columns,
                ..Default::default()
            }
        };
        // TODO: update the table with the current actor as owner
        // TODO: create updated_* relations
        self.create(info.into()).await?.0.try_into()
    }

    async fn get_table(
        &self,
        request: GetTableRequest,
        context: RequestContext,
    ) -> Result<TableInfo> {
        self.check_required(&request, context.as_ref()).await?;
        // TODO: get columns etc ...
        self.get(&request.resource()).await?.0.try_into()
    }

    async fn get_table_exists(
        &self,
        request: GetTableExistsRequest,
        context: RequestContext,
    ) -> Result<GetTableExistsResponse> {
        self.check_required(&request, context.as_ref()).await?;
        match self.get(&request.resource()).await {
            Ok(_) => Ok(GetTableExistsResponse { table_exists: true }),
            Err(Error::NotFound) => Ok(GetTableExistsResponse {
                table_exists: false,
            }),
            Err(e) => Err(e),
        }
    }

    async fn delete_table(
        &self,
        request: DeleteTableRequest,
        context: RequestContext,
    ) -> Result<()> {
        self.check_required(&request, context.as_ref()).await?;
        self.delete(&request.resource()).await
    }
}

trait FieldExt {
    fn type_text(&self) -> String;
    fn type_json(&self) -> Result<String>;
    fn type_name(&self) -> ColumnTypeName;
    fn type_precision(&self) -> Option<i32>;
    fn type_scale(&self) -> Option<i32>;
}

impl FieldExt for StructField {
    fn type_text(&self) -> String {
        self.data_type().to_string()
    }

    fn type_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self.data_type())?)
    }

    fn type_name(&self) -> ColumnTypeName {
        match self.data_type() {
            DataType::Primitive(p) => match p {
                PrimitiveType::String => ColumnTypeName::String,
                PrimitiveType::Long => ColumnTypeName::Long,
                PrimitiveType::Integer => ColumnTypeName::Int,
                PrimitiveType::Short => ColumnTypeName::Short,
                PrimitiveType::Byte => ColumnTypeName::Byte,
                PrimitiveType::Double => ColumnTypeName::Double,
                PrimitiveType::Float => ColumnTypeName::Float,
                PrimitiveType::Boolean => ColumnTypeName::Boolean,
                PrimitiveType::Binary => ColumnTypeName::Binary,
                PrimitiveType::Date => ColumnTypeName::Date,
                PrimitiveType::Timestamp => ColumnTypeName::Timestamp,
                PrimitiveType::TimestampNtz => ColumnTypeName::TimestampNtz,
                PrimitiveType::Decimal(_) => ColumnTypeName::Decimal,
            },
            DataType::Struct(_) => ColumnTypeName::Struct,
            DataType::Array(_) => ColumnTypeName::Array,
            DataType::Map(_) => ColumnTypeName::Map,
        }
    }

    fn type_precision(&self) -> Option<i32> {
        match self.data_type() {
            DataType::Primitive(PrimitiveType::Decimal(dec)) => Some(dec.precision() as i32),
            _ => None,
        }
    }

    fn type_scale(&self) -> Option<i32> {
        match self.data_type() {
            DataType::Primitive(PrimitiveType::Decimal(dec)) => Some(dec.scale() as i32),
            _ => None,
        }
    }
}

fn schema_to_columns(schema: &Schema, partition_columns: &[String]) -> Result<Vec<ColumnInfo>> {
    let partition_index = |name: &str| partition_columns.iter().position(|n| n == name);
    schema
        .fields()
        .enumerate()
        .map(|(idx, f)| {
            Ok(ColumnInfo {
                name: f.name.clone(),
                nullable: Some(f.nullable),
                type_text: f.type_text(),
                type_json: f.type_json()?,
                type_name: f.type_name() as i32,
                type_precision: f.type_precision(),
                type_scale: f.type_scale(),
                type_interval_type: None,
                position: Some(idx as i32),
                partition_index: partition_index(&f.name).map(|v| v as i32),
                ..Default::default()
            })
        })
        .try_collect()
}
