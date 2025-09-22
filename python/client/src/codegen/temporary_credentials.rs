use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::TemporaryCredentialClient;
use unitycatalog_common::models::temporary_credentials::v1::*;
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
