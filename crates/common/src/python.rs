use pyo3::prelude::*;

use crate::models::catalogs::v1::CatalogInfo;
use crate::models::credentials::v1::{
    AzureManagedIdentity, AzureServicePrincipal, AzureStorageKey, CredentialInfo,
    azure_managed_identity::Identifier, azure_service_principal::Credential as SpCredential,
};
use crate::models::external_locations::v1::ExternalLocationInfo;
use crate::models::recipients::v1::{RecipientInfo, RecipientToken};
use crate::models::schemas::v1::SchemaInfo;
use crate::models::shares::v1::{
    Action as ShareUpdateAction, DataObject, DataObjectType, DataObjectUpdate, HistoryStatus, Share,
};
use crate::models::tables::v1::{ColumnInfo, TableInfo};

#[pymethods]
impl AzureStorageKey {
    #[new]
    pub fn new(account_name: String, account_key: String) -> Self {
        Self {
            account_name,
            account_key,
        }
    }
}

#[pymethods]
impl AzureServicePrincipal {
    #[new]
    #[pyo3(signature = (application_id, directory_id, client_secret = None, federated_token_file = None))]
    pub fn new(
        application_id: String,
        directory_id: String,
        client_secret: Option<String>,
        federated_token_file: Option<String>,
    ) -> Self {
        let credential = match (client_secret, federated_token_file) {
            (Some(client_secret), _) => Some(SpCredential::ClientSecret(client_secret)),
            (None, Some(file)) => Some(SpCredential::FederatedTokenFile(file)),
            (None, None) => None,
        };
        Self {
            application_id,
            credential,
            directory_id,
        }
    }
}

#[pymethods]
impl AzureManagedIdentity {
    #[new]
    pub fn new(
        application_id: Option<String>,
        object_id: Option<String>,
        msi_resource_id: Option<String>,
    ) -> Self {
        let identifier = match (application_id, object_id, msi_resource_id) {
            (Some(application_id), _, _) => Some(Identifier::ApplicationId(application_id)),
            (_, Some(object_id), _) => Some(Identifier::ObjectId(object_id)),
            (_, _, Some(msi_resource_id)) => Some(Identifier::MsiResourceId(msi_resource_id)),
            _ => None,
        };
        Self { identifier }
    }
}

#[pymethods]
impl DataObject {
    #[new]
    #[pyo3(signature = (
        name,
        data_object_type,
        *,
        added_at = None,
        added_by = None,
        comment = None,
        shared_as = None,
        partitions = None,
        enable_cdf = None,
        history_data_sharing_status = None,
        start_version = None
    ))]
    pub fn new(
        name: String,
        data_object_type: DataObjectType,
        added_at: Option<i64>,
        added_by: Option<String>,
        comment: Option<String>,
        shared_as: Option<String>,
        partitions: Option<Vec<String>>,
        enable_cdf: Option<bool>,
        history_data_sharing_status: Option<HistoryStatus>,
        start_version: Option<i64>,
    ) -> Self {
        Self {
            name,
            data_object_type: data_object_type as i32,
            added_at,
            added_by,
            comment,
            shared_as,
            partitions: partitions.unwrap_or_default(),
            enable_cdf,
            history_data_sharing_status: history_data_sharing_status.map(|s| s as i32),
            start_version,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "DataObject(name={}, data_object_type={}, added_at={}, added_by={}, comment={}, shared_as={}, partitions={}, enable_cdf={}, history_data_sharing_status={}, start_version={})",
            self.name,
            self.data_object_type,
            self.added_at
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.added_by.as_ref().unwrap_or(&"None".to_owned()),
            self.comment.as_ref().unwrap_or(&"None".to_owned()),
            self.shared_as.as_ref().unwrap_or(&"None".to_owned()),
            format!(
                "[{}]",
                self.partitions
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            self.enable_cdf
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.history_data_sharing_status
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.start_version
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
        )
    }
}

#[pymethods]
impl DataObjectUpdate {
    #[new]
    pub fn new(action: ShareUpdateAction, data_object: DataObject) -> Self {
        Self {
            action: action as i32,
            data_object: Some(data_object),
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "DataObjectUpdate(action={}, data_object={})",
            self.action,
            self.data_object
                .as_ref()
                .map_or("None".to_owned(), |n| n.__repr__())
        )
    }
}

#[pymethods]
impl CatalogInfo {
    pub fn __repr__(&self) -> String {
        format!(
            "CatalogInfo(id={}, name={}, owner={}, comment={}, storage_root={}, provider_name={}, share_name={}, catalog_type={}, created_at={}, created_by={}, updated_at={}, updated_by={})",
            self.id.as_ref().unwrap_or(&"None".to_owned()),
            self.name,
            self.owner.as_ref().unwrap_or(&"None".to_owned()),
            self.comment.as_ref().unwrap_or(&"None".to_owned()),
            self.storage_root.as_ref().unwrap_or(&"None".to_owned()),
            self.provider_name.as_ref().unwrap_or(&"None".to_owned()),
            self.share_name.as_ref().unwrap_or(&"None".to_owned()),
            self.catalog_type
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.created_at
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.created_by.as_ref().unwrap_or(&"None".to_owned()),
            self.updated_at
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.updated_by.as_ref().unwrap_or(&"None".to_owned())
        )
    }
}

