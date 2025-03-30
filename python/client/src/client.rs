use pyo3::prelude::*;
use unitycatalog_common::models::catalogs::v1::CatalogInfo;
use unitycatalog_common::rest::client::{
    CatalogClient, CredentialsClient, ExternalLocationsClient, RecipientsClient, SchemasClient,
    SharesClient, TablesClient, UnityCatalogClient,
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
            client: self.0.catalogs(),
        }
    }

    #[getter]
    pub fn credentials(&self) -> PyCredentialsClient {
        PyCredentialsClient(self.0.credentials())
    }

    #[getter]
    pub fn external_locations(&self) -> PyExternalLocationsClient {
        PyExternalLocationsClient(self.0.external_locations())
    }

    #[getter]
    pub fn recipients(&self) -> PyRecipientsClient {
        PyRecipientsClient(self.0.recipients())
    }

    #[getter]
    pub fn schemas(&self) -> PySchemasClient {
        PySchemasClient(self.0.schemas())
    }

    #[getter]
    pub fn shares(&self) -> PySharesClient {
        PySharesClient(self.0.shares())
    }

    #[getter]
    pub fn tables(&self) -> PyTablesClient {
        PyTablesClient(self.0.tables())
    }
}

#[pyclass(name = "CatalogClient")]
pub struct PyCatalogClient {
    client: CatalogClient,
    name: String,
}

#[pymethods]
impl PyCatalogClient {
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<CatalogInfo> {
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let info = runtime.block_on(self.client.get(&self.name))?;
            Ok::<_, PyUnityCatalogError>(info)
        })
    }
}

#[pyclass(name = "CredentialsClient")]
pub struct PyCredentialsClient(CredentialsClient);

#[pyclass(name = "ExternalLocationsClient")]
pub struct PyExternalLocationsClient(ExternalLocationsClient);

#[pyclass(name = "RecipientsClient")]
pub struct PyRecipientsClient(RecipientsClient);

#[pyclass(name = "SchemasClient")]
pub struct PySchemasClient(SchemasClient);

#[pyclass(name = "SharesClient")]
pub struct PySharesClient(SharesClient);

#[pyclass(name = "TablesClient")]
pub struct PyTablesClient(TablesClient);
