use pyo3::prelude::*;

use crate::models::catalogs::v1::CreateCatalogRequest;

#[pymethods]
impl CreateCatalogRequest {
    /// Create a new [`CreateCatalogRequest`]
    #[new]
    #[pyo3(signature = (*, name, comment = None, storage_root = None, provider_name = None, share_name = None))]
    pub fn new(
        name: String,
        comment: Option<String>,
        storage_root: Option<String>,
        provider_name: Option<String>,
        share_name: Option<String>,
    ) -> Self {
        Self {
            name,
            comment,
            properties: None,
            storage_root,
            provider_name,
            share_name,
        }
    }
}
