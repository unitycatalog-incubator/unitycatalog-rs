use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::SchemaClient;
use unitycatalog_common::models::schemas::v1::*;
#[pyclass(name = "SchemaClient")]
pub struct PySchemaClient {
    pub(crate) client: SchemaClient,
}
#[pymethods]
impl PySchemaClient {
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<Schema> {
        let request = self.client.get();
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(signature = (comment = None, properties = None, new_name = None))]
    pub fn update(
        &self,
        py: Python,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        new_name: Option<String>,
    ) -> PyUnityCatalogResult<Schema> {
        let mut request = self.client.update();
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
    #[pyo3(signature = (force = None))]
    pub fn delete(&self, py: Python, force: Option<bool>) -> PyUnityCatalogResult<()> {
        let mut request = self.client.delete();
        request = request.with_force(force);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}
impl PySchemaClient {
    pub fn new(client: SchemaClient) -> Self {
        Self { client }
    }
}
