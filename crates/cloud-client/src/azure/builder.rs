// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use std::str::FromStr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;

use super::AzureConfig;
use crate::azure::credential::{
    AzureCliCredential, ClientSecretOAuthProvider, ImdsManagedIdentityProvider,
    WorkloadIdentityOAuthProvider,
};
use crate::azure::{AzureCredential, AzureCredentialProvider};
use crate::config::ConfigValue;
use crate::service::make_service;
use crate::{
    ClientConfigKey, ClientOptions, Result, RetryConfig, StaticCredentialProvider,
    TokenCredentialProvider,
};

const MSI_ENDPOINT_ENV_KEY: &str = "IDENTITY_ENDPOINT";

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("OAuth scope must be set when using client secret authentication")]
    MissingScope,

    #[error("Configuration key: '{}' is not known.", key)]
    UnknownConfigurationKey { key: String },
}

impl From<Error> for crate::Error {
    fn from(source: Error) -> Self {
        match source {
            Error::UnknownConfigurationKey { key } => Self::UnknownConfigurationKey { key },
            Error::MissingScope => Self::Generic {
                source: Box::new(source),
            },
        }
    }
}

/// Configure Azure authentication credentials.
///
/// # Example
/// ```
/// # let ACCOUNT = "foo";
/// # let ACCESS_KEY = "foo";
/// # use cloud_client::azure::AzureBuilder;
/// let config = AzureBuilder::new()
///  .with_account(ACCOUNT)
///  .with_access_key(ACCESS_KEY)
///  .build(None);
/// ```
#[derive(Default, Clone)]
pub struct AzureBuilder {
    account_name: Option<String>,
    access_key: Option<String>,
    bearer_token: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    tenant_id: Option<String>,
    scope: Option<String>,
    authority_host: Option<String>,
    endpoint: Option<String>,
    msi_endpoint: Option<String>,
    object_id: Option<String>,
    msi_resource_id: Option<String>,
    federated_token_file: Option<String>,
    use_azure_cli: ConfigValue<bool>,
    retry_config: RetryConfig,
    client_options: ClientOptions,
    credentials: Option<AzureCredentialProvider>,
    skip_signature: ConfigValue<bool>,
}

/// Configuration keys for [`AzureBuilder`]
///
/// Configuration via keys can be done via [`AzureBuilder::with_config`]
///
/// # Example
/// ```
/// # use cloud_client::azure::{AzureBuilder, AzureConfigKey};
/// let builder = AzureBuilder::new()
///     .with_config("azure_client_id".parse().unwrap(), "my-client-id")
///     .with_config(AzureConfigKey::AuthorityId, "my-tenant-id");
/// ```
#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy, Deserialize, Serialize)]
#[non_exhaustive]
pub enum AzureConfigKey {
    /// The name of the azure storage account
    ///
    /// Supported keys:
    /// - `azure_storage_account_name`
    /// - `account_name`
    AccountName,

    /// Master key for accessing storage account
    ///
    /// Supported keys:
    /// - `azure_storage_account_key`
    /// - `azure_storage_access_key`
    /// - `azure_storage_master_key`
    /// - `access_key`
    /// - `account_key`
    /// - `master_key`
    AccessKey,

    /// Service principal client id for authorizing requests
    ///
    /// Supported keys:
    /// - `azure_storage_client_id`
    /// - `azure_client_id`
    /// - `client_id`
    ClientId,

    /// Service principal client secret for authorizing requests
    ///
    /// Supported keys:
    /// - `azure_storage_client_secret`
    /// - `azure_client_secret`
    /// - `client_secret`
    ClientSecret,

    /// Tenant id used in oauth flows
    ///
    /// Supported keys:
    /// - `azure_storage_tenant_id`
    /// - `azure_storage_authority_id`
    /// - `azure_tenant_id`
    /// - `azure_authority_id`
    /// - `tenant_id`
    /// - `authority_id`
    AuthorityId,

    /// Authority host used in oauth flows
    ///
    /// Supported keys:
    /// - `azure_storage_authority_host`
    /// - `azure_authority_host`
    /// - `authority_host`
    AuthorityHost,

    /// OAuth scope for client credentials flow
    ///
    /// Supported keys:
    /// - `azure_scope`
    /// - `scope`
    Scope,

