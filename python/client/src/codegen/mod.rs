pub mod catalogs;
pub mod credentials;
pub mod external_locations;
pub mod recipients;
pub mod schemas;
pub mod shares;
pub mod tables;
pub mod temporary_credentials;
pub mod volumes;
use crate::codegen::catalogs::PyCatalogClient;
use crate::codegen::credentials::PyCredentialClient;
use crate::codegen::external_locations::PyExternalLocationClient;
use crate::codegen::recipients::PyRecipientClient;
use crate::codegen::schemas::PySchemaClient;
use crate::codegen::shares::PyShareClient;
use crate::codegen::tables::PyTableClient;
use crate::codegen::volumes::PyVolumeClient;
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use futures::stream::TryStreamExt;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::models::catalogs::v1::*;
use unitycatalog_common::models::credentials::v1::*;
use unitycatalog_common::models::external_locations::v1::*;
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
    #[pyo3(signature = (max_results = None))]
    pub fn list_catalogs(
        &self,
        py: Python,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<CatalogInfo>> {
        let mut request = self.client.list_catalogs();
        request = request.with_max_results(max_results);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(
        signature = (
            name,
            comment = None,
            properties = None,
            storage_root = None,
            provider_name = None,
            share_name = None
        )
    )]
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
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            max_results = None,
            include_browse = None
        )
    )]
    pub fn list_volumes(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<Vec<VolumeInfo>> {
        let mut request = self.client.list_volumes(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            name,
            volume_type,
            storage_location = None,
            comment = None
        )
    )]
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
    #[pyo3(signature = (catalog_name, max_results = None, include_browse = None))]
    pub fn list_schemas(
        &self,
        py: Python,
        catalog_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<Vec<SchemaInfo>> {
        let mut request = self.client.list_schemas(catalog_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(signature = (name, catalog_name, comment = None, properties = None))]
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
    #[pyo3(signature = (max_results = None, include_browse = None))]
    pub fn list_external_locations(
        &self,
        py: Python,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<Vec<ExternalLocationInfo>> {
        let mut request = self.client.list_external_locations();
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(
        signature = (
            name,
            url,
            credential_name,
            read_only = None,
            comment = None,
            skip_validation = None
        )
    )]
    pub fn create_external_location(
        &self,
        py: Python,
        name: String,
        url: String,
        credential_name: String,
        read_only: Option<bool>,
        comment: Option<String>,
        skip_validation: Option<bool>,
    ) -> PyUnityCatalogResult<ExternalLocationInfo> {
        let mut request = self
            .client
            .create_external_location(name, url, credential_name);
        request = request.with_read_only(read_only);
        request = request.with_comment(comment);
        request = request.with_skip_validation(skip_validation);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(signature = (max_results = None))]
    pub fn list_shares(
        &self,
        py: Python,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<ShareInfo>> {
        let mut request = self.client.list_shares();
        request = request.with_max_results(max_results);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(signature = (name, comment = None))]
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
    #[pyo3(signature = (purpose = None, max_results = None))]
    pub fn list_credentials(
        &self,
        py: Python,
        purpose: Option<Purpose>,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<CredentialInfo>> {
        let mut request = self.client.list_credentials();
        request = request.with_purpose(purpose);
        request = request.with_max_results(max_results);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(
        signature = (
            name,
            purpose,
            comment = None,
            read_only = None,
            skip_validation = None,
            azure_service_principal = None,
            azure_managed_identity = None,
            azure_storage_key = None
        )
    )]
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
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            max_results = None,
            include_delta_metadata = None,
            omit_columns = None,
            omit_properties = None,
            omit_username = None,
            include_browse = None,
            include_manifest_capabilities = None
        )
    )]
    pub fn list_tables(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_delta_metadata: Option<bool>,
        omit_columns: Option<bool>,
        omit_properties: Option<bool>,
        omit_username: Option<bool>,
        include_browse: Option<bool>,
        include_manifest_capabilities: Option<bool>,
    ) -> PyUnityCatalogResult<Vec<TableInfo>> {
        let mut request = self.client.list_tables(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_delta_metadata(include_delta_metadata);
        request = request.with_omit_columns(omit_columns);
        request = request.with_omit_properties(omit_properties);
        request = request.with_omit_username(omit_username);
        request = request.with_include_browse(include_browse);
        request = request.with_include_manifest_capabilities(include_manifest_capabilities);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(
        signature = (
            name,
            schema_name,
            catalog_name,
            table_type,
            data_source_format,
            columns = None,
            storage_location = None,
            comment = None,
            properties = None
        )
    )]
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
    #[pyo3(signature = (max_results = None))]
    pub fn list_recipients(
        &self,
        py: Python,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<RecipientInfo>> {
        let mut request = self.client.list_recipients();
        request = request.with_max_results(max_results);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(
        signature = (
            name,
            authentication_type,
            owner,
            comment = None,
            properties = None,
            expiration_time = None
        )
    )]
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
    pub fn catalog(&self, name: String) -> PyCatalogClient {
        PyCatalogClient {
            client: self.client.catalog(&name),
        }
    }
    pub fn volume(
        &self,
        catalog_name: String,
        schema_name: String,
        volume_name: String,
    ) -> PyVolumeClient {
        PyVolumeClient {
            client: self.client.volume(catalog_name, schema_name, volume_name),
        }
    }
    pub fn schema(&self, catalog_name: String, schema_name: String) -> PySchemaClient {
        PySchemaClient {
            client: self.client.schema(&catalog_name, &schema_name),
        }
    }
    pub fn external_location(&self, name: String) -> PyExternalLocationClient {
        PyExternalLocationClient {
            client: self.client.external_location(&name),
        }
    }
    pub fn share(&self, name: String) -> PyShareClient {
        PyShareClient {
            client: self.client.share(&name),
        }
    }
    pub fn credential(&self, name: String) -> PyCredentialClient {
        PyCredentialClient {
            client: self.client.credential(&name),
        }
    }
    pub fn table(&self, full_name: String) -> PyTableClient {
        PyTableClient {
            client: self.client.table(&full_name),
        }
    }
    pub fn recipient(&self, name: String) -> PyRecipientClient {
        PyRecipientClient {
            client: self.client.recipient(&name),
        }
    }
}
