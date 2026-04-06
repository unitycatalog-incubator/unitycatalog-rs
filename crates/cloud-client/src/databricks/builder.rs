use std::str::FromStr;
use std::sync::Arc;

use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;

use super::cfg_file::load_cfg_profile;
use super::credential::{
    DatabricksCredential, DatabricksGcpTokenExchangeProvider, DatabricksM2MProvider,
    OidcEnvTokenProvider, OidcFileTokenProvider,
};
use crate::azure::credential::{AZURE_DATABRICKS_RESOURCE, ImdsManagedIdentityProvider};
// AZURE_DATABRICKS_SCOPE is used by the Azure SP two-token flow (Phase 2.5)
#[allow(unused_imports)]
use crate::azure::credential::AZURE_DATABRICKS_SCOPE;
use crate::service::make_service;
use crate::{
    ClientConfigKey, ClientOptions, CredentialProvider, RequestSigner, Result, RetryConfig,
    StaticCredentialProvider, TokenCredentialProvider,
};

/// Default OAuth scope for Databricks Unity Catalog (least privilege).
const DEFAULT_DATABRICKS_SCOPE: &str = "unity-catalog";

/// Default env var name holding an OIDC token.
const DEFAULT_OIDC_TOKEN_ENV: &str = "DATABRICKS_OIDC_TOKEN";

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

    /// Path to a non-default `.databrickscfg` file.
    ///
    /// Supported keys: `databricks_config_file`, `config_file`
    ConfigFile,

    /// Profile name within `.databrickscfg`.
    ///
    /// Supported keys: `databricks_config_profile`, `config_profile`, `profile`
    ConfigProfile,

    /// Force a specific auth type (e.g. `pat`, `oauth-m2m`, `env-oidc`, `file-oidc`).
    ///
    /// Supported keys: `databricks_auth_type`, `auth_type`
    AuthType,

    /// Name of the environment variable holding an OIDC token.
    /// Defaults to `DATABRICKS_OIDC_TOKEN`.
    ///
    /// Supported keys: `databricks_oidc_token_env`, `oidc_token_env`
    OidcTokenEnv,

    /// File path to an OIDC token.
    ///
    /// Supported keys: `databricks_oidc_token_filepath`, `oidc_token_filepath`
    OidcTokenFilepath,

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
            Self::ConfigFile => "databricks_config_file",
            Self::ConfigProfile => "databricks_config_profile",
            Self::AuthType => "databricks_auth_type",
            Self::OidcTokenEnv => "databricks_oidc_token_env",
            Self::OidcTokenFilepath => "databricks_oidc_token_filepath",
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
            "databricks_config_file" | "config_file" => Ok(Self::ConfigFile),
            "databricks_config_profile" | "config_profile" | "profile" => Ok(Self::ConfigProfile),
            "databricks_auth_type" | "auth_type" => Ok(Self::AuthType),
            "databricks_oidc_token_env" | "oidc_token_env" => Ok(Self::OidcTokenEnv),
            "databricks_oidc_token_filepath" | "oidc_token_filepath" => Ok(Self::OidcTokenFilepath),
            _ => match s.strip_prefix("databricks_").unwrap_or(s).parse() {
                Ok(key) => Ok(Self::Client(key)),
                Err(_) => Err(crate::Error::UnknownConfigurationKey { key: s.into() }),
            },
        }
    }
}

