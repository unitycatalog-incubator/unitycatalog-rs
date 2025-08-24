use pyo3::prelude::*;
use unitycatalog_client::ExternalLocationClient;
#[pyclass(name = "ExternalLocationClient")]
pub struct PyExternalLocationClient {
    pub(crate) client: ExternalLocationClient,
}
#[pymethods]
impl PyExternalLocationClient {}
impl PyExternalLocationClient {
    pub fn new(client: ExternalLocationClient) -> Self {
        Self { client }
    }
}
