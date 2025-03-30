use pyo3::prelude::*;

use crate::models::credentials::v1::{
    AzureManagedIdentity, AzureServicePrincipal, AzureStorageKey,
    azure_managed_identity::Identifier, azure_service_principal::Credential as SpCredential,
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
