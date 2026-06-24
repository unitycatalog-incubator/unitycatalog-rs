// @generated — do not edit by hand.
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::FunctionClient;
use unitycatalog_common::models::functions::v1::*;
#[pyclass(name = "FunctionClient")]
pub struct PyFunctionClient {
    pub(crate) client: FunctionClient,
}
#[pymethods]
impl PyFunctionClient {
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<Function> {
        let request = self.client.get();
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(signature = (owner = None))]
    pub fn update(&self, py: Python, owner: Option<String>) -> PyUnityCatalogResult<Function> {
        let mut request = self.client.update();
        request = request.with_owner(owner);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
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
impl PyFunctionClient {
    pub fn new(client: FunctionClient) -> Self {
        Self { client }
    }
}
