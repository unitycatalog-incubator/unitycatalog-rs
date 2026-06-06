// @generated — do not edit by hand.
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
    #[pyo3(signature = (include_shared_data = None))]
    pub fn get(
        &self,
        py: Python,
        include_shared_data: Option<bool>,
    ) -> PyUnityCatalogResult<Share> {
        let mut request = self.client.get();
        request = request.with_include_shared_data(include_shared_data);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(signature = (updates = None, new_name = None, owner = None, comment = None))]
    pub fn update(
        &self,
        py: Python,
        updates: Option<Vec<DataObjectUpdate>>,
        new_name: Option<String>,
        owner: Option<String>,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<Share> {
        let mut request = self.client.update();
        if let Some(updates) = updates {
            request = request.with_updates(updates);
        }
        request = request.with_new_name(new_name);
        request = request.with_owner(owner);
        request = request.with_comment(comment);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    pub fn delete(&self, py: Python) -> PyUnityCatalogResult<()> {
        let request = self.client.delete();
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
    #[pyo3(signature = (max_results = None, page_token = None))]
    pub fn get_permissions(
        &self,
        py: Python,
        max_results: Option<i32>,
        page_token: Option<String>,
    ) -> PyUnityCatalogResult<GetPermissionsResponse> {
        let mut request = self.client.get_permissions();
        request = request.with_max_results(max_results);
        request = request.with_page_token(page_token);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(signature = (changes = None, omit_permissions_list = None))]
    pub fn update_permissions(
        &self,
        py: Python,
        changes: Option<Vec<PermissionsChange>>,
        omit_permissions_list: Option<bool>,
    ) -> PyUnityCatalogResult<UpdatePermissionsResponse> {
        let mut request = self.client.update_permissions();
        if let Some(changes) = changes {
            request = request.with_changes(changes);
        }
        request = request.with_omit_permissions_list(omit_permissions_list);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
}
impl PyShareClient {
    pub fn new(client: ShareClient) -> Self {
        Self { client }
    }
}