    /// Bearer token
    ///
    /// Supported keys:
    /// - `azure_storage_token`
    /// - `bearer_token`
    /// - `token`
    Token,

    /// Override the endpoint used to communicate with Azure
    ///
    /// Supported keys:
    /// - `azure_storage_endpoint`
    /// - `azure_endpoint`
    /// - `endpoint`
    Endpoint,

    /// Endpoint to request a imds managed identity token
    ///
    /// Supported keys:
    /// - `azure_msi_endpoint`
    /// - `azure_identity_endpoint`
    /// - `identity_endpoint`
    /// - `msi_endpoint`
    MsiEndpoint,

    /// Object id for use with managed identity authentication
    ///
    /// Supported keys:
    /// - `azure_object_id`
    /// - `object_id`
    ObjectId,

    /// Msi resource id for use with managed identity authentication
    ///
    /// Supported keys:
    /// - `azure_msi_resource_id`
    /// - `msi_resource_id`
    MsiResourceId,

    /// File containing token for Azure AD workload identity federation
    ///
    /// Supported keys:
    /// - `azure_federated_token_file`
    /// - `federated_token_file`
    FederatedTokenFile,

    /// Use azure cli for acquiring access token
    ///
    /// Supported keys:
    /// - `azure_use_azure_cli`
    /// - `use_azure_cli`
    UseAzureCli,

    /// Skip signing requests
    ///
    /// Supported keys:
    /// - `azure_skip_signature`
    /// - `skip_signature`
    SkipSignature,

    /// Client options
    Client(ClientConfigKey),
}

impl AsRef<str> for AzureConfigKey {
    fn as_ref(&self) -> &str {
        match self {
            Self::AccountName => "azure_storage_account_name",
            Self::AccessKey => "azure_storage_account_key",
            Self::ClientId => "azure_storage_client_id",
            Self::ClientSecret => "azure_storage_client_secret",
            Self::AuthorityId => "azure_storage_tenant_id",
            Self::AuthorityHost => "azure_storage_authority_host",
            Self::Scope => "azure_scope",
            Self::Token => "azure_storage_token",
            Self::Endpoint => "azure_storage_endpoint",
            Self::MsiEndpoint => "azure_msi_endpoint",
            Self::ObjectId => "azure_object_id",
            Self::MsiResourceId => "azure_msi_resource_id",
            Self::FederatedTokenFile => "azure_federated_token_file",
            Self::UseAzureCli => "azure_use_azure_cli",
            Self::SkipSignature => "azure_skip_signature",
            Self::Client(key) => key.as_ref(),
        }
    }
}

impl FromStr for AzureConfigKey {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "azure_storage_account_key"
            | "azure_storage_access_key"
            | "azure_storage_master_key"
            | "master_key"
            | "account_key"
            | "access_key" => Ok(Self::AccessKey),
            "azure_storage_account_name" | "account_name" => Ok(Self::AccountName),
            "azure_storage_client_id" | "azure_client_id" | "client_id" => Ok(Self::ClientId),
            "azure_storage_client_secret" | "azure_client_secret" | "client_secret" => {
                Ok(Self::ClientSecret)
            }
            "azure_storage_tenant_id"
            | "azure_storage_authority_id"
            | "azure_tenant_id"
            | "azure_authority_id"
            | "tenant_id"
            | "authority_id" => Ok(Self::AuthorityId),
            "azure_storage_authority_host" | "azure_authority_host" | "authority_host" => {
                Ok(Self::AuthorityHost)
            }
            "azure_scope" | "scope" => Ok(Self::Scope),
            "azure_storage_token" | "bearer_token" | "token" => Ok(Self::Token),
            "azure_storage_endpoint" | "azure_endpoint" | "endpoint" => Ok(Self::Endpoint),
            "azure_msi_endpoint"
            | "azure_identity_endpoint"
            | "identity_endpoint"
            | "msi_endpoint" => Ok(Self::MsiEndpoint),
            "azure_object_id" | "object_id" => Ok(Self::ObjectId),
            "azure_msi_resource_id" | "msi_resource_id" => Ok(Self::MsiResourceId),
            "azure_federated_token_file" | "federated_token_file" => Ok(Self::FederatedTokenFile),
            "azure_use_azure_cli" | "use_azure_cli" => Ok(Self::UseAzureCli),
            "azure_skip_signature" | "skip_signature" => Ok(Self::SkipSignature),
            "azure_allow_http" => Ok(Self::Client(ClientConfigKey::AllowHttp)),
            _ => match s.strip_prefix("azure_").unwrap_or(s).parse() {
                Ok(key) => Ok(Self::Client(key)),
                Err(_) => Err(Error::UnknownConfigurationKey { key: s.into() }.into()),
            },
        }
    }
}

