use std::collections::HashMap;

use futures::stream::TryStreamExt;
use pyo3::prelude::*;
use unitycatalog_client::{
    PathOperation, TableOperation, TableReference, TemporaryCredentialClient, UnityCatalogClient,
};
use unitycatalog_common::models::catalogs::v1::CatalogInfo;
use unitycatalog_common::models::credentials::v1::{CredentialInfo, Purpose as CredentialPurpose};
use unitycatalog_common::models::external_locations::v1::ExternalLocationInfo;
use unitycatalog_common::models::recipients::v1::{AuthenticationType, RecipientInfo};
use unitycatalog_common::models::schemas::v1::SchemaInfo;
use unitycatalog_common::models::shares::v1::ShareInfo;
use unitycatalog_common::models::tables::v1::TableInfo;
use unitycatalog_common::models::temporary_credentials::v1::TemporaryCredential;
use unitycatalog_common::models::volumes::v1::{VolumeInfo, VolumeType};

pub use crate::codegen::catalogs::PyCatalogClient;
pub use crate::codegen::credentials::PyCredentialClient;
pub use crate::codegen::external_locations::PyExternalLocationClient;
pub use crate::codegen::recipients::PyRecipientClient;
pub use crate::codegen::schemas::PySchemaClient;
pub use crate::codegen::shares::PyShareClient;
pub use crate::codegen::tables::PyTableClient;
pub use crate::codegen::volumes::PyVolumeClient;
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;

#[pyclass(name = "UnityCatalogClient")]
pub struct PyUnityCatalogClient(UnityCatalogClient);

#[pymethods]
impl PyUnityCatalogClient {
    #[new]
    #[pyo3(signature = (base_url, token = None))]
    pub fn new(base_url: String, token: Option<String>) -> PyResult<Self> {
        let client = if let Some(token) = token {
            cloud_client::CloudClient::new_with_token(token)
        } else {
            cloud_client::CloudClient::new_unauthenticated()
        };
        let base_url = base_url.parse().unwrap();
        Ok(Self(UnityCatalogClient::new(client, base_url)))
    }

    #[pyo3(signature = (max_results = None))]
    pub fn list_catalogs(
        &self,
        py: Python,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<CatalogInfo>> {
        let stream = self.0.list_catalogs().with_max_results(max_results);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let catalogs =
                runtime.block_on(async move { stream.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(catalogs)
        })
    }

    pub fn catalog(&self, name: String) -> PyCatalogClient {
        PyCatalogClient {
            client: self.0.catalog(&name),
        }
    }

    #[pyo3(signature = (catalog_name, max_results = None, include_browse = None))]
    pub fn list_schemas(
        &self,
        py: Python,
        catalog_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<Vec<SchemaInfo>> {
        let stream = self
            .0
            .list_schemas(catalog_name)
            .with_max_results(max_results)
            .with_include_browse(include_browse);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let schemas = runtime
                .block_on(async move { stream.into_stream().try_collect::<Vec<_>>().await })?;
            Ok::<_, PyUnityCatalogError>(schemas)
        })
    }

    pub fn schema(&self, catalog_name: String, schema_name: String) -> PySchemaClient {
        PySchemaClient {
            client: self.0.schema(&catalog_name, &schema_name),
        }
    }

