use self::client::{
    PyCatalogClient, PyCredentialsClient, PyExternalLocationsClient, PyRecipientsClient,
    PySchemasClient, PySharesClient, PyTablesClient, PyUnityCatalogClient,
};
use pyo3::prelude::*;
use unitycatalog_common::models::catalogs::v1::{CatalogInfo, CreateCatalogRequest};

mod client;
mod error;
mod runtime;

/// A Python module implemented in Rust.
#[pymodule]
fn unitycatalog_client(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<CatalogInfo>()?;

    // service clients
    m.add_class::<CreateCatalogRequest>()?;
    m.add_class::<PyCatalogClient>()?;
    m.add_class::<PyCredentialsClient>()?;
    m.add_class::<PyExternalLocationsClient>()?;
    m.add_class::<PyRecipientsClient>()?;
    m.add_class::<PySchemasClient>()?;
    m.add_class::<PySharesClient>()?;
    m.add_class::<PyTablesClient>()?;
    m.add_class::<PyUnityCatalogClient>()?;

    Ok(())
}