impl std::fmt::Debug for AzureBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AzureBuilder {{ account: {:?} }}", self.account_name)
    }
}

impl AzureBuilder {
    /// Create a new [`AzureBuilder`] with default values.
    pub fn new() -> Self {
        Default::default()
    }

    /// Create an instance of [`AzureBuilder`] with values pre-populated from environment variables.
    ///
    /// Variables extracted from environment:
    /// * AZURE_STORAGE_ACCOUNT_NAME: storage account name
    /// * AZURE_STORAGE_ACCOUNT_KEY: storage account master key
    /// * AZURE_STORAGE_ACCESS_KEY: alias for AZURE_STORAGE_ACCOUNT_KEY
    /// * AZURE_STORAGE_CLIENT_ID -> client id for service principal authorization
    /// * AZURE_STORAGE_CLIENT_SECRET -> client secret for service principal authorization
    /// * AZURE_STORAGE_TENANT_ID -> tenant id used in oauth flows
    pub fn from_env() -> Self {
        let mut builder = Self::default();
        for (os_key, os_value) in std::env::vars_os() {
            if let (Some(key), Some(value)) = (os_key.to_str(), os_value.to_str()) {
                if key.starts_with("AZURE_") {
                    if let Ok(config_key) = key.to_ascii_lowercase().parse() {
                        builder = builder.with_config(config_key, value);
                    }
                }
            }
        }

        if let Ok(text) = std::env::var(MSI_ENDPOINT_ENV_KEY) {
            builder = builder.with_msi_endpoint(text);
        }

        builder
    }

    /// Set an option on the builder via a key - value pair.
    pub fn with_config(mut self, key: AzureConfigKey, value: impl Into<String>) -> Self {
        match key {
            AzureConfigKey::AccessKey => self.access_key = Some(value.into()),
            AzureConfigKey::AccountName => self.account_name = Some(value.into()),
            AzureConfigKey::ClientId => self.client_id = Some(value.into()),
            AzureConfigKey::ClientSecret => self.client_secret = Some(value.into()),
            AzureConfigKey::AuthorityId => self.tenant_id = Some(value.into()),
            AzureConfigKey::AuthorityHost => self.authority_host = Some(value.into()),
            AzureConfigKey::Scope => self.scope = Some(value.into()),
            AzureConfigKey::Token => self.bearer_token = Some(value.into()),
            AzureConfigKey::MsiEndpoint => self.msi_endpoint = Some(value.into()),
            AzureConfigKey::ObjectId => self.object_id = Some(value.into()),
            AzureConfigKey::MsiResourceId => self.msi_resource_id = Some(value.into()),
            AzureConfigKey::FederatedTokenFile => self.federated_token_file = Some(value.into()),
            AzureConfigKey::UseAzureCli => self.use_azure_cli.parse(value),
            AzureConfigKey::SkipSignature => self.skip_signature.parse(value),
            AzureConfigKey::Endpoint => self.endpoint = Some(value.into()),
            AzureConfigKey::Client(key) => {
                self.client_options = self.client_options.with_config(key, value)
            }
        };
        self
    }

