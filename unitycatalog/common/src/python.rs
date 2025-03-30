use pyo3::prelude::*;

use crate::models::credentials::v1::{
    AzureManagedIdentity, AzureServicePrincipal, AzureStorageKey,
    azure_managed_identity::Identifier, azure_service_principal::Credential as SpCredential,
};
use crate::models::shares::v1::{
    Action as ShareUpdateAction, DataObject, DataObjectType, DataObjectUpdate, HistoryStatus,
};

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
}
