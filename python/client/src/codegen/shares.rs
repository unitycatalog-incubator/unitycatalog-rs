use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use pyo3::prelude::*;
use unitycatalog_client::ShareClient;
use unitycatalog_common::models::shares::v1::*;
#[pyclass(name = "ShareClient")]
pub struct PyShareClient {
    pub(crate) client: ShareClient,
}
#[pymethods]
impl PyShareClient {
    pub fn get(
        &self,
        py: Python,
        include_shared_data: Option<bool>,
    ) -> PyUnityCatalogResult<ShareInfo> {
        let mut request = self.client.get();
        request = request.with_include_shared_data(include_shared_data);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    pub fn update(
        &self,
        py: Python,
        updates: Option<DataObjectUpdate>,
        new_name: Option<String>,
        owner: Option<String>,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<ShareInfo> {
        let mut request = self.client.update();
        request = request.with_updates(updates);
        request = request.with_new_name(new_name);
        request = request.with_owner(owner);
        request = request.with_comment(comment);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    pub fn delete_share(&self, py: Python) -> PyUnityCatalogResult<()> {
        let request = self.client.delete();
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}
impl PyShareClient {
    pub fn new(client: ShareClient) -> Self {
        Self { client }
    }
}
