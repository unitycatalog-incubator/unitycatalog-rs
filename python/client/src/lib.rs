use self::client::{
    PyCatalogClient, PyCredentialsClient, PyExternalLocationsClient, PyRecipientsClient,
    PySchemasClient, PySharesClient, PyTablesClient, PyUnityCatalogClient,
};
use pyo3::prelude::*;
use unitycatalog_common::models::catalogs::v1::{CatalogInfo, CatalogType};
use unitycatalog_common::models::credentials::v1::{
    AzureManagedIdentity, AzureServicePrincipal, AzureStorageKey, CredentialInfo, Purpose,
};
use unitycatalog_common::models::external_locations::v1::ExternalLocationInfo;
use unitycatalog_common::models::recipients::v1::RecipientInfo;
use unitycatalog_common::models::schemas::v1::SchemaInfo;
use unitycatalog_common::models::shares::v1::{
    DataObject, DataObjectType, DataObjectUpdate, HistoryStatus, ShareInfo,
};
use unitycatalog_common::models::tables::v1::{
    ColumnInfo, ColumnTypeName, DataSourceFormat, TableInfo, TableType,
};

mod client;
mod error;
mod runtime;

/// A Python module implemented in Rust.
#[pymodule]
fn unitycatalog_client(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // objects and enums
    m.add_class::<CatalogInfo>()?;
    m.add_class::<CatalogType>()?;
    m.add_class::<CredentialInfo>()?;
    m.add_class::<Purpose>()?;
    m.add_class::<AzureManagedIdentity>()?;
    m.add_class::<AzureServicePrincipal>()?;
    m.add_class::<AzureStorageKey>()?;
    m.add_class::<ExternalLocationInfo>()?;
    m.add_class::<RecipientInfo>()?;
    m.add_class::<SchemaInfo>()?;
    m.add_class::<ShareInfo>()?;
    m.add_class::<DataObject>()?;
    m.add_class::<DataObjectUpdate>()?;
    m.add_class::<DataObjectType>()?;
    m.add_class::<HistoryStatus>()?;
    m.add_class::<TableInfo>()?;
    m.add_class::<TableType>()?;
    m.add_class::<ColumnInfo>()?;
    m.add_class::<ColumnTypeName>()?;
    m.add_class::<DataSourceFormat>()?;

    // service clients
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
