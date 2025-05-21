use pyo3::prelude::*;

use crate::models::credentials::v1::{
    AzureManagedIdentity, AzureServicePrincipal, AzureStorageKey,
    azure_managed_identity::Identifier, azure_service_principal::Credential as SpCredential,
};
use crate::models::shares::v1::{
    Action as ShareUpdateAction, DataObject, DataObjectType, DataObjectUpdate, HistoryStatus,
};
use crate::models::sharing::v1::{Share, SharingSchema, SharingTable};

#[pymethods]
impl Share {
    #[new]
    #[pyo3(signature = (name, id = None))]
    pub fn new(name: String, id: Option<String>) -> Self {
        Self { id, name }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Share(name={}, id={})",
            self.name,
            self.id.as_ref().unwrap_or(&"None".to_owned())
        )
    }
}

#[pymethods]
impl SharingSchema {
    #[new]
    #[pyo3(signature = (*, name, share, id = None))]
    pub fn new(name: String, share: String, id: Option<String>) -> Self {
        Self { id, name, share }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "SharingSchema(name={}, share={}, id={})",
            self.name,
            self.share,
            self.id.as_ref().unwrap_or(&"None".to_owned())
        )
    }
}

#[pymethods]
impl SharingTable {
    #[new]
    pub fn new(
        name: String,
        schema: String,
        share: String,
        share_id: Option<String>,
        id: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            schema,
            share,
            share_id,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "SharingTable(name={}, schema={}, share={}, share_id={}, id={})",
            self.name,
            self.schema,
            self.share,
            self.share_id.as_ref().unwrap_or(&"None".to_owned()),
            self.id.as_ref().unwrap_or(&"None".to_owned())
        )
    }
}

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
