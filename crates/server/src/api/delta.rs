//! Handler trait for the UC Delta REST API (`/delta/v1/...`).
//!
//! This is the hand-written counterpart to the generated resource handlers: the
//! Delta API is a standalone REST protocol (mirrored from the Unity Catalog Java
//! reference `DeltaApiService` / `openapi/delta.yaml`), so the trait, router, and
//! models are all maintained by hand. See the Delta Sharing handler for the
//! precedent.
//!
//! Phase 1 ships the trait + a stubbed default implementation surface; the router
//! wires every operation. Behavior (contract validation, the `updateTable` action
//! dispatcher, commit coordination) lands in Phase 2.

use async_trait::async_trait;

use crate::api::RequestContext;
use crate::policy::Policy;
use crate::rest::routers::delta::models::*;
use crate::store::ResourceStore;
use crate::{Error, Result};

/// A fully-qualified table coordinate parsed from the request path.
#[derive(Debug, Clone)]
pub struct TablePath {
    pub catalog: String,
    pub schema: String,
    pub table: String,
}

/// A schema coordinate (parent of staging-tables / tables creation).
#[derive(Debug, Clone)]
pub struct SchemaPath {
    pub catalog: String,
    pub schema: String,
}

/// Query parameters for `getConfig`.
#[derive(Debug, Clone)]
pub struct GetConfigQuery {
    pub catalog: String,
    /// Comma-separated list of highest protocol versions the client supports.
    pub protocol_versions: String,
}

/// Handler for the Delta REST API. One method per `delta.yaml` operation.
///
/// Method names match the spec `operationId`s. Path/query parameters are passed
/// as typed structs; request bodies use the hand-written model types.
#[async_trait]
pub trait DeltaApiHandler<Cx = RequestContext>: Send + Sync + 'static {
    /// `GET /delta/v1/config`
    async fn get_config(&self, query: GetConfigQuery, context: Cx) -> Result<DeltaCatalogConfig>;

    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/staging-tables`
    async fn create_staging_table(
        &self,
        path: SchemaPath,
        request: DeltaCreateStagingTableRequest,
        context: Cx,
    ) -> Result<DeltaStagingTableResponse>;

    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/tables`
    async fn create_table(
        &self,
        path: SchemaPath,
        request: DeltaCreateTableRequest,
        context: Cx,
    ) -> Result<DeltaLoadTableResponse>;

    /// `GET /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}`
    async fn load_table(&self, path: TablePath, context: Cx) -> Result<DeltaLoadTableResponse>;

    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}`
    async fn update_table(
        &self,
        path: TablePath,
        request: DeltaUpdateTableRequest,
        context: Cx,
    ) -> Result<DeltaLoadTableResponse>;

    /// `DELETE /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}`
    async fn delete_table(&self, path: TablePath, context: Cx) -> Result<()>;

    /// `HEAD /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}`
    ///
    /// Returns `Ok(())` if the table exists; the router maps a not-found error to 404.
    async fn table_exists(&self, path: TablePath, context: Cx) -> Result<()>;

    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/rename`
    async fn rename_table(
        &self,
        path: TablePath,
        request: DeltaRenameTableRequest,
        context: Cx,
    ) -> Result<()>;

    /// `GET /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/credentials`
    async fn get_table_credentials(
        &self,
        path: TablePath,
        operation: DeltaCredentialOperation,
        context: Cx,
    ) -> Result<DeltaCredentialsResponse>;

    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/metrics`
    async fn report_metrics(
        &self,
        path: TablePath,
        request: DeltaReportMetricsRequest,
        context: Cx,
    ) -> Result<()>;

    /// `GET /delta/v1/staging-tables/{table_id}/credentials`
    async fn get_staging_table_credentials(
        &self,
        table_id: String,
        context: Cx,
    ) -> Result<DeltaCredentialsResponse>;

    /// `GET /delta/v1/temporary-path-credentials`
    async fn get_temporary_path_credentials(
        &self,
        location: String,
        operation: DeltaCredentialOperation,
        context: Cx,
    ) -> Result<DeltaCredentialsResponse>;
}

/// Phase 1 stub implementation.
///
/// Every backend that is a [`ResourceStore`] + [`Policy`] gets a `DeltaApiHandler`
/// that returns [`Error::NotImplemented`] for each operation. This lets the router
/// mount and serve the Delta API surface (and its OpenAPI spec) while the actual
/// behavior is ported in Phase 2. The real implementation will replace this blanket
/// impl with one that carries the additional bounds it needs (commit coordinator,
/// credential vending, `TableManager`).
#[async_trait]
impl<T> DeltaApiHandler<RequestContext> for T
where
    T: ResourceStore + Policy<RequestContext>,
{
    async fn get_config(
        &self,
        _query: GetConfigQuery,
        _context: RequestContext,
    ) -> Result<DeltaCatalogConfig> {
        Err(Error::NotImplemented("Delta API: getConfig"))
    }

    async fn create_staging_table(
        &self,
        _path: SchemaPath,
        _request: DeltaCreateStagingTableRequest,
        _context: RequestContext,
    ) -> Result<DeltaStagingTableResponse> {
        Err(Error::NotImplemented("Delta API: createStagingTable"))
    }

    async fn create_table(
        &self,
        _path: SchemaPath,
        _request: DeltaCreateTableRequest,
        _context: RequestContext,
    ) -> Result<DeltaLoadTableResponse> {
        Err(Error::NotImplemented("Delta API: createTable"))
    }

    async fn load_table(
        &self,
        _path: TablePath,
        _context: RequestContext,
    ) -> Result<DeltaLoadTableResponse> {
        Err(Error::NotImplemented("Delta API: loadTable"))
    }

    async fn update_table(
        &self,
        _path: TablePath,
        _request: DeltaUpdateTableRequest,
        _context: RequestContext,
    ) -> Result<DeltaLoadTableResponse> {
        Err(Error::NotImplemented("Delta API: updateTable"))
    }

    async fn delete_table(&self, _path: TablePath, _context: RequestContext) -> Result<()> {
        Err(Error::NotImplemented("Delta API: deleteTable"))
    }

    async fn table_exists(&self, _path: TablePath, _context: RequestContext) -> Result<()> {
        Err(Error::NotImplemented("Delta API: tableExists"))
    }

    async fn rename_table(
        &self,
        _path: TablePath,
        _request: DeltaRenameTableRequest,
        _context: RequestContext,
    ) -> Result<()> {
        Err(Error::NotImplemented("Delta API: renameTable"))
    }

    async fn get_table_credentials(
        &self,
        _path: TablePath,
        _operation: DeltaCredentialOperation,
        _context: RequestContext,
    ) -> Result<DeltaCredentialsResponse> {
        Err(Error::NotImplemented("Delta API: getTableCredentials"))
    }

    async fn report_metrics(
        &self,
        _path: TablePath,
        _request: DeltaReportMetricsRequest,
        _context: RequestContext,
    ) -> Result<()> {
        Err(Error::NotImplemented("Delta API: reportMetrics"))
    }

    async fn get_staging_table_credentials(
        &self,
        _table_id: String,
        _context: RequestContext,
    ) -> Result<DeltaCredentialsResponse> {
        Err(Error::NotImplemented(
            "Delta API: getStagingTableCredentials",
        ))
    }

    async fn get_temporary_path_credentials(
        &self,
        _location: String,
        _operation: DeltaCredentialOperation,
        _context: RequestContext,
    ) -> Result<DeltaCredentialsResponse> {
        Err(Error::NotImplemented(
            "Delta API: getTemporaryPathCredentials",
        ))
    }
}
