use std::collections::HashMap;

use pyo3::prelude::*;
use unitycatalog_common::models::catalogs::v1::{
    CatalogInfo, CreateCatalogRequest, UpdateCatalogRequest,
};
use unitycatalog_common::models::credentials::v1::{
    create_credential_request::Credential,
    update_credential_request::Credential as UpdateCredential, AzureManagedIdentity,
    AzureServicePrincipal, AzureStorageKey, CreateCredentialRequest, CredentialInfo,
    Purpose as CredentialPurpose, UpdateCredentialRequest,
};
use unitycatalog_common::models::external_locations::v1::{
    CreateExternalLocationRequest, ExternalLocationInfo, UpdateExternalLocationRequest,
};
use unitycatalog_common::models::google::protobuf::{value::Kind as ValueKind, Struct, Value};
use unitycatalog_common::models::recipients::v1::RecipientInfo;
use unitycatalog_common::models::schemas::v1::{
    CreateSchemaRequest, SchemaInfo, UpdateSchemaRequest,
};
use unitycatalog_common::models::shares::v1::{DataObjectUpdate, ShareInfo, UpdateShareRequest};
use unitycatalog_common::models::tables::v1::{
    ColumnInfo, CreateTableRequest, DataSourceFormat, TableInfo, TableType,
};
use unitycatalog_common::rest::client::{
    CredentialsClient, ExternalLocationsClient, RecipientsClient, SharesClient, UnityCatalogClient,
};

use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;

#[pyclass(name = "UnityCatalogClient")]
pub struct PyUnityCatalogClient(UnityCatalogClient);

#[pymethods]
impl PyUnityCatalogClient {
    #[new]
    pub fn new(base_url: String) -> PyResult<Self> {
        let client = cloud_client::CloudClient::new_unauthenticated();
        let base_url = base_url.parse().unwrap();
        Ok(Self(UnityCatalogClient::new(client, base_url)))
    }

    pub fn catalogs(&self, name: String) -> PyCatalogClient {
        PyCatalogClient {
            name,
            client: self.0.clone(),
        }
    }

    pub fn credentials(&self, name: String) -> PyCredentialsClient {
        PyCredentialsClient {
            name,
            client: self.0.credentials(),
        }
    }

    pub fn external_locations(&self, name: String) -> PyExternalLocationsClient {
        PyExternalLocationsClient {
            name,
            client: self.0.external_locations(),
        }
    }

    pub fn recipients(&self, name: String) -> PyRecipientsClient {
        PyRecipientsClient {
            name,
            client: self.0.recipients(),
        }
    }

    pub fn shares(&self, name: String) -> PySharesClient {
        PySharesClient {
            name,
            client: self.0.shares(),
        }
    }
}

#[pyclass(name = "CatalogClient")]
pub struct PyCatalogClient {
    client: UnityCatalogClient,
    name: String,
}

#[pymethods]
impl PyCatalogClient {
    pub fn schemas(&self, name: String) -> PySchemasClient {
        PySchemasClient {
            name,
            catalog_name: self.name.clone(),
            client: self.client.clone(),
        }
    }

    pub fn get(&self, py: Python) -> PyUnityCatalogResult<CatalogInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.catalogs().get(&self.name))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (
        *,
        comment = None,
        storage_root = None,
        provider_name = None,
        share_name = None,
        properties = None
    ))]
    pub fn create(
        &self,
        py: Python,
        comment: Option<String>,
        storage_root: Option<String>,
        provider_name: Option<String>,
        share_name: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> PyUnityCatalogResult<CatalogInfo> {
        let request = CreateCatalogRequest {
            name: self.name.clone(),
            comment,
            properties: properties.map(hash_map_to_struct),
            storage_root,
            provider_name,
            share_name,
        };
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.catalogs().create_catalog(&request))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (*, new_name = None, comment = None, owner = None, properties = None))]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        comment: Option<String>,
        owner: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> PyUnityCatalogResult<CatalogInfo> {
        let request = UpdateCatalogRequest {
            name: self.name.clone(),
            comment,
            new_name: new_name.unwrap_or_else(|| self.name.clone()),
            owner,
            properties: properties.map(hash_map_to_struct),
        };
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.catalogs().update_catalog(&request))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (force = false))]
    pub fn delete(&self, py: Python, force: bool) -> PyUnityCatalogResult<()> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(self.client.catalogs().delete(&self.name, force))?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}

#[pyclass(name = "CredentialsClient")]
pub struct PyCredentialsClient {
    client: CredentialsClient,
    name: String,
}

