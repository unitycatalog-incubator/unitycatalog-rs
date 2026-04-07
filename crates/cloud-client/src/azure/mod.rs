use std::sync::Arc;

use futures::future::BoxFuture;

use crate::service::make_service;
use crate::token::TemporaryToken;
use crate::{ClientOptions, CredentialProvider, RequestSigner, Result, RetryConfig, TokenProvider};

mod builder;
pub(crate) mod credential;
mod sas;

pub(crate) use self::credential::*;
pub use builder::*;
pub use credential::AzureCredential;
pub use sas::{generate_storage_key_sas, generate_user_delegation_sas};

pub type AzureCredentialProvider = Arc<dyn CredentialProvider<Credential = AzureCredential>>;

/// Configuration for Azure authentication
#[derive(Debug, Clone)]
pub struct AzureConfig {
    pub credentials: AzureCredentialProvider,
    pub retry_config: RetryConfig,
    pub skip_signature: bool,
    pub client_options: ClientOptions,
}

impl AzureConfig {
    pub(crate) async fn get_credential(&self) -> Result<Option<Arc<AzureCredential>>> {
        if self.skip_signature {
            Ok(None)
        } else {
            Some(self.credentials.get_credential().await).transpose()
        }
    }
}

impl RequestSigner for AzureConfig {
    fn sign<'a>(
        &'a self,
        req: reqwest::RequestBuilder,
    ) -> BoxFuture<'a, Result<reqwest::RequestBuilder>> {
        Box::pin(async move {
            let credential = self.get_credential().await?;
            Ok(req.with_azure_authorization(&credential))
        })
    }
}

/// Fetch a short-lived Azure storage bearer token using an OAuth2 client-secret flow.
///
/// Calls the Azure AD token endpoint for the given tenant and returns a
/// `TemporaryToken<Arc<AzureCredential>>` scoped to Azure Storage
/// (`https://storage.azure.com/.default`).
///
/// This is used by the Unity Catalog server to vend temporary Azure credentials
/// for external locations backed by an `AzureServicePrincipal` credential.
pub async fn fetch_client_secret_token(
    tenant_id: &str,
    client_id: String,
    client_secret: String,
    authority_host: Option<String>,
) -> Result<TemporaryToken<Arc<AzureCredential>>> {
    let provider = ClientSecretOAuthProvider::new_with_scope(
        client_id,
        client_secret,
        tenant_id,
        authority_host,
        credential::AZURE_STORAGE_SCOPE,
    );
    let client = ClientOptions::default().client()?;
    let service = make_service(client.clone(), None);
    provider
        .fetch_token(&client, &service, &RetryConfig::default())
        .await
}

/// Fetch a short-lived Azure storage bearer token using workload identity (federated token file).
///
/// Reads the OIDC token from the given file path and exchanges it for an Azure AD bearer
/// token via the client-credentials grant with a JWT client assertion.
pub async fn fetch_workload_identity_token(
    tenant_id: &str,
    client_id: String,
    federated_token_file: String,
    authority_host: Option<String>,
) -> Result<TemporaryToken<Arc<AzureCredential>>> {
    let provider = WorkloadIdentityOAuthProvider::new(
        client_id,
        federated_token_file,
        tenant_id,
        authority_host,
    );
    let client = ClientOptions::default().client()?;
    let service = make_service(client.clone(), None);
    provider
        .fetch_token(&client, &service, &RetryConfig::default())
        .await
}

/// Fetch a short-lived Azure storage bearer token using IMDS managed identity.
///
/// Calls the Azure Instance Metadata Service (IMDS) endpoint and returns a
/// bearer token scoped to Azure Storage.  Supports user-assigned identities via
/// `client_id`, `object_id`, or `msi_res_id`.
pub async fn fetch_managed_identity_token(
    client_id: Option<String>,
    object_id: Option<String>,
    msi_res_id: Option<String>,
) -> Result<TemporaryToken<Arc<AzureCredential>>> {
    let provider = ImdsManagedIdentityProvider::new(client_id, object_id, msi_res_id, None);
    let client = ClientOptions::default().with_allow_http(true).client()?;
    let service = make_service(client.clone(), None);
    provider
        .fetch_token(&client, &service, &RetryConfig::default())
        .await
}