    #[pyo3(signature = (
        catalog_name,
        schema_name,
        max_results = None,
        include_delta_metadata = None,
        omit_columns = None,
        omit_properties = None,
        omit_username = None,
        include_browse = None,
        include_manifest_capabilities = None)
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
        let stream = self
            .0
            .list_tables(catalog_name, schema_name)
            .with_max_results(max_results)
            .with_include_browse(include_browse)
            .with_include_delta_metadata(include_delta_metadata)
            .with_omit_columns(omit_columns)
            .with_omit_properties(omit_properties)
            .with_omit_username(omit_username)
            .with_include_manifest_capabilities(include_manifest_capabilities);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let tables = runtime
                .block_on(async move { stream.into_stream().try_collect::<Vec<_>>().await })?;
            Ok::<_, PyUnityCatalogError>(tables)
        })
    }

    pub fn table(&self, full_name: String) -> PyTableClient {
        PyTableClient {
            client: self.0.table(&full_name),
        }
    }

    #[pyo3(signature = (max_results = None))]
    pub fn list_shares(
        &self,
        py: Python,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<ShareInfo>> {
        let stream = self.0.list_shares().with_max_results(max_results);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let shares = runtime
                .block_on(async move { stream.into_stream().try_collect::<Vec<_>>().await })?;
            Ok::<_, PyUnityCatalogError>(shares)
        })
    }

    pub fn share(&self, name: String) -> PyShareClient {
        PyShareClient {
            client: self.0.share(&name),
        }
    }

    #[pyo3(signature = (max_results = None))]
    pub fn list_recipients(
        &self,
        py: Python,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<RecipientInfo>> {
        let stream = self.0.list_recipients().with_max_results(max_results);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let recipients = runtime
                .block_on(async move { stream.into_stream().try_collect::<Vec<_>>().await })?;
            Ok::<_, PyUnityCatalogError>(recipients)
        })
    }

    pub fn recipient(&self, name: String) -> PyRecipientClient {
        PyRecipientClient {
            client: self.0.recipient(&name),
        }
    }

    #[pyo3(signature = (purpose = None, max_results = None))]
    pub fn list_credentials(
        &self,
        py: Python,
        purpose: Option<CredentialPurpose>,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<CredentialInfo>> {
        let stream = self
            .0
            .list_credentials()
            .with_max_results(max_results)
            .with_purpose(purpose);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let credentials = runtime
                .block_on(async move { stream.into_stream().try_collect::<Vec<_>>().await })?;
            Ok::<_, PyUnityCatalogError>(credentials)
        })
    }

    pub fn credential(&self, name: String) -> PyCredentialClient {
        PyCredentialClient {
            client: self.0.credential(&name),
        }
    }

    #[pyo3(signature = (max_results = None, include_browse = None))]
    pub fn list_external_locations(
        &self,
        py: Python,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<Vec<ExternalLocationInfo>> {
        let stream = self
            .0
            .list_external_locations()
            .with_max_results(max_results)
            .with_include_browse(include_browse);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let locations = runtime
                .block_on(async move { stream.into_stream().try_collect::<Vec<_>>().await })?;
            Ok::<_, PyUnityCatalogError>(locations)
        })
    }

    pub fn external_location(&self, name: String) -> PyExternalLocationClient {
        PyExternalLocationClient {
            client: self.0.external_location(&name),
        }
    }

    // Create methods
    #[pyo3(signature = (name, storage_root = None, comment = None, properties = None))]
    pub fn create_catalog(
        &self,
        py: Python,
        name: String,
        storage_root: Option<String>,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> PyUnityCatalogResult<CatalogInfo> {
        let mut request = self
            .0
            .create_catalog(name)
            .with_storage_root(storage_root)
            .with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (name, provider_name, share_name, comment = None, properties = None))]
    pub fn create_sharing_catalog(
        &self,
        py: Python,
        name: String,
        provider_name: String,
        share_name: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> PyUnityCatalogResult<CatalogInfo> {
        let mut request = self
            .0
            .create_catalog(name)
            .with_provider_name(provider_name)
            .with_share_name(share_name)
            .with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (catalog_name, schema_name, comment = None))]
    pub fn create_schema(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<SchemaInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let mut request = self.0.create_schema(catalog_name, schema_name);
            if let Some(comment) = comment {
                request = request.with_comment(comment);
            }
            let info = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (name, comment = None))]
    pub fn create_share(
        &self,
        py: Python,
        name: String,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<ShareInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let mut request = self.0.create_share(name);
            if let Some(comment) = comment {
                request = request.with_comment(comment);
            }
            let info = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (name, authentication_type, comment = None))]
    pub fn create_recipient(
        &self,
        py: Python,
        name: String,
        authentication_type: AuthenticationType,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<RecipientInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let mut request = self.0.create_recipient(name, authentication_type, "");
            if let Some(comment) = comment {
                request = request.with_comment(comment);
            }
            let info = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (name, purpose, comment = None))]
    pub fn create_credential(
        &self,
        py: Python,
        name: String,
        purpose: CredentialPurpose,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<CredentialInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let mut request = self.0.create_credential(name, purpose);
            if let Some(comment) = comment {
                request = request.with_comment(comment);
            }
            let info = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (name, url, credential_name, comment = None))]
    pub fn create_external_location(
        &self,
        py: Python,
        name: String,
        url: String,
        credential_name: String,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<ExternalLocationInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let mut request = self.0.create_external_location(name, url, credential_name);
            if let Some(comment) = comment {
                request = request.with_comment(comment);
            }
            let info = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    pub fn temporary_credentials(&self) -> PyTemporaryCredentialClient {
        PyTemporaryCredentialClient {
            client: self.0.temporary_credentials(),
        }
    }

    pub fn list_volumes(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<Vec<VolumeInfo>> {
        let stream = self
            .0
            .list_volumes(catalog_name, schema_name)
            .with_max_results(max_results)
            .with_include_browse(include_browse);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let volumes =
                runtime.block_on(async move { stream.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(volumes)
        })
    }

    pub fn volume(
        &self,
        catalog_name: String,
        schema_name: String,
        volume_name: String,
    ) -> PyVolumeClient {
        PyVolumeClient {
            client: self.0.volume(catalog_name, schema_name, volume_name),
        }
    }

    pub fn create_volume(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        volume_name: String,
        volume_type: VolumeType,
        storage_location: Option<String>,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<VolumeInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let mut request =
                self.0
                    .create_volume(catalog_name, schema_name, volume_name, volume_type);
            if let Some(storage_location) = storage_location {
                request = request.with_storage_location(storage_location);
            }
            if let Some(comment) = comment {
                request = request.with_comment(comment);
            }
            let info = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }
}

#[pyclass(name = "TemporaryCredentialClient")]
pub struct PyTemporaryCredentialClient {
    client: TemporaryCredentialClient,
}

#[pymethods]
impl PyTemporaryCredentialClient {
    #[pyo3(signature = (table, operation))]
    pub fn temporary_table_credential(
        &self,
        py: Python,
        table: String,
        operation: String,
    ) -> PyUnityCatalogResult<(TemporaryCredential, String)> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let table_ref = TableReference::Name(table);
            let op = match operation.to_ascii_lowercase().as_str() {
                "read" => TableOperation::Read,
                "read_write" => TableOperation::ReadWrite,
                _ => {
                    return Err(PyUnityCatalogError::from(
                        unitycatalog_common::error::Error::invalid_argument(format!(
                            "Invalid operation: {}. Must be 'read' or 'read_write'",
                            operation
                        )),
                    ));
                }
            };

            let (credential, uuid) =
                runtime.block_on(self.client.temporary_table_credential(table_ref, op))?;
            Ok::<_, PyUnityCatalogError>((credential, uuid.to_string()))
        })
    }

    #[pyo3(signature = (path, operation, dry_run = None))]
    pub fn temporary_path_credential(
        &self,
        py: Python,
        path: String,
        operation: String,
        dry_run: Option<bool>,
    ) -> PyUnityCatalogResult<(TemporaryCredential, String)> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let op = match operation.as_str().to_ascii_lowercase().as_str() {
                "read" => PathOperation::Read,
                "read_write" => PathOperation::ReadWrite,
                "create_table" => PathOperation::CreateTable,
                _ => return Err(PyUnityCatalogError::from(
                    unitycatalog_common::error::Error::invalid_argument(
                        format!("Invalid operation: {}. Must be 'read', 'read_write', or 'create_table'", operation)
                    )
                )),
            };

            let (credential, url) = runtime.block_on(
                self.client.temporary_path_credential(path, op, dry_run)
            )?;
            Ok::<_, PyUnityCatalogError>((credential, url.to_string()))
        })
    }
}