    /// Get config value via a [`AzureConfigKey`].
    pub fn get_config_value(&self, key: &AzureConfigKey) -> Option<String> {
        match key {
            AzureConfigKey::AccountName => self.account_name.clone(),
            AzureConfigKey::AccessKey => self.access_key.clone(),
            AzureConfigKey::ClientId => self.client_id.clone(),
            AzureConfigKey::ClientSecret => self.client_secret.clone(),
            AzureConfigKey::AuthorityId => self.tenant_id.clone(),
            AzureConfigKey::AuthorityHost => self.authority_host.clone(),
            AzureConfigKey::Scope => self.scope.clone(),
            AzureConfigKey::Token => self.bearer_token.clone(),
            AzureConfigKey::Endpoint => self.endpoint.clone(),
            AzureConfigKey::MsiEndpoint => self.msi_endpoint.clone(),
            AzureConfigKey::ObjectId => self.object_id.clone(),
            AzureConfigKey::MsiResourceId => self.msi_resource_id.clone(),
            AzureConfigKey::FederatedTokenFile => self.federated_token_file.clone(),
            AzureConfigKey::UseAzureCli => Some(self.use_azure_cli.to_string()),
            AzureConfigKey::SkipSignature => Some(self.skip_signature.to_string()),
            AzureConfigKey::Client(key) => self.client_options.get_config_value(key),
        }
    }

    /// Set the Azure Account
    pub fn with_account(mut self, account: impl Into<String>) -> Self {
        self.account_name = Some(account.into());
        self
    }

    /// Set the Azure Access Key
    pub fn with_access_key(mut self, access_key: impl Into<String>) -> Self {
        self.access_key = Some(access_key.into());
        self
    }

    /// Set a static bearer token to be used for authorizing requests
    pub fn with_bearer_token_authorization(mut self, bearer_token: impl Into<String>) -> Self {
        self.bearer_token = Some(bearer_token.into());
        self
    }

    /// Set a client secret used for client secret authorization
    pub fn with_client_secret_authorization(
        mut self,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        tenant_id: impl Into<String>,
    ) -> Self {
        self.client_id = Some(client_id.into());
        self.client_secret = Some(client_secret.into());
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// Sets the client id for use in client secret or k8s federated credential flow
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    /// Sets the client secret for use in client secret flow
    pub fn with_client_secret(mut self, client_secret: impl Into<String>) -> Self {
        self.client_secret = Some(client_secret.into());
        self
    }

    /// Sets the tenant id for use in client secret or k8s federated credential flow
    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// Set the OAuth scope for client credentials flow
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// Set the credential provider overriding any other options
    pub fn with_credentials(mut self, credentials: AzureCredentialProvider) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Override the endpoint used to communicate with Azure
    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = Some(endpoint);
        self
    }

    /// Sets what protocol is allowed
    pub fn with_allow_http(mut self, allow_http: bool) -> Self {
        self.client_options = self.client_options.with_allow_http(allow_http);
        self
    }

    /// Sets an alternative authority host for OAuth based authorization
    ///
    /// Defaults to <https://login.microsoftonline.com>
    pub fn with_authority_host(mut self, authority_host: impl Into<String>) -> Self {
        self.authority_host = Some(authority_host.into());
        self
    }

    /// Set the retry configuration
    pub fn with_retry(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }

    /// Set the proxy_url to be used by the underlying client
    pub fn with_proxy_url(mut self, proxy_url: impl Into<String>) -> Self {
        self.client_options = self.client_options.with_proxy_url(proxy_url);
        self
    }

    /// Set a trusted proxy CA certificate
    pub fn with_proxy_ca_certificate(mut self, proxy_ca_certificate: impl Into<String>) -> Self {
        self.client_options = self
            .client_options
            .with_proxy_ca_certificate(proxy_ca_certificate);
        self
    }

    /// Set a list of hosts to exclude from proxy connections
    pub fn with_proxy_excludes(mut self, proxy_excludes: impl Into<String>) -> Self {
        self.client_options = self.client_options.with_proxy_excludes(proxy_excludes);
        self
    }

    /// Sets the client options, overriding any already set
    pub fn with_client_options(mut self, options: ClientOptions) -> Self {
        self.client_options = options;
        self
    }

    /// Sets the endpoint for acquiring managed identity token
    pub fn with_msi_endpoint(mut self, msi_endpoint: impl Into<String>) -> Self {
        self.msi_endpoint = Some(msi_endpoint.into());
        self
    }

    /// Sets a file path for acquiring azure federated identity token in k8s
    pub fn with_federated_token_file(mut self, federated_token_file: impl Into<String>) -> Self {
        self.federated_token_file = Some(federated_token_file.into());
        self
    }