#[pymethods]
impl PyCredentialsClient {
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<CredentialInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.get(&self.name))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (
        *,
        purpose,
        comment = None,
        read_only = None,
        skip_validation = false,
        azure_service_principal = None,
        azure_managed_identity = None,
        azure_storage_key = None
    ))]
    pub fn create(
        &self,
        py: Python,
        purpose: CredentialPurpose,
        comment: Option<String>,
        read_only: Option<bool>,
        skip_validation: bool,
        azure_service_principal: Option<AzureServicePrincipal>,
        azure_managed_identity: Option<AzureManagedIdentity>,
        azure_storage_key: Option<AzureStorageKey>,
    ) -> PyUnityCatalogResult<CredentialInfo> {
        let credential = if azure_service_principal.is_some() {
            Credential::AzureServicePrincipal(azure_service_principal.unwrap())
        } else if azure_managed_identity.is_some() {
            Credential::AzureManagedIdentity(azure_managed_identity.unwrap())
        } else if azure_storage_key.is_some() {
            Credential::AzureStorageKey(azure_storage_key.unwrap())
        } else {
            return Err(unitycatalog_common::error::Error::invalid_argument(
                "One of azure_service_principal, azure_managed_identity, or azure_storage_key must be provided"
            ).into());
        };
        let request = CreateCredentialRequest {
            name: self.name.clone(),
            purpose: purpose.into(),
            comment,
            read_only,
            skip_validation,
            credential: Some(credential),
        };
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.create_credential(&request))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (
        *,
        new_name = None,
        comment = None,
        read_only = None,
        owner = None,
        skip_validation = None,
        force = None,
        azure_service_principal = None,
        azure_managed_identity = None,
        azure_storage_key = None
    ))]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        comment: Option<String>,
        read_only: Option<bool>,
        owner: Option<String>,
        skip_validation: Option<bool>,
        force: Option<bool>,
        azure_service_principal: Option<AzureServicePrincipal>,
        azure_managed_identity: Option<AzureManagedIdentity>,
        azure_storage_key: Option<AzureStorageKey>,
    ) -> PyUnityCatalogResult<CredentialInfo> {
        let credential = if azure_service_principal.is_some() {
            Some(UpdateCredential::AzureServicePrincipal(
                azure_service_principal.unwrap(),
            ))
        } else if azure_managed_identity.is_some() {
            Some(UpdateCredential::AzureManagedIdentity(
                azure_managed_identity.unwrap(),
            ))
        } else if azure_storage_key.is_some() {
            Some(UpdateCredential::AzureStorageKey(
                azure_storage_key.unwrap(),
            ))
        } else {
            None
        };
        let request = UpdateCredentialRequest {
            name: self.name.clone(),
            new_name,
            comment,
            read_only,
            owner,
            skip_validation,
            force,
            credential,
        };
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.update_credential(&request))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }
}

#[pyclass(name = "ExternalLocationsClient")]
pub struct PyExternalLocationsClient {
    client: ExternalLocationsClient,
    name: String,
}

#[pymethods]
impl PyExternalLocationsClient {
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<ExternalLocationInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.get(&self.name))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (
        *,
        url,
        credential_name,
        read_only = None,
        comment = None,
        skip_validation = None
    ))]
    pub fn create(
        &self,
        py: Python,
        url: String,
        credential_name: String,
        read_only: Option<bool>,
        comment: Option<String>,
        skip_validation: Option<bool>,
    ) -> PyUnityCatalogResult<ExternalLocationInfo> {
        let request = CreateExternalLocationRequest {
            name: self.name.clone(),
            url,
            credential_name,
            read_only,
            comment,
            skip_validation,
        };
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.create_external_location(&request))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (
        *,
        url = None,
        credential_name = None,
        read_only = None,
        owner = None,
        comment = None,
        new_name = None,
        force = None,
        skip_validation = None
    ))]
    pub fn update(
        &self,
        py: Python,
        url: Option<String>,
        credential_name: Option<String>,
        read_only: Option<bool>,
        owner: Option<String>,
        comment: Option<String>,
        new_name: Option<String>,
        force: Option<bool>,
        skip_validation: Option<bool>,
    ) -> PyUnityCatalogResult<ExternalLocationInfo> {
        let request = UpdateExternalLocationRequest {
            name: self.name.clone(),
            url,
            credential_name,
            read_only,
            owner,
            comment,
            new_name,
            force,
            skip_validation,
        };
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.update_external_location(&request))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (force = None))]
    pub fn delete(&self, py: Python, force: Option<bool>) -> PyUnityCatalogResult<()> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(self.client.delete(&self.name, force))?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}

#[pyclass(name = "RecipientsClient")]
pub struct PyRecipientsClient {
    client: RecipientsClient,
    name: String,
}

#[pymethods]
impl PyRecipientsClient {
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<RecipientInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.get(&self.name))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }
}

#[pyclass(name = "SchemasClient")]
pub struct PySchemasClient {
    client: UnityCatalogClient,
    catalog_name: String,
    name: String,
}

#[pymethods]
impl PySchemasClient {
    pub fn tables(&self, name: String) -> PyTablesClient {
        PyTablesClient {
            name,
            schema_name: self.name.clone(),
            catalog_name: self.catalog_name.clone(),
            client: self.client.clone(),
        }
    }

