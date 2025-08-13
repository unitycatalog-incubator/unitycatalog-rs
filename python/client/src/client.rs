use std::collections::HashMap;

use chrono::{DateTime, Utc};
use futures::stream::TryStreamExt;
use pyo3::prelude::*;
use unitycatalog_common::api::sharing::{MetadataResponseData, ProtocolResponseData};
use unitycatalog_common::client::{
    CatalogClient, CredentialClient, DeltaSharingClient, ExternalLocationClient, RecipientClient,
    SchemaClient, ShareClient, TableClient, UnityCatalogClient,
};
use unitycatalog_common::models::catalogs::v1::CatalogInfo;
use unitycatalog_common::models::credentials::v1::{CredentialInfo, Purpose as CredentialPurpose};
use unitycatalog_common::models::external_locations::v1::ExternalLocationInfo;
use unitycatalog_common::models::google::protobuf::{Struct, Value, value::Kind as ValueKind};
use unitycatalog_common::models::recipients::v1::{AuthenticationType, RecipientInfo};
use unitycatalog_common::models::schemas::v1::SchemaInfo;
use unitycatalog_common::models::shares::v1::{DataObjectUpdate, ShareInfo};
use unitycatalog_common::models::sharing::v1::{Share, SharingSchema, SharingTable};
use unitycatalog_common::models::tables::v1::{ColumnInfo, DataSourceFormat, TableInfo, TableType};

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
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let catalogs = runtime.block_on(async move {
                self.0
                    .list_catalogs(max_results)
                    .try_collect::<Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(catalogs)
        })
    }

    pub fn catalog(&self, name: String) -> PyCatalogClient {
        PyCatalogClient {
            client: self.0.catalog(&name),
        }
    }

    #[pyo3(signature = (catalog_name, max_results = None))]
    pub fn list_schemas(
        &self,
        py: Python,
        catalog_name: String,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<SchemaInfo>> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let schemas = runtime.block_on(async move {
                self.0
                    .list_schemas(catalog_name, max_results)
                    .try_collect::<Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(schemas)
        })
    }

    pub fn schema(&self, catalog_name: String, schema_name: String) -> PySchemaClient {
        PySchemaClient {
            client: self.0.schema(&catalog_name, &schema_name),
        }
    }

    #[pyo3(signature = (catalog_name, schema_name, max_results = None, include_delta_metadata = None, omit_columns = None, omit_properties = None, omit_username = None))]
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
    ) -> PyUnityCatalogResult<Vec<TableInfo>> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let tables = runtime.block_on(async move {
                self.0
                    .list_tables(
                        catalog_name,
                        schema_name,
                        max_results,
                        include_delta_metadata,
                        omit_columns,
                        omit_properties,
                        omit_username,
                    )
                    .try_collect::<Vec<_>>()
                    .await
            })?;
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
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let shares = runtime.block_on(async move {
                self.0
                    .list_shares(max_results)
                    .try_collect::<Vec<_>>()
                    .await
            })?;
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
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let recipients = runtime.block_on(async move {
                self.0
                    .list_recipients(max_results)
                    .try_collect::<Vec<_>>()
                    .await
            })?;
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
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let credentials = runtime.block_on(async move {
                self.0
                    .list_credentials(purpose, max_results)
                    .try_collect::<Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(credentials)
        })
    }

    pub fn credential(&self, name: String) -> PyCredentialClient {
        PyCredentialClient {
            client: self.0.credential(&name),
        }
    }

    #[pyo3(signature = (max_results = None))]
    pub fn list_external_locations(
        &self,
        py: Python,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<ExternalLocationInfo>> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let locations = runtime.block_on(async move {
                self.0
                    .list_external_locations(max_results)
                    .try_collect::<Vec<_>>()
                    .await
            })?;
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
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info =
                runtime.block_on(
                    self.0
                        .create_catalog(name, storage_root, comment, properties),
                )?;
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
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.0.create_sharing_catalog(
                name,
                provider_name,
                share_name,
                comment,
                properties,
            ))?;
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
            let info =
                runtime.block_on(self.0.create_schema(catalog_name, schema_name, comment))?;
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
            let info = runtime.block_on(self.0.create_share(name, comment))?;
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
            let info =
                runtime.block_on(self.0.create_recipient(name, authentication_type, comment))?;
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
            let info = runtime.block_on(self.0.create_credential(name, purpose, comment))?;
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
            let info = runtime.block_on(self.0.create_external_location(
                name,
                url,
                credential_name,
                comment,
            ))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }
}

#[pyclass(name = "CatalogClient")]
pub struct PyCatalogClient {
    client: CatalogClient,
}

#[pymethods]
impl PyCatalogClient {
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<CatalogInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.get())?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (name, comment = None))]
    pub fn create_schema(
        &self,
        py: Python,
        name: String,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<SchemaInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.create_schema(name, comment))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (new_name = None, comment = None, owner = None, properties = None))]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        comment: Option<String>,
        owner: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> PyUnityCatalogResult<CatalogInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info =
                runtime.block_on(self.client.update(new_name, comment, owner, properties))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (force = None))]
    pub fn delete(&self, py: Python, force: Option<bool>) -> PyUnityCatalogResult<()> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(self.client.delete(force))?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }

    pub fn schema(&self, name: String) -> PySchemaClient {
        PySchemaClient {
            client: self.client.schema(name),
        }
    }
}