/// Builder for Databricks authentication configuration.
///
/// Auth resolution order (when no `auth_type` is forced):
/// 1. Explicit `with_credentials(provider)` override.
/// 2. PAT / static token (`DATABRICKS_TOKEN` / `with_token()`).
/// 3. OAuth M2M (`DATABRICKS_CLIENT_ID` + `DATABRICKS_CLIENT_SECRET`).
/// 4. `env-oidc` — OIDC token from env var (`DATABRICKS_OIDC_TOKEN_ENV`, default `DATABRICKS_OIDC_TOKEN`).
/// 5. `file-oidc` — OIDC token from file (`DATABRICKS_OIDC_TOKEN_FILEPATH`).
/// 6. Azure MSI fallback (if `DATABRICKS_AZURE_RESOURCE_ID` is set).
/// 7. GCP service account token exchange (if `GOOGLE_APPLICATION_CREDENTIALS` is set).
///
/// If `DATABRICKS_AUTH_TYPE` is set (or `with_auth_type()` called), only the named
/// auth type is attempted; an error is returned immediately if its required fields are absent.
///
/// `.databrickscfg` profile values are loaded as the lowest-priority source — env vars
/// and code-level setters override them.
#[derive(Default, Clone)]
pub struct DatabricksBuilder {
    host: Option<String>,
    token: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    account_id: Option<String>,
    azure_resource_id: Option<String>,
    scope: Option<String>,
    config_file: Option<String>,
    config_profile: Option<String>,
    auth_type: Option<String>,
    oidc_token_env: Option<String>,
    oidc_token_filepath: Option<String>,
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

