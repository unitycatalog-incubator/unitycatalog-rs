use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use pyo3::prelude::*;
use unitycatalog_client::TableClient;
use unitycatalog_common::models::tables::v1::*;
#[pyclass(name = "TableClient")]
pub struct PyTableClient {
    pub(crate) client: TableClient,
}
#[pymethods]
impl PyTableClient {
    pub fn get(
        &self,
        py: Python,
        include_delta_metadata: Option<bool>,
        include_browse: Option<bool>,
        include_manifest_capabilities: Option<bool>,
    ) -> PyUnityCatalogResult<TableInfo> {
        let mut request = self.client.get();
        request = request.with_include_delta_metadata(include_delta_metadata);
        request = request.with_include_browse(include_browse);
        request = request.with_include_manifest_capabilities(include_manifest_capabilities);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    pub fn delete_table(&self, py: Python) -> PyUnityCatalogResult<()> {
        let request = self.client.delete();
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}
impl PyTableClient {
    pub fn new(client: TableClient) -> Self {
        Self { client }
    }
}
