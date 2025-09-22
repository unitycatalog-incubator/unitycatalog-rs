use pyo3::prelude::*;
use unitycatalog_client::{
    PathOperation, TableOperation, TableReference, TemporaryCredentialClient,
};
use unitycatalog_common::models::temporary_credentials::v1::TemporaryCredential;

pub use crate::codegen::PyUnityCatalogClient;
pub use crate::codegen::catalogs::PyCatalogClient;
pub use crate::codegen::credentials::PyCredentialClient;
pub use crate::codegen::external_locations::PyExternalLocationClient;
pub use crate::codegen::recipients::PyRecipientClient;
pub use crate::codegen::schemas::PySchemaClient;
pub use crate::codegen::shares::PyShareClient;
pub use crate::codegen::tables::PyTableClient;
pub use crate::codegen::volumes::PyVolumeClient;
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;

#[pyclass(name = "TemporaryCredentialClient")]
pub struct PyTemporaryCredentialClient {
    client: TemporaryCredentialClient,
}

#[pymethods]
impl PyTemporaryCredentialClient {
    #[pyo3(signature = (table, operation))]
    pub fn temporary_table_credential(
        &self,
        py: Python,
        table: String,
        operation: String,
    ) -> PyUnityCatalogResult<(TemporaryCredential, String)> {
        let table_ref = TableReference::Name(table);
        let op = match operation.to_ascii_lowercase().as_str() {
            "read" => TableOperation::Read,
            "read_write" => TableOperation::ReadWrite,
            _ => {
                return Err(PyUnityCatalogError::from(
                    unitycatalog_common::error::Error::invalid_argument(format!(
                        "Invalid operation: {}. Must be 'read' or 'read_write'",
                        operation
                    )),
                ));
            }
        };
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let (credential, uuid) =
                runtime.block_on(self.client.temporary_table_credential(table_ref, op))?;
            Ok::<_, PyUnityCatalogError>((credential, uuid.to_string()))
        })
    }

    #[pyo3(signature = (path, operation, dry_run = None))]
    pub fn temporary_path_credential(
        &self,
        py: Python,
        path: String,
        operation: String,
        dry_run: Option<bool>,
    ) -> PyUnityCatalogResult<(TemporaryCredential, String)> {
        let runtime = get_runtime(py)?;
        let op = match operation.as_str().to_ascii_lowercase().as_str() {
            "read" => PathOperation::Read,
            "read_write" => PathOperation::ReadWrite,
            "create_table" => PathOperation::CreateTable,
            _ => {
                return Err(PyUnityCatalogError::from(
                    unitycatalog_common::error::Error::invalid_argument(format!(
                        "Invalid operation: {}. Must be 'read', 'read_write', or 'create_table'",
                        operation
                    )),
                ));
            }
        };
        py.allow_threads(|| {
            let (credential, url) =
                runtime.block_on(self.client.temporary_path_credential(path, op, dry_run))?;
            Ok::<_, PyUnityCatalogError>((credential, url.to_string()))
        })
    }
}