#[pyclass(name = "SchemaClient")]
pub struct PySchemaClient {
    client: SchemaClient,
}

#[pymethods]
impl PySchemaClient {
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<SchemaInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.get())?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (new_name = None, comment = None, properties = None))]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> PyUnityCatalogResult<SchemaInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.update(new_name, comment, properties))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (name, table_type, data_source_format, columns, storage_location = None, comment = None, properties = None))]
    pub fn create_table(
        &self,
        py: Python,
        name: String,
        table_type: TableType,
        data_source_format: DataSourceFormat,
        columns: Vec<ColumnInfo>,
        storage_location: Option<String>,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> PyUnityCatalogResult<TableInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.create_table(
                name,
                table_type,
                data_source_format,
                columns,
                storage_location,
                comment,
                properties,
            ))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (force = None))]
    pub fn delete(&self, py: Python, force: Option<bool>) -> PyUnityCatalogResult<()> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(self.client.delete(force))?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}

#[pyclass(name = "TableClient")]
pub struct PyTableClient {
    client: TableClient,
}

#[pymethods]
impl PyTableClient {
    #[pyo3(signature = (include_delta_metadata = None))]
    pub fn get(
        &self,
        py: Python,
        include_delta_metadata: Option<bool>,
    ) -> PyUnityCatalogResult<TableInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.get(include_delta_metadata))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    pub fn delete(&self, py: Python) -> PyUnityCatalogResult<()> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(self.client.delete())?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}

#[pyclass(name = "ShareClient")]
pub struct PyShareClient {
    client: ShareClient,
}

#[pymethods]
impl PyShareClient {
    #[pyo3(signature = (include_shared_data = None))]
    pub fn get(
        &self,
        py: Python,
        include_shared_data: Option<bool>,
    ) -> PyUnityCatalogResult<ShareInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.get(include_shared_data))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (new_name = None, updates = None, comment = None, owner = None))]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        updates: Option<Vec<DataObjectUpdate>>,
        comment: Option<String>,
        owner: Option<String>,
    ) -> PyUnityCatalogResult<ShareInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.update(
                new_name,
                updates.unwrap_or_default(),
                comment,
                owner,
            ))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    pub fn delete(&self, py: Python) -> PyUnityCatalogResult<()> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(self.client.delete())?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}

#[pyclass(name = "RecipientClient")]
pub struct PyRecipientClient {
    client: RecipientClient,
}

#[pymethods]
impl PyRecipientClient {
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<RecipientInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.get())?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (new_name = None, comment = None, owner = None, properties = None, expiration_time = None))]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        comment: Option<String>,
        owner: Option<String>,
        properties: Option<HashMap<String, String>>,
        expiration_time: Option<i64>,
    ) -> PyUnityCatalogResult<RecipientInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let props = properties.map(hash_map_to_struct);
            let info = runtime.block_on(self.client.update(
                new_name,
                comment,
                owner,
                props,
                expiration_time,
            ))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    pub fn delete(&self, py: Python) -> PyUnityCatalogResult<()> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(self.client.delete())?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}

#[pyclass(name = "CredentialClient")]
pub struct PyCredentialClient {
    client: CredentialClient,
}

#[pymethods]
impl PyCredentialClient {
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<CredentialInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.get())?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (new_name = None, comment = None, owner = None, read_only = None, skip_validation = None, force = None, credential = None))]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        comment: Option<String>,
        owner: Option<String>,
        read_only: Option<bool>,
        skip_validation: Option<bool>,
        force: Option<bool>,
        credential: Option<
            unitycatalog_common::models::credentials::v1::update_credential_request::Credential,
        >,
    ) -> PyUnityCatalogResult<CredentialInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.update(
                new_name,
                comment,
                owner,
                read_only,
                skip_validation,
                force,
                credential,
            ))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    pub fn delete(&self, py: Python) -> PyUnityCatalogResult<()> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(self.client.delete())?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}

#[pyclass(name = "ExternalLocationClient")]
pub struct PyExternalLocationClient {
    client: ExternalLocationClient,
}

#[pymethods]
impl PyExternalLocationClient {
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<ExternalLocationInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.get())?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (new_name = None, url = None, credential_name = None, comment = None, owner = None, read_only = None, skip_validation = None, force = None))]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        url: Option<String>,
        credential_name: Option<String>,
        comment: Option<String>,
        owner: Option<String>,
        read_only: Option<bool>,
        skip_validation: Option<bool>,
        force: Option<bool>,
    ) -> PyUnityCatalogResult<ExternalLocationInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.update(
                new_name,
                url,
                credential_name,
                comment,
                owner,
                read_only,
                skip_validation,
                force,
            ))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (force = None))]
    pub fn delete(&self, py: Python, force: Option<bool>) -> PyUnityCatalogResult<()> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(self.client.delete(force))?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}