    /// Set if the Azure Cli should be used for acquiring access token
    pub fn with_use_azure_cli(mut self, use_azure_cli: bool) -> Self {
        self.use_azure_cli = use_azure_cli.into();
        self
    }

    /// If enabled, requests will not be signed
    pub fn with_skip_signature(mut self, skip_signature: bool) -> Self {
        self.skip_signature = skip_signature.into();
        self
    }

    /// Build an [`AzureConfig`] from the provided values, consuming `self`.
    /// Build an [`AzureConfig`] from the provided values, consuming `self`.
    ///
    /// If `runtime` is provided, all HTTP I/O (including credential refresh)
    /// will be spawned on the given runtime handle.
    pub fn build(self, runtime: Option<&Handle>) -> Result<AzureConfig> {
        let static_creds = |credential: AzureCredential| -> AzureCredentialProvider {
            Arc::new(StaticCredentialProvider::new(credential))
        };

        let auth = if let Some(credential) = self.credentials {
            credential
        } else if let Some(bearer_token) = self.bearer_token {
            static_creds(AzureCredential::BearerToken(bearer_token))
        } else if let (Some(client_id), Some(tenant_id), Some(federated_token_file)) =
            (&self.client_id, &self.tenant_id, self.federated_token_file)
        {
            let client_credential = WorkloadIdentityOAuthProvider::new(
                client_id,
                federated_token_file,
                tenant_id,
                self.authority_host,
            );
            let client = self.client_options.client()?;
            let service = make_service(client.clone(), runtime);
            Arc::new(TokenCredentialProvider::new(
                client_credential,
                client,
                service,
                self.retry_config.clone(),
            )) as _
        } else if let (Some(client_id), Some(client_secret), Some(tenant_id)) =
            (&self.client_id, self.client_secret, &self.tenant_id)
        {
            let scope = self.scope.ok_or(Error::MissingScope)?;
            let client_credential = ClientSecretOAuthProvider::new_with_scope(
                client_id.clone(),
                client_secret,
                tenant_id,
                self.authority_host,
                &scope,
            );
            let client = self.client_options.client()?;
            let service = make_service(client.clone(), runtime);
            Arc::new(TokenCredentialProvider::new(
                client_credential,
                client,
                service,
                self.retry_config.clone(),
            )) as _
        } else if self.use_azure_cli.get()? {
            Arc::new(AzureCliCredential::new()) as _
        } else {
            let msi_credential = ImdsManagedIdentityProvider::new(
                self.client_id,
                self.object_id,
                self.msi_resource_id,
                self.msi_endpoint,
            );
            let client = self.client_options.metadata_client()?;
            let service = make_service(client.clone(), runtime);
            Arc::new(TokenCredentialProvider::new(
                msi_credential,
                client,
                service,
                self.retry_config.clone(),
            )) as _
        };

        Ok(AzureConfig {
            skip_signature: self.skip_signature.get()?,
            retry_config: self.retry_config,
            client_options: self.client_options,
            credentials: auth,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn azure_test_config_from_map() {
        let azure_client_id = "object_store:fake_access_key_id";
        let azure_storage_account_name = "object_store:fake_secret_key";
        let azure_storage_token = "object_store:fake_default_region";
        let options = HashMap::from([
            ("azure_client_id", azure_client_id),
            ("azure_storage_account_name", azure_storage_account_name),
            ("azure_storage_token", azure_storage_token),
        ]);

        let builder = options
            .into_iter()
            .fold(AzureBuilder::new(), |builder, (key, value)| {
                builder.with_config(key.parse().unwrap(), value)
            });
        assert_eq!(builder.client_id.unwrap(), azure_client_id);
        assert_eq!(builder.account_name.unwrap(), azure_storage_account_name);
        assert_eq!(builder.bearer_token.unwrap(), azure_storage_token);
    }

    #[test]
    fn azure_test_client_opts() {
        let key = "AZURE_PROXY_URL";
        if let Ok(config_key) = key.to_ascii_lowercase().parse() {
            assert_eq!(
                AzureConfigKey::Client(ClientConfigKey::ProxyUrl),
                config_key
            );
        } else {
            panic!("{key} not propagated as ClientConfigKey");
        }
    }
}
