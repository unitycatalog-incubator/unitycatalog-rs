use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::CatalogClient;
use unitycatalog_common::models::catalogs::v1::*;
#[pyclass(name = "CatalogClient")]
pub struct PyCatalogClient {
    pub(crate) client: CatalogClient,
}
#[pymethods]
impl PyCatalogClient {
    pub fn get(
        &self,
        py: Python,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<CatalogInfo> {
        let mut request = self.client.get();
        request = request.with_include_browse(include_browse);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    pub fn update(
        &self,
        py: Python,
        owner: Option<String>,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        new_name: Option<String>,
    ) -> PyUnityCatalogResult<CatalogInfo> {
        let mut request = self.client.update();
        request = request.with_owner(owner);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_new_name(new_name);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    pub fn delete_catalog(&self, py: Python, force: Option<bool>) -> PyUnityCatalogResult<()> {
        let mut request = self.client.delete();
        request = request.with_force(force);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}
impl PyCatalogClient {
    pub fn new(client: CatalogClient) -> Self {
        Self { client }
    }
}
