use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use pyo3::prelude::*;
use unitycatalog_client::CredentialClient;
use unitycatalog_common::models::credentials::v1::*;
#[pyclass(name = "CredentialClient")]
pub struct PyCredentialClient {
    pub(crate) client: CredentialClient,
}
#[pymethods]
impl PyCredentialClient {
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<CredentialInfo> {
        let request = self.client.get();
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
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
        let mut request = self.client.update();
        request = request.with_new_name(new_name);
        request = request.with_comment(comment);
        request = request.with_read_only(read_only);
        request = request.with_owner(owner);
        request = request.with_skip_validation(skip_validation);
        request = request.with_force(force);
        request = request.with_azure_service_principal(azure_service_principal);
        request = request.with_azure_managed_identity(azure_managed_identity);
        request = request.with_azure_storage_key(azure_storage_key);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    pub fn delete_credential(&self, py: Python) -> PyUnityCatalogResult<()> {
        let request = self.client.delete();
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}
impl PyCredentialClient {
    pub fn new(client: CredentialClient) -> Self {
        Self { client }
    }
}
