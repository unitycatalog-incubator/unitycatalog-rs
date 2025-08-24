pub mod catalogs;
pub mod credentials;
pub mod external_locations;
pub mod recipients;
pub mod schemas;
pub mod shares;
pub mod tables;
pub mod temporary_credentials;
pub mod volumes;
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use futures::stream::TryStreamExt;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::models::catalogs::v1::*;
use unitycatalog_common::models::credentials::v1::*;
use unitycatalog_common::models::recipients::v1::*;
use unitycatalog_common::models::schemas::v1::*;
use unitycatalog_common::models::shares::v1::*;
use unitycatalog_common::models::tables::v1::*;
use unitycatalog_common::models::volumes::v1::*;
#[pyclass(name = "UnityCatalogClient")]
pub struct PyUnityCatalogClientABC {
    client: UnityCatalogClient,
}
#[pymethods]
impl PyUnityCatalogClientABC {
    #[new]
    #[pyo3(signature = (base_url, token = None))]
    pub fn new(base_url: String, token: Option<String>) -> PyResult<Self> {
        let client = if let Some(token) = token {
            cloud_client::CloudClient::new_with_token(token)
        } else {
            cloud_client::CloudClient::new_unauthenticated()
        };
        let base_url = base_url.parse().unwrap();
        Ok(Self {
            client: UnityCatalogClient::new(client, base_url),
        })
    }
    pub fn create_volume(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        name: String,
        volume_type: VolumeType,
        storage_location: Option<String>,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<VolumeInfo> {
        let mut request = self
            .client
            .create_volume(catalog_name, schema_name, name, volume_type);
        request = request.with_storage_location(storage_location);
        request = request.with_comment(comment);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    pub fn create_credential(
        &self,
        py: Python,
        name: String,
        purpose: Purpose,
        comment: Option<String>,
        read_only: Option<bool>,
        skip_validation: Option<bool>,
        azure_service_principal: Option<AzureServicePrincipal>,
        azure_managed_identity: Option<AzureManagedIdentity>,
        azure_storage_key: Option<AzureStorageKey>,
    ) -> PyUnityCatalogResult<CredentialInfo> {
        let mut request = self.client.create_credential(name, purpose);
        request = request.with_comment(comment);
        request = request.with_read_only(read_only);
        request = request.with_skip_validation(skip_validation);
        request = request.with_azure_service_principal(azure_service_principal);
        request = request.with_azure_managed_identity(azure_managed_identity);
        request = request.with_azure_storage_key(azure_storage_key);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    pub fn create_recipient(
        &self,
        py: Python,
        name: String,
        authentication_type: AuthenticationType,
        owner: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        expiration_time: Option<i64>,
    ) -> PyUnityCatalogResult<RecipientInfo> {
        let mut request = self
            .client
            .create_recipient(name, authentication_type, owner);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_expiration_time(expiration_time);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    pub fn create_table(
        &self,
        py: Python,
        name: String,
        schema_name: String,
        catalog_name: String,
        table_type: TableType,
        data_source_format: DataSourceFormat,
        columns: Option<ColumnInfo>,
        storage_location: Option<String>,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> PyUnityCatalogResult<TableInfo> {
        let mut request = self.client.create_table(
            name,
            schema_name,
            catalog_name,
            table_type,
            data_source_format,
        );
        request = request.with_columns(columns);
        request = request.with_storage_location(storage_location);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    pub fn create_schema(
        &self,
        py: Python,
        name: String,
        catalog_name: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> PyUnityCatalogResult<SchemaInfo> {
        let mut request = self.client.create_schema(name, catalog_name);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    pub fn create_share(
        &self,
        py: Python,
        name: String,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<ShareInfo> {
        let mut request = self.client.create_share(name);
        request = request.with_comment(comment);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    pub fn create_catalog(
        &self,
        py: Python,
        name: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        storage_root: Option<String>,
        provider_name: Option<String>,
        share_name: Option<String>,
    ) -> PyUnityCatalogResult<CatalogInfo> {
        let mut request = self.client.create_catalog(name);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_storage_root(storage_root);
        request = request.with_provider_name(provider_name);
        request = request.with_share_name(share_name);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
}
