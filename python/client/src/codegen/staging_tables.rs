// @generated — do not edit by hand.
use pyo3::prelude::*;
use unitycatalog_client::StagingTableClient;
#[pyclass(name = "StagingTableClient")]
pub struct PyStagingTableClient {
    pub(crate) client: StagingTableClient,
}
#[pymethods]
impl PyStagingTableClient {}
impl PyStagingTableClient {
    pub fn new(client: StagingTableClient) -> Self {
        Self { client }
    }
}