#[pymethods]
impl SchemaInfo {
    pub fn __repr__(&self) -> String {
        format!(
            "SchemaInfo(name={}, catalog_name={}, comment={}, full_name={}, owner={}, created_at={}, created_by={}, updated_at={}, updated_by={}, schema_id={})",
            self.name,
            self.catalog_name,
            self.comment.as_ref().unwrap_or(&"None".to_owned()),
            self.full_name,
            self.owner.as_ref().unwrap_or(&"None".to_owned()),
            self.created_at
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.created_by.as_ref().unwrap_or(&"None".to_owned()),
            self.updated_at
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.updated_by.as_ref().unwrap_or(&"None".to_owned()),
            self.schema_id.as_ref().unwrap_or(&"None".to_owned())
        )
    }
}

#[pymethods]
impl TableInfo {
    pub fn __repr__(&self) -> String {
        format!(
            "TableInfo(name={}, catalog_name={}, schema_name={}, table_type={}, data_source_format={}, comment={}, owner={}, storage_location={}, created_at={}, created_by={}, updated_at={}, updated_by={}, table_id={})",
            self.name,
            self.catalog_name,
            self.schema_name,
            self.table_type,
            self.data_source_format,
            self.comment.as_ref().unwrap_or(&"None".to_owned()),
            self.owner.as_ref().unwrap_or(&"None".to_owned()),
            self.storage_location.as_ref().unwrap_or(&"None".to_owned()),
            self.created_at
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.created_by.as_ref().unwrap_or(&"None".to_owned()),
            self.updated_at
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.updated_by.as_ref().unwrap_or(&"None".to_owned()),
            self.table_id.as_ref().unwrap_or(&"None".to_owned())
        )
    }
}

#[pymethods]
impl ColumnInfo {
    pub fn __repr__(&self) -> String {
        format!(
            "ColumnInfo(name={}, type_text={}, type_json={}, type_name={}, type_precision={}, type_scale={}, type_interval_type={}, position={}, comment={}, nullable={}, partition_index={})",
            self.name,
            self.type_text,
            self.type_json,
            self.type_name,
            self.type_precision
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.type_scale
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.type_interval_type
                .as_ref()
                .unwrap_or(&"None".to_owned()),
            self.position
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.comment.as_ref().unwrap_or(&"None".to_owned()),
            self.nullable
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.partition_index
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string())
        )
    }
}

#[pymethods]
impl Share {
    pub fn __repr__(&self) -> String {
        format!(
            "Share(id={}, name={}, comment={}, owner={}, created_at={}, created_by={}, updated_at={}, updated_by={})",
            self.id.as_ref().unwrap_or(&"None".to_owned()),
            self.name,
            self.comment.as_ref().unwrap_or(&"None".to_owned()),
            self.owner.as_ref().unwrap_or(&"None".to_owned()),
            self.created_at
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.created_by.as_ref().unwrap_or(&"None".to_owned()),
            self.updated_at
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.updated_by.as_ref().unwrap_or(&"None".to_owned())
        )
    }
}

#[pymethods]
impl RecipientInfo {
    pub fn __repr__(&self) -> String {
        format!(
            "RecipientInfo(id={}, name={}, authentication_type={}, owner={}, comment={}, created_at={}, created_by={}, updated_at={}, updated_by={}, tokens={})",
            self.id.as_ref().unwrap_or(&"None".to_owned()),
            self.name,
            self.authentication_type,
            self.owner,
            self.comment.as_ref().unwrap_or(&"None".to_owned()),
            self.created_at
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.created_by.as_ref().unwrap_or(&"None".to_owned()),
            self.updated_at
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.updated_by.as_ref().unwrap_or(&"None".to_owned()),
            format!(
                "[{}]",
                self.tokens
                    .iter()
                    .map(|t| t.__repr__())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        )
    }
}

#[pymethods]
impl RecipientToken {
    pub fn __repr__(&self) -> String {
        format!(
            "RecipientToken(id={}, created_at={}, created_by={}, activation_url={}, expiration_time={}, updated_at={}, updated_by={})",
            self.id,
            self.created_at,
            self.created_by,
            self.activation_url,
            self.expiration_time,
            self.updated_at,
            self.updated_by
        )
    }
}

#[pymethods]
impl CredentialInfo {
    pub fn __repr__(&self) -> String {
        format!(
            "CredentialInfo(id={}, name={}, purpose={}, credential='***', read_only={}, owner={}, comment={}, created_at={}, created_by={}, updated_at={}, updated_by={})",
            self.id.as_ref().unwrap_or(&"None".to_owned()),
            self.name,
            self.purpose,
            self.read_only,
            self.owner.as_ref().unwrap_or(&"None".to_owned()),
            self.comment.as_ref().unwrap_or(&"None".to_owned()),
            self.created_at
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.created_by.as_ref().unwrap_or(&"None".to_owned()),
            self.updated_at
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.updated_by.as_ref().unwrap_or(&"None".to_owned())
        )
    }
}

#[pymethods]
impl ExternalLocationInfo {
    pub fn __repr__(&self) -> String {
        format!(
            "ExternalLocationInfo(name={}, url={}, credential_name={}, read_only={}, comment={}, owner={}, credential_id={}, created_at={}, created_by={}, updated_at={}, updated_by={}, browse_only={}, external_location_id={})",
            self.name,
            self.url,
            self.credential_name,
            self.read_only,
            self.comment.as_ref().unwrap_or(&"None".to_owned()),
            self.owner.as_ref().unwrap_or(&"None".to_owned()),
            self.credential_id,
            self.created_at
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.created_by.as_ref().unwrap_or(&"None".to_owned()),
            self.updated_at
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.updated_by.as_ref().unwrap_or(&"None".to_owned()),
            self.browse_only
                .as_ref()
                .map_or("None".to_owned(), |n| n.to_string()),
            self.external_location_id
                .as_ref()
                .unwrap_or(&"None".to_owned())
        )
    }
}