    /// Populate from environment variables and optionally a `.databrickscfg` profile.
    ///
    /// Profile values from `.databrickscfg` (or `DATABRICKS_CONFIG_FILE`) are loaded first
    /// as the lowest-priority source. Environment variables override profile values.
    ///
    /// Variables read: `DATABRICKS_HOST`, `DATABRICKS_TOKEN`, `DATABRICKS_CLIENT_ID`,
    /// `DATABRICKS_CLIENT_SECRET`, `DATABRICKS_ACCOUNT_ID`, `DATABRICKS_AZURE_RESOURCE_ID`,
    /// `DATABRICKS_OAUTH_SCOPE`, `DATABRICKS_CONFIG_FILE`, `DATABRICKS_CONFIG_PROFILE`,
    /// `DATABRICKS_AUTH_TYPE`, `DATABRICKS_OIDC_TOKEN_ENV`, `DATABRICKS_OIDC_TOKEN_FILEPATH`,
    /// `GOOGLE_APPLICATION_CREDENTIALS`.
    pub fn from_env() -> Self {
        let mut builder = Self::default();

        // First pass: collect DATABRICKS_* env vars into a map so we can read
        // CONFIG_FILE / CONFIG_PROFILE before applying other values.
        let mut env_map: Vec<(DatabricksConfigKey, String)> = Vec::new();
        let mut config_file_env: Option<String> = None;
        let mut config_profile_env: Option<String> = None;

        for (os_key, os_value) in std::env::vars_os() {
            if let (Some(key), Some(value)) = (os_key.to_str(), os_value.to_str()) {
                if key.starts_with("DATABRICKS_") || key == "GOOGLE_APPLICATION_CREDENTIALS" {
                    if let Ok(config_key) = key.to_ascii_lowercase().parse::<DatabricksConfigKey>()
                    {
                        match config_key {
                            DatabricksConfigKey::ConfigFile => {
                                config_file_env = Some(value.to_owned())
                            }
                            DatabricksConfigKey::ConfigProfile => {
                                config_profile_env = Some(value.to_owned())
                            }
                            _ => env_map.push((config_key, value.to_owned())),
                        }
                    }
                }
            }
        }

        // Apply profile values first (lowest priority).
        if let Ok(profile_values) =
            load_cfg_profile(config_file_env.as_deref(), config_profile_env.as_deref())
        {
            for (k, v) in profile_values {
                builder = builder.with_config(k, v);
            }
        }

        // Apply env vars (override profile).
        for (key, value) in env_map {
            builder = builder.with_config(key, value);
        }

        // Restore config_file / config_profile from env (they were extracted above).
        if let Some(v) = config_file_env {
            builder.config_file = Some(v);
        }
        if let Some(v) = config_profile_env {
            builder.config_profile = Some(v);
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
            DatabricksConfigKey::ConfigFile => self.config_file = Some(value.into()),
            DatabricksConfigKey::ConfigProfile => self.config_profile = Some(value.into()),
            DatabricksConfigKey::AuthType => self.auth_type = Some(value.into()),
            DatabricksConfigKey::OidcTokenEnv => self.oidc_token_env = Some(value.into()),
            DatabricksConfigKey::OidcTokenFilepath => self.oidc_token_filepath = Some(value.into()),
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

    /// Force a specific auth type (e.g. `"pat"`, `"oauth-m2m"`, `"env-oidc"`, `"file-oidc"`).
    pub fn with_auth_type(mut self, auth_type: impl Into<String>) -> Self {
        self.auth_type = Some(auth_type.into());
        self
    }

    /// Set the env var name that holds an OIDC token (default: `DATABRICKS_OIDC_TOKEN`).
    pub fn with_oidc_token_env(mut self, env_var: impl Into<String>) -> Self {
        self.oidc_token_env = Some(env_var.into());
        self
    }

    /// Set the file path to an OIDC token.
    pub fn with_oidc_token_filepath(mut self, path: impl Into<String>) -> Self {
        self.oidc_token_filepath = Some(path.into());
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

        // Determine which auth types are candidates based on available fields.
        let has_pat = self.token.is_some();
        let has_m2m = self.client_id.is_some() && self.client_secret.is_some();
        let has_env_oidc =
            self.oidc_token_env.is_some() || std::env::var(DEFAULT_OIDC_TOKEN_ENV).is_ok();
        let has_file_oidc = self.oidc_token_filepath.is_some();
        let has_azure_msi = self.azure_resource_id.is_some();
        let has_gcp = std::env::var("GOOGLE_APPLICATION_CREDENTIALS").is_ok();

        // Extract fields we need after auth resolution.
        let retry_config = self.retry_config.clone();
        let client_options = self.client_options.clone();

        let credentials: DatabricksCredentialProvider = if let Some(creds) = self.credentials {
            // 1. Explicit override — always wins regardless of auth_type.
            creds
        } else if let Some(auth_type) = self.auth_type.clone() {
            // Forced auth type — only attempt the named type.
            self.build_forced_auth(
                auth_type,
                scope,
                runtime,
                has_pat,
                has_m2m,
                has_env_oidc,
                has_file_oidc,
                has_azure_msi,
                has_gcp,
            )?
        } else if has_pat {
            // 2. PAT / static token
            Arc::new(StaticCredentialProvider::new(DatabricksCredential {
                bearer: self.token.unwrap(),
            }))
        } else if has_m2m {
            // 3. OAuth M2M (AWS/GCP Databricks-hosted)
            let host = self.host.as_deref().unwrap_or("").trim_end_matches('/');
            let token_url = format!("{host}/oidc/v1/token");
            let provider = DatabricksM2MProvider {
                token_url,
                client_id: self.client_id.unwrap(),
                client_secret: self.client_secret.unwrap(),
                scope,
            };
            let client = client_options.client()?;
            let service = make_service(client.clone(), runtime);
            Arc::new(TokenCredentialProvider::new(
                provider,
                client,
                service,
                retry_config.clone(),
            )) as _
        } else if has_env_oidc {
            // 4. env-oidc
            let env_var = self
                .oidc_token_env
                .unwrap_or_else(|| DEFAULT_OIDC_TOKEN_ENV.to_owned());
            let provider = OidcEnvTokenProvider { env_var };
            let client = client_options.client()?;
            let service = make_service(client.clone(), runtime);
            Arc::new(TokenCredentialProvider::new(
                provider,
                client,
                service,
                retry_config.clone(),
            )) as _
        } else if has_file_oidc {
            // 5. file-oidc
            let provider = OidcFileTokenProvider {
                filepath: self.oidc_token_filepath.unwrap(),
            };
            let client = client_options.client()?;
            let service = make_service(client.clone(), runtime);
            Arc::new(TokenCredentialProvider::new(
                provider,
                client,
                service,
                retry_config.clone(),
            )) as _
        } else if has_azure_msi {
            // 6. Azure MSI fallback
            let msi_provider = ImdsManagedIdentityProvider::new_with_resource(
                None,
                None,
                None,
                None,
                AZURE_DATABRICKS_RESOURCE,
            );
            let client = client_options.metadata_client()?;
            let service = make_service(client.clone(), runtime);
            let azure_provider =
                TokenCredentialProvider::new(msi_provider, client, service, retry_config.clone());
            Arc::new(AzureToDatabricksBridge {
                inner: Arc::new(azure_provider),
            }) as _
        } else if has_gcp {
            // 7. GCP SA token exchange
            let host = self.host.as_deref().unwrap_or("").trim_end_matches('/');
            let token_url = format!("{host}/oidc/v1/token");
            let gcp_config = crate::gcp::GoogleBuilder::new().build(runtime)?;
            let provider = DatabricksGcpTokenExchangeProvider {
                token_url,
                gcp_provider: gcp_config.credentials,
            };
            let client = client_options.client()?;
            let service = make_service(client.clone(), runtime);
            Arc::new(TokenCredentialProvider::new(
                provider,
                client,
                service,
                retry_config.clone(),
            )) as _
        } else {
            return Err(crate::Error::Generic {
                source: "No Databricks credentials configured. Set DATABRICKS_TOKEN, \
                         DATABRICKS_CLIENT_ID + DATABRICKS_CLIENT_SECRET, \
                         DATABRICKS_OIDC_TOKEN (env-oidc), DATABRICKS_OIDC_TOKEN_FILEPATH (file-oidc), \
                         DATABRICKS_AZURE_RESOURCE_ID, or GOOGLE_APPLICATION_CREDENTIALS."
                    .into(),
            });
        };

        Ok(DatabricksConfig {
            credentials,
            retry_config,
            client_options,
        })
    }

    /// Build credentials when `auth_type` forces a specific method.
    #[allow(clippy::too_many_arguments)]
    fn build_forced_auth(
        self,
        auth_type: String,
        scope: String,
        runtime: Option<&Handle>,
        has_pat: bool,
        has_m2m: bool,
        has_env_oidc: bool,
        has_file_oidc: bool,
        has_azure_msi: bool,
        has_gcp: bool,
    ) -> Result<DatabricksCredentialProvider> {
        match auth_type.as_str() {
            "pat" => {
                if !has_pat {
                    return Err(crate::Error::Generic {
                        source: "auth_type=pat requires DATABRICKS_TOKEN to be set".into(),
                    });
                }
                Ok(Arc::new(StaticCredentialProvider::new(DatabricksCredential {
                    bearer: self.token.unwrap(),
                })))
            }
            "oauth-m2m" => {
                if !has_m2m {
                    return Err(crate::Error::Generic {
                        source: "auth_type=oauth-m2m requires DATABRICKS_CLIENT_ID and \
                                 DATABRICKS_CLIENT_SECRET to be set"
                            .into(),
                    });
                }
                let host = self.host.as_deref().unwrap_or("").trim_end_matches('/');
                let token_url = format!("{host}/oidc/v1/token");
                let provider = DatabricksM2MProvider {
                    token_url,
                    client_id: self.client_id.unwrap(),
                    client_secret: self.client_secret.unwrap(),
                    scope,
                };
                let client = self.client_options.client()?;
                let service = make_service(client.clone(), runtime);
                Ok(Arc::new(TokenCredentialProvider::new(
                    provider,
                    client,
                    service,
                    self.retry_config,
                )) as _)
            }
            "env-oidc" => {
                if !has_env_oidc {
                    return Err(crate::Error::Generic {
                        source: "auth_type=env-oidc requires DATABRICKS_OIDC_TOKEN or \
                                 DATABRICKS_OIDC_TOKEN_ENV to be set"
                            .into(),
                    });
                }
                let env_var = self
                    .oidc_token_env
                    .unwrap_or_else(|| DEFAULT_OIDC_TOKEN_ENV.to_owned());
                let provider = OidcEnvTokenProvider { env_var };
                let client = self.client_options.client()?;
                let service = make_service(client.clone(), runtime);
                Ok(Arc::new(TokenCredentialProvider::new(
                    provider,
                    client,
                    service,
                    self.retry_config,
                )) as _)
            }
            "file-oidc" => {
                if !has_file_oidc {
                    return Err(crate::Error::Generic {
                        source: "auth_type=file-oidc requires DATABRICKS_OIDC_TOKEN_FILEPATH \
                                 to be set"
                            .into(),
                    });
                }
                let provider = OidcFileTokenProvider {
                    filepath: self.oidc_token_filepath.unwrap(),
                };
                let client = self.client_options.client()?;
                let service = make_service(client.clone(), runtime);
                Ok(Arc::new(TokenCredentialProvider::new(
                    provider,
                    client,
                    service,
                    self.retry_config,
                )) as _)
            }
            "azure-msi" => {
                if !has_azure_msi {
                    return Err(crate::Error::Generic {
                        source: "auth_type=azure-msi requires DATABRICKS_AZURE_RESOURCE_ID \
                                 to be set"
                            .into(),
                    });
                }
                let msi_provider = ImdsManagedIdentityProvider::new_with_resource(
                    None,
                    None,
                    None,
                    None,
                    AZURE_DATABRICKS_RESOURCE,
                );
                let client = self.client_options.metadata_client()?;
                let service = make_service(client.clone(), runtime);
                let azure_provider = TokenCredentialProvider::new(
                    msi_provider,
                    client,
                    service,
                    self.retry_config,
                );
                Ok(Arc::new(AzureToDatabricksBridge {
                    inner: Arc::new(azure_provider),
                }) as _)
            }
            "gcp" => {
                if !has_gcp {
                    return Err(crate::Error::Generic {
                        source: "auth_type=gcp requires GOOGLE_APPLICATION_CREDENTIALS to be set"
                            .into(),
                    });
                }
                let host = self.host.as_deref().unwrap_or("").trim_end_matches('/');
                let token_url = format!("{host}/oidc/v1/token");
                let gcp_config = crate::gcp::GoogleBuilder::new().build(runtime)?;
                let provider = DatabricksGcpTokenExchangeProvider {
                    token_url,
                    gcp_provider: gcp_config.credentials,
                };
                let client = self.client_options.client()?;
                let service = make_service(client.clone(), runtime);
                Ok(Arc::new(TokenCredentialProvider::new(
                    provider,
                    client,
                    service,
                    self.retry_config,
                )) as _)
            }
            other => Err(crate::Error::Generic {
                source: format!("Unknown auth_type: {other:?}. Valid values: pat, oauth-m2m, env-oidc, file-oidc, azure-msi, gcp").into(),
            }),
        }
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
            ("databricks_config_file", DatabricksConfigKey::ConfigFile),
            ("config_file", DatabricksConfigKey::ConfigFile),
            (
                "databricks_config_profile",
                DatabricksConfigKey::ConfigProfile,
            ),
            ("config_profile", DatabricksConfigKey::ConfigProfile),
            ("profile", DatabricksConfigKey::ConfigProfile),
            ("databricks_auth_type", DatabricksConfigKey::AuthType),
            ("auth_type", DatabricksConfigKey::AuthType),
            (
                "databricks_oidc_token_env",
                DatabricksConfigKey::OidcTokenEnv,
            ),
            ("oidc_token_env", DatabricksConfigKey::OidcTokenEnv),
            (
                "databricks_oidc_token_filepath",
                DatabricksConfigKey::OidcTokenFilepath,
            ),
            (
                "oidc_token_filepath",
                DatabricksConfigKey::OidcTokenFilepath,
            ),
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

    #[test]
    fn test_auth_type_enforcement_missing_fields() {
        let err = DatabricksBuilder::new()
            .with_auth_type("pat")
            .build(None)
            .unwrap_err();
        assert!(err.to_string().contains("pat"));
    }
}