    pub fn get(&self, py: Python) -> PyUnityCatalogResult<SchemaInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info =
                runtime.block_on(self.client.schemas().get(&self.catalog_name, &self.name))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (*, comment = None, properties = None))]
    pub fn create(
        &self,
        py: Python,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> PyUnityCatalogResult<SchemaInfo> {
        let request = CreateSchemaRequest {
            name: self.name.clone(),
            catalog_name: self.catalog_name.clone(),
            comment,
            properties: properties.map(hash_map_to_struct),
        };
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.schemas().create_schema(&request))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (*, new_name = None, comment = None, properties = None))]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> PyUnityCatalogResult<SchemaInfo> {
        let request = UpdateSchemaRequest {
            full_name: format!("{}.{}", self.catalog_name, self.name),
            comment,
            properties: properties.map(hash_map_to_struct),
            new_name: new_name.unwrap_or_else(|| self.name.clone()),
        };
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.schemas().update_schema(&request))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (force = false))]
    pub fn delete(&self, py: Python, force: bool) -> PyUnityCatalogResult<()> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(self.client.schemas().delete(
                &self.catalog_name,
                &self.name,
                force,
            ))?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}

#[pyclass(name = "SharesClient")]
pub struct PySharesClient {
    client: SharesClient,
    name: String,
}

#[pymethods]
impl PySharesClient {
    #[pyo3(signature = (include_shared_data = None))]
    pub fn get(
        &self,
        py: Python,
        include_shared_data: Option<bool>,
    ) -> PyUnityCatalogResult<ShareInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.get(&self.name, include_shared_data))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (*, comment = None))]
    pub fn create(&self, py: Python, comment: Option<String>) -> PyUnityCatalogResult<ShareInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.create(&self.name, comment))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (*, updates, new_name = None, comment = None, owner = None))]
    pub fn update(
        &self,
        py: Python,
        updates: Vec<DataObjectUpdate>,
        new_name: Option<String>,
        comment: Option<String>,
        owner: Option<String>,
    ) -> PyUnityCatalogResult<ShareInfo> {
        let request = UpdateShareRequest {
            name: self.name.clone(),
            new_name,
            updates,
            comment,
            owner,
        };
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.update_share(&request))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    pub fn delete(&self, py: Python) -> PyUnityCatalogResult<()> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(self.client.delete(&self.name))?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}

#[pyclass(name = "TablesClient")]
pub struct PyTablesClient {
    client: UnityCatalogClient,
    catalog_name: String,
    schema_name: String,
    name: String,
}

#[pymethods]
impl PyTablesClient {
    #[pyo3(signature = (include_delta_metadata = None))]
    pub fn get(
        &self,
        py: Python,
        include_delta_metadata: Option<bool>,
    ) -> PyUnityCatalogResult<TableInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let full_name = format!("{}.{}.{}", &self.catalog_name, self.schema_name, self.name);
            let info =
                runtime.block_on(self.client.tables().get(full_name, include_delta_metadata))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    #[pyo3(signature = (
        *,
        table_type,
        data_source_format,
        storage_location = None,
        comment = None,
        columns = None,
        properties = None
    ))]
    pub fn create(
        &self,
        py: Python,
        table_type: TableType,
        data_source_format: DataSourceFormat,
        storage_location: Option<String>,
        comment: Option<String>,
        columns: Option<Vec<ColumnInfo>>,
        properties: Option<HashMap<String, String>>,
    ) -> PyUnityCatalogResult<TableInfo> {
        let request = CreateTableRequest {
            name: self.name.clone(),
            schema_name: self.schema_name.clone(),
            catalog_name: self.catalog_name.clone(),
            table_type: table_type as i32,
            data_source_format: data_source_format as i32,
            columns: columns.unwrap_or_default(),
            storage_location,
            comment,
            properties: properties.map(hash_map_to_struct),
        };
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.tables().create_table(&request))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }

    pub fn delete(&self, py: Python) -> PyUnityCatalogResult<()> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let full_name = format!("{}.{}.{}", &self.catalog_name, self.schema_name, self.name);
            runtime.block_on(self.client.tables().delete(&full_name))?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }

    // pub fn exists(&self, py: Python) -> PyUnityCatalogResult<bool> {
    //     let runtime = get_runtime(py)?;
    //     py.allow_threads(|| {
    //         let full_name = format!("{}.{}.{}", &self.catalog_name, self.schema_name, self.name);
    //         let exists = runtime.block_on(self.client.tables().exists(&full_name))?;
    //         Ok::<_, PyUnityCatalogError>(exists)
    //     })
    // }
}

fn hash_map_to_struct(map: HashMap<String, String>) -> Struct {
    Struct {
        fields: map
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    Value {
                        kind: Some(ValueKind::StringValue(v.clone())),
                    },
                )
            })
            .collect(),
    }
}
