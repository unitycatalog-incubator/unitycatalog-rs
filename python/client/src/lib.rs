use pyo3::prelude::*;
use unitycatalog_common::models::catalogs::v1::{CatalogInfo, CreateCatalogRequest};

/// A Python module implemented in Rust.
#[pymodule]
fn unitycatalog_client(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<CatalogInfo>()?;
    m.add_class::<CreateCatalogRequest>()?;
    Ok(())
}
