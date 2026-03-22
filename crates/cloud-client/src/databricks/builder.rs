use std::str::FromStr;
use std::sync::Arc;

use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;

use super::credential::{
    DatabricksCredential, DatabricksGcpTokenExchangeProvider, DatabricksM2MProvider,
};
use crate::azure::credential::{
    AZURE_DATABRICKS_RESOURCE, AZURE_DATABRICKS_SCOPE, ImdsManagedIdentityProvider,
};
use crate::service::make_service;
use crate::{
    ClientConfigKey, ClientOptions, CredentialProvider, RequestSigner, Result, RetryConfig,
    StaticCredentialProvider, TokenCredentialProvider,
};

/// Default OAuth scope for Databricks Unity Catalog (least privilege).
const DEFAULT_DATABRICKS_SCOPE: &str = "unity-catalog";

type DatabricksCredentialProvider = Arc<dyn CredentialProvider<Credential = DatabricksCredential>>;

/// Unified Databricks auth configuration (result of building a [`DatabricksBuilder`]).
#[derive(Debug, Clone)]
pub struct DatabricksConfig {
    pub credentials: DatabricksCredentialProvider,
    pub retry_config: RetryConfig,
    pub client_options: ClientOptions,
}

impl RequestSigner for DatabricksConfig {
    fn sign<'a>(
        &'a self,
        req: reqwest::RequestBuilder,
    ) -> BoxFuture<'a, Result<reqwest::RequestBuilder>> {
        Box::pin(async move {
            let cred = self.credentials.get_credential().await?;
            Ok(req.bearer_auth(&cred.bearer))
        })
    }
}

/// Configuration keys for [`DatabricksBuilder`].
#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy, Deserialize, Serialize)]
#[non_exhaustive]
pub enum DatabricksConfigKey {
    /// Databricks workspace host URL.
    ///
    /// Supported keys: `databricks_host`, `host`
    Host,

    /// Personal Access Token or static token.
    ///
    /// Supported keys: `databricks_token`, `token`
    Token,

    /// OAuth M2M service principal client ID.
    ///
    /// Supported keys: `databricks_client_id`, `client_id`
    ClientId,

    /// OAuth M2M service principal client secret.
    ///
    /// Supported keys: `databricks_client_secret`, `client_secret`
    ClientSecret,

    /// Account ID (for account-level operations).
    ///
    /// Supported keys: `databricks_account_id`, `account_id`
    AccountId,

    /// Azure resource ID; presence triggers Azure MSI fallback path.
    ///
    /// Supported keys: `databricks_azure_resource_id`, `azure_resource_id`
    AzureResourceId,

    /// OAuth scope override (defaults to `unity-catalog`).
    ///
    /// Supported keys: `databricks_oauth_scope`, `oauth_scope`
    Scope,

    /// Delegated HTTP client config keys.
    Client(ClientConfigKey),
}

impl AsRef<str> for DatabricksConfigKey {
    fn as_ref(&self) -> &str {
        match self {
            Self::Host => "databricks_host",
            Self::Token => "databricks_token",
            Self::ClientId => "databricks_client_id",
            Self::ClientSecret => "databricks_client_secret",
            Self::AccountId => "databricks_account_id",
            Self::AzureResourceId => "databricks_azure_resource_id",
            Self::Scope => "databricks_oauth_scope",
            Self::Client(key) => key.as_ref(),
        }
    }
}

impl FromStr for DatabricksConfigKey {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "databricks_host" | "host" => Ok(Self::Host),
            "databricks_token" | "token" => Ok(Self::Token),
            "databricks_client_id" | "client_id" => Ok(Self::ClientId),
            "databricks_client_secret" | "client_secret" => Ok(Self::ClientSecret),
            "databricks_account_id" | "account_id" => Ok(Self::AccountId),
            "databricks_azure_resource_id" | "azure_resource_id" => Ok(Self::AzureResourceId),
            "databricks_oauth_scope" | "oauth_scope" => Ok(Self::Scope),
            _ => match s.strip_prefix("databricks_").unwrap_or(s).parse() {
                Ok(key) => Ok(Self::Client(key)),
                Err(_) => Err(crate::Error::UnknownConfigurationKey { key: s.into() }),
            },
        }
    }
}

/// Builder for Databricks authentication configuration.
///
/// Auth resolution order:
/// 1. Explicit `with_credentials(provider)` override.
/// 2. PAT / static token (`DATABRICKS_TOKEN` / `with_token()`).
/// 3. OAuth M2M (`DATABRICKS_CLIENT_ID` + `DATABRICKS_CLIENT_SECRET`).
/// 4. Azure MSI fallback (if `DATABRICKS_AZURE_RESOURCE_ID` is set).
/// 5. GCP service account token exchange (if `GOOGLE_APPLICATION_CREDENTIALS` is set).
#[derive(Default, Clone)]
pub struct DatabricksBuilder {
    host: Option<String>,
    token: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    account_id: Option<String>,
    azure_resource_id: Option<String>,
    scope: Option<String>,
    retry_config: RetryConfig,
    client_options: ClientOptions,
    credentials: Option<DatabricksCredentialProvider>,
}

