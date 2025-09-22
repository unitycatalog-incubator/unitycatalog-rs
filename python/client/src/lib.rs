use self::client::{
    PyCatalogClient, PyCredentialClient, PyExternalLocationClient, PyRecipientClient,
    PySchemaClient, PyShareClient, PyTableClient, PyUnityCatalogClient, PyVolumeClient,
};
use pyo3::prelude::*;
use unitycatalog_common::models::catalogs::v1::{Catalog, CatalogType};
use unitycatalog_common::models::credentials::v1::{
    AzureManagedIdentity, AzureServicePrincipal, AzureStorageKey, Credential, Purpose,
};
use unitycatalog_common::models::external_locations::v1::ExternalLocation;
use unitycatalog_common::models::recipients::v1::Recipient;
use unitycatalog_common::models::schemas::v1::Schema;
use unitycatalog_common::models::shares::v1::{
    Action, DataObject, DataObjectType, DataObjectUpdate, HistoryStatus, Share,
};
use unitycatalog_common::models::tables::v1::{
    Column, ColumnTypeName, DataSourceFormat, Table, TableType,
};
use unitycatalog_common::models::volumes::v1::{Volume, VolumeType};

mod client;
mod codegen;
mod error;
mod runtime;

/// A Python module implemented in Rust.
#[pymodule]
fn unitycatalog_client(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // objects and enums
    m.add_class::<Catalog>()?;
    m.add_class::<CatalogType>()?;
    m.add_class::<Credential>()?;
    m.add_class::<Purpose>()?;
    m.add_class::<AzureManagedIdentity>()?;
    m.add_class::<AzureServicePrincipal>()?;
    m.add_class::<AzureStorageKey>()?;
    m.add_class::<ExternalLocation>()?;
    m.add_class::<Recipient>()?;
    m.add_class::<Schema>()?;
    m.add_class::<Share>()?;
    m.add_class::<DataObject>()?;
    m.add_class::<DataObjectUpdate>()?;
    m.add_class::<DataObjectType>()?;
    m.add_class::<HistoryStatus>()?;
    m.add_class::<Action>()?;
    m.add_class::<Table>()?;
    m.add_class::<TableType>()?;
    m.add_class::<Column>()?;
    m.add_class::<ColumnTypeName>()?;
    m.add_class::<DataSourceFormat>()?;
    m.add_class::<Volume>()?;
    m.add_class::<VolumeType>()?;

    // service clients
    m.add_class::<PyCatalogClient>()?;
    m.add_class::<PyCredentialClient>()?;
    m.add_class::<PyExternalLocationClient>()?;
    m.add_class::<PyRecipientClient>()?;
    m.add_class::<PySchemaClient>()?;
    m.add_class::<PyShareClient>()?;
    m.add_class::<PyTableClient>()?;
    m.add_class::<PyUnityCatalogClient>()?;
    m.add_class::<PyVolumeClient>()?;

    Ok(())
}