#[pyclass(name = "Protocol")]
pub struct PyProtocol {
    protocol: ProtocolResponseData,
}

#[pymethods]
impl PyProtocol {
    pub fn min_reader_version(&self) -> i32 {
        self.protocol.min_reader_version()
    }

    pub fn min_writer_version(&self) -> Option<i32> {
        self.protocol.min_writer_version()
    }

    pub fn __repr__(&self) -> String {
        format!(
            "PyProtocol(min_reader_version={}, min_writer_version={:?})",
            self.min_reader_version(),
            self.min_writer_version()
        )
    }
}

#[pyclass(name = "Metadata")]
pub struct PyMetadata {
    metadata: MetadataResponseData,
}

#[pymethods]
impl PyMetadata {
    pub fn partition_columns(&self) -> Vec<String> {
        self.metadata.partition_columns().to_vec()
    }

    pub fn configuration(&self) -> HashMap<String, String> {
        self.metadata.configuration().clone()
    }

    pub fn __repr__(&self) -> String {
        format!(
            "PyMetadata(partition_columns={:?}, configuration={:?})",
            self.partition_columns(),
            self.configuration()
        )
    }
}

#[pyclass]
pub struct PySharingClient {
    client: DeltaSharingClient,
}

#[pymethods]
impl PySharingClient {
    #[new]
    #[pyo3(signature = (base_url, token = None, prefix = None))]
    pub fn new(base_url: String, token: Option<String>, prefix: Option<String>) -> PyResult<Self> {
        let client = if let Some(token) = token {
            cloud_client::CloudClient::new_with_token(token)
        } else {
            cloud_client::CloudClient::new_unauthenticated()
        };
        let base_url = base_url.parse().unwrap();
        let sharing_client = if let Some(prefix) = prefix {
            DeltaSharingClient::new_with_prefix(client, base_url, prefix)
        } else {
            DeltaSharingClient::new(client, base_url)
        };
        Ok(Self {
            client: sharing_client,
        })
    }

    #[pyo3(signature = (max_results = None))]
    pub fn list_shares(
        &self,
        py: Python,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<Share>> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let shares = runtime.block_on(async move {
                self.client
                    .list_shares(max_results)
                    .try_collect::<Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(shares)
        })
    }

    pub fn get_share(&self, py: Python, name: String) -> PyUnityCatalogResult<Share> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let share = runtime.block_on(self.client.get_share(name))?;
            Ok::<_, PyUnityCatalogError>(share)
        })
    }

    #[pyo3(signature = (share, max_results = None))]
    pub fn list_share_schemas(
        &self,
        py: Python,
        share: String,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<SharingSchema>> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let schemas = runtime.block_on(async move {
                self.client
                    .list_share_schemas(share, max_results)
                    .try_collect::<Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(schemas)
        })
    }

    #[pyo3(signature = (share, max_results = None))]
    pub fn list_share_tables(
        &self,
        py: Python,
        share: String,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<SharingTable>> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let tables = runtime.block_on(async move {
                self.client
                    .list_share_tables(share, max_results)
                    .try_collect::<Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(tables)
        })
    }

    #[pyo3(signature = (share, schema, max_results = None))]
    pub fn list_schema_tables(
        &self,
        py: Python,
        share: String,
        schema: String,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<SharingTable>> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let tables = runtime.block_on(async move {
                self.client
                    .list_schema_tables(share, schema, max_results)
                    .try_collect::<Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(tables)
        })
    }

    #[pyo3(signature = (share, schema, table, starting_timestamp = None))]
    pub fn get_table_version(
        &self,
        py: Python,
        share: String,
        schema: String,
        table: String,
        starting_timestamp: Option<String>,
    ) -> PyUnityCatalogResult<u64> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let timestamp = starting_timestamp
                .map(|s| s.parse::<DateTime<Utc>>())
                .transpose()
                .map_err(|e| {
                    unitycatalog_common::error::Error::invalid_argument(format!(
                        "Invalid timestamp: {}",
                        e
                    ))
                })?;
            let version = runtime.block_on(
                self.client
                    .get_table_version(share, schema, table, timestamp),
            )?;
            Ok::<_, PyUnityCatalogError>(version)
        })
    }

    #[pyo3(signature = (share, schema, table))]
    pub fn get_table_metadata(
        &self,
        py: Python,
        share: String,
        schema: String,
        table: String,
    ) -> PyUnityCatalogResult<(PyProtocol, PyMetadata)> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let (protocol, metadata) =
                runtime.block_on(self.client.get_table_metadata(share, schema, table))?;
            Ok::<_, PyUnityCatalogError>((PyProtocol { protocol }, PyMetadata { metadata }))
        })
    }
}

fn hash_map_to_struct(map: HashMap<String, String>) -> Struct {
    let fields = map
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                Value {
                    kind: Some(ValueKind::StringValue(v)),
                },
            )
        })
        .collect();
    Struct { fields }
}