impl std::fmt::Debug for DatabricksBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DatabricksBuilder {{ host: {:?} }}", self.host)
    }
}

impl DatabricksBuilder {
    /// Create a new [`DatabricksBuilder`] with default values.
    pub fn new() -> Self {
        Default::default()
    }

    /// Populate from environment variables.
    ///
    /// Variables read: `DATABRICKS_HOST`, `DATABRICKS_TOKEN`, `DATABRICKS_CLIENT_ID`,
    /// `DATABRICKS_CLIENT_SECRET`, `DATABRICKS_ACCOUNT_ID`, `DATABRICKS_AZURE_RESOURCE_ID`,
    /// `DATABRICKS_OAUTH_SCOPE`.
    pub fn from_env() -> Self {
        let mut builder = Self::default();
        for (os_key, os_value) in std::env::vars_os() {
            if let (Some(key), Some(value)) = (os_key.to_str(), os_value.to_str()) {
                if key.starts_with("DATABRICKS_") {
                    if let Ok(config_key) = key.to_ascii_lowercase().parse() {
                        builder = builder.with_config(config_key, value);
                    }
                }
            }
        }
        builder
    }

    /// Set an option via a key-value pair.
    pub fn with_config(mut self, key: DatabricksConfigKey, value: impl Into<String>) -> Self {
        match key {
            DatabricksConfigKey::Host => self.host = Some(value.into()),
            DatabricksConfigKey::Token => self.token = Some(value.into()),
            DatabricksConfigKey::ClientId => self.client_id = Some(value.into()),
            DatabricksConfigKey::ClientSecret => self.client_secret = Some(value.into()),
            DatabricksConfigKey::AccountId => self.account_id = Some(value.into()),
            DatabricksConfigKey::AzureResourceId => self.azure_resource_id = Some(value.into()),
            DatabricksConfigKey::Scope => self.scope = Some(value.into()),
            DatabricksConfigKey::Client(key) => {
                self.client_options = self.client_options.with_config(key, value)
            }
        }
        self
    }

    /// Override credential resolution with a custom provider.
    pub fn with_credentials(mut self, credentials: DatabricksCredentialProvider) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Set the Databricks host URL.
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Set a static PAT or bearer token.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Set the retry configuration.
    pub fn with_retry(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }

    /// Set the HTTP client options.
    pub fn with_client_options(mut self, options: ClientOptions) -> Self {
        self.client_options = options;
        self
    }

    /// Build a [`DatabricksConfig`].
    ///
    /// If `runtime` is provided, HTTP I/O for credential refresh is spawned on that runtime.
    pub fn build(self, runtime: Option<&Handle>) -> Result<DatabricksConfig> {
        let scope = self
            .scope
            .as_deref()
            .unwrap_or(DEFAULT_DATABRICKS_SCOPE)
            .to_owned();

        let credentials: DatabricksCredentialProvider = if let Some(creds) = self.credentials {
            // 1. Explicit override
            creds
        } else if let Some(token) = self.token {
            // 2. PAT / static token (also covers notebook context auth and open-source UC tokens)
            Arc::new(StaticCredentialProvider::new(DatabricksCredential {
                bearer: token,
            }))
        } else if let (Some(client_id), Some(client_secret)) = (self.client_id, self.client_secret)
        {
            // 3. OAuth M2M (AWS/GCP Databricks-hosted)
            let host = self.host.as_deref().unwrap_or("").trim_end_matches('/');
            let token_url = format!("{host}/oidc/v1/token");
            let provider = DatabricksM2MProvider {
                token_url,
                client_id,
                client_secret,
                scope,
            };
            let client = self.client_options.client()?;
            let service = make_service(client.clone(), runtime);
            Arc::new(TokenCredentialProvider::new(
                provider,
                client,
                service,
                self.retry_config.clone(),
            )) as _
        } else if self.azure_resource_id.is_some() {
            // 4. Azure MSI fallback
            let msi_provider = ImdsManagedIdentityProvider::new_with_resource(
                None,
                None,
                None,
                None,
                AZURE_DATABRICKS_RESOURCE,
            );
            // Azure MSI returns an AzureCredential; we need to wrap it as a DatabricksCredential.
            let client = self.client_options.metadata_client()?;
            let service = make_service(client.clone(), runtime);
            let azure_provider = TokenCredentialProvider::new(
                msi_provider,
                client,
                service,
                self.retry_config.clone(),
            );
            // Bridge: AzureCredential -> DatabricksCredential
            Arc::new(AzureToDatabricksBridge {
                inner: Arc::new(azure_provider),
            }) as _
        } else if std::env::var("GOOGLE_APPLICATION_CREDENTIALS").is_ok() {
            // 5. GCP SA token exchange
            let host = self.host.as_deref().unwrap_or("").trim_end_matches('/');
            let token_url = format!("{host}/oidc/v1/token");
            let gcp_config = crate::gcp::GoogleBuilder::new().build(runtime)?;
            let provider = DatabricksGcpTokenExchangeProvider {
                token_url,
                gcp_provider: gcp_config.credentials,
            };
            let client = self.client_options.client()?;
            let service = make_service(client.clone(), runtime);
            Arc::new(TokenCredentialProvider::new(
                provider,
                client,
                service,
                self.retry_config.clone(),
            )) as _
        } else {
            return Err(crate::Error::Generic {
                source: "No Databricks credentials configured. Set DATABRICKS_TOKEN, \
                         DATABRICKS_CLIENT_ID + DATABRICKS_CLIENT_SECRET, \
                         DATABRICKS_AZURE_RESOURCE_ID, or GOOGLE_APPLICATION_CREDENTIALS."
                    .into(),
            });
        };

        Ok(DatabricksConfig {
            credentials,
            retry_config: self.retry_config,
            client_options: self.client_options,
        })
    }
}

