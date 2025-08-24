use pyo3::prelude::*;
use unitycatalog_client::TemporaryCredentialClient;
#[pyclass(name = "TemporaryCredentialClient")]
pub struct PyTemporaryCredentialClient {
    pub(crate) client: TemporaryCredentialClient,
}
#[pymethods]
impl PyTemporaryCredentialClient {}
impl PyTemporaryCredentialClient {
    pub fn new(client: TemporaryCredentialClient) -> Self {
        Self { client }
    }
}