/// Bridges `AzureCredential::BearerToken` → `DatabricksCredential`.
///
/// Azure MSI returns an Azure AD bearer token for the Databricks resource ID;
/// that token is used directly as the Databricks API bearer token.
#[derive(Debug)]
struct AzureToDatabricksBridge {
    inner: Arc<dyn CredentialProvider<Credential = crate::azure::credential::AzureCredential>>,
}

#[async_trait::async_trait]
impl CredentialProvider for AzureToDatabricksBridge {
    type Credential = DatabricksCredential;

    async fn get_credential(&self) -> crate::Result<Arc<DatabricksCredential>> {
        use crate::azure::credential::AzureCredential;
        let cred = self.inner.get_credential().await?;
        match cred.as_ref() {
            AzureCredential::BearerToken(token) => Ok(Arc::new(DatabricksCredential {
                bearer: token.clone(),
            })),
            AzureCredential::SASToken(_) => Err(crate::Error::Generic {
                source: "Azure MSI returned a SAS token instead of a bearer token for Databricks"
                    .into(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_databricks_config_from_map() {
        let options = HashMap::from([
            (
                "databricks_host",
                "https://my-workspace.azuredatabricks.net",
            ),
            ("databricks_token", "dapi1234567890"),
            ("databricks_oauth_scope", "all-apis"),
        ]);

        let builder = options
            .into_iter()
            .fold(DatabricksBuilder::new(), |b, (k, v)| {
                b.with_config(k.parse().unwrap(), v)
            });

        assert_eq!(
            builder.host.as_deref(),
            Some("https://my-workspace.azuredatabricks.net")
        );
        assert_eq!(builder.token.as_deref(), Some("dapi1234567890"));
        assert_eq!(builder.scope.as_deref(), Some("all-apis"));
    }

    #[test]
    fn test_databricks_config_key_roundtrip() {
        let cases = [
            ("databricks_host", DatabricksConfigKey::Host),
            ("host", DatabricksConfigKey::Host),
            ("databricks_token", DatabricksConfigKey::Token),
            ("token", DatabricksConfigKey::Token),
            ("databricks_client_id", DatabricksConfigKey::ClientId),
            ("client_id", DatabricksConfigKey::ClientId),
            (
                "databricks_client_secret",
                DatabricksConfigKey::ClientSecret,
            ),
            ("client_secret", DatabricksConfigKey::ClientSecret),
            ("databricks_account_id", DatabricksConfigKey::AccountId),
            ("account_id", DatabricksConfigKey::AccountId),
            (
                "databricks_azure_resource_id",
                DatabricksConfigKey::AzureResourceId,
            ),
            ("azure_resource_id", DatabricksConfigKey::AzureResourceId),
            ("databricks_oauth_scope", DatabricksConfigKey::Scope),
            ("oauth_scope", DatabricksConfigKey::Scope),
        ];

        for (s, expected) in cases {
            assert_eq!(
                s.parse::<DatabricksConfigKey>().unwrap(),
                expected,
                "failed for {s}"
            );
        }
    }

    #[tokio::test]
    async fn test_build_with_pat() {
        let config = DatabricksBuilder::new()
            .with_host("https://my.azuredatabricks.net")
            .with_token("dapimytoken")
            .build(None)
            .unwrap();

        let cred = config.credentials.get_credential().await.unwrap();
        assert_eq!(cred.bearer, "dapimytoken");
    }
}
