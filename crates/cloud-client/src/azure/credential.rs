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

use std::fmt::Debug;
use std::ops::Deref;
use std::process::Command;
use std::str;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use async_trait::async_trait;
use chrono::DateTime;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderValue};
use reqwest::{Client, Method, Request, RequestBuilder};
use serde::Deserialize;

use crate::retry::RetryExt;
use crate::service::HttpService;
use crate::token::{TemporaryToken, TokenCache};
use crate::{CredentialProvider, RetryConfig, TokenProvider};

const CONTENT_TYPE_JSON: &str = "application/json";
const MSI_SECRET_ENV_KEY: &str = "IDENTITY_HEADER";
const MSI_API_VERSION: &str = "2019-08-01";

/// OIDC scope used when interacting with Azure Storage OAuth2 APIs
pub(crate) const AZURE_STORAGE_SCOPE: &str = "https://storage.azure.com/.default";

/// Resource ID used when obtaining an access token for Azure Storage from the metadata endpoint
pub(crate) const AZURE_STORAGE_RESOURCE: &str = "https://storage.azure.com";

/// Azure AD Application ID for Databricks; used as resource ID for Azure Databricks API tokens
pub(crate) const AZURE_DATABRICKS_RESOURCE: &str = "2ff814a6-3304-4ab8-85cb-cd0e6f879c1d";

/// OIDC scope for Azure Databricks OAuth2 APIs
// Used by the Azure SP two-token flow (Phase 2.5)
#[allow(dead_code)]
pub(crate) const AZURE_DATABRICKS_SCOPE: &str = "2ff814a6-3304-4ab8-85cb-cd0e6f879c1d/.default";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error performing token request: {}", source)]
    TokenRequest { source: crate::retry::Error },

    #[error("Error getting token response body: {}", source)]
    TokenResponseBody { source: reqwest::Error },

    #[error("Error reading federated token file ")]
    FederatedTokenFile,

    #[error("Invalid Access Key: {}", source)]
    InvalidAccessKey { source: base64::DecodeError },

    #[error("'az account get-access-token' command failed: {message}")]
    AzureCli { message: String },

    #[error("Failed to parse azure cli response: {source}")]
    AzureCliResponse { source: serde_json::Error },
}

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        Self::Generic {
            source: Box::new(value),
        }
    }
}

/// An Azure storage credential
#[derive(Debug, Eq, PartialEq)]
pub enum AzureCredential {
    /// An authorization token
    BearerToken(String),
}

/// A list of known Azure authority hosts
#[allow(dead_code)]
pub mod authority_hosts {
    /// China-based Azure Authority Host
    pub const AZURE_CHINA: &str = "https://login.chinacloudapi.cn";
    /// Germany-based Azure Authority Host
    pub const AZURE_GERMANY: &str = "https://login.microsoftonline.de";
    /// US Government Azure Authority Host
    pub const AZURE_GOVERNMENT: &str = "https://login.microsoftonline.us";
    /// Public Cloud Azure Authority Host
    pub const AZURE_PUBLIC_CLOUD: &str = "https://login.microsoftonline.com";
}

/// Authorize a [`Request`] with an [`AzureAuthorizer`]
#[derive(Debug)]
pub struct AzureAuthorizer<'a> {
    credential: &'a AzureCredential,
}

impl<'a> AzureAuthorizer<'a> {
    /// Create a new [`AzureAuthorizer`]
    pub fn new(credential: &'a AzureCredential) -> Self {
        AzureAuthorizer { credential }
    }

    /// Authorize `request`
    pub fn authorize(&self, request: &mut Request) {
        match self.credential {
            AzureCredential::BearerToken(token) => {
                request.headers_mut().append(
                    AUTHORIZATION,
                    HeaderValue::from_str(format!("Bearer {token}").as_str()).unwrap(),
                );
            }
        }
    }
}

pub trait AzureCredentialExt {
    /// Apply authorization to requests against azure storage accounts
    fn with_azure_authorization(
        self,
        credential: &Option<impl Deref<Target = AzureCredential>>,
    ) -> Self;
}

impl AzureCredentialExt for RequestBuilder {
    fn with_azure_authorization(
        self,
        credential: &Option<impl Deref<Target = AzureCredential>>,
    ) -> Self {
        match credential.as_deref() {
            Some(credential) => {
                let (client, request) = self.build_split();
                let mut request = request.expect("request valid");
                AzureAuthorizer::new(credential).authorize(&mut request);
                Self::from_parts(client, request)
            }
            None => self,
        }
    }
}

/// <https://learn.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-client-creds-grant-flow#successful-response-1>
#[derive(Deserialize, Debug)]
struct OAuthTokenResponse {
    access_token: String,
    expires_in: u64,
}

/// Encapsulates the logic to perform an OAuth token challenge using a client
/// secret (shared-secret variant of the OAuth 2.0 client-credentials flow).
///
/// POSTs a `client_credentials` grant to the Azure AD token endpoint and
/// returns a bearer token for the configured scope (defaults to
/// `https://storage.azure.com/.default`).
///
/// # References
/// - <https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-client-creds-grant-flow>
/// - <https://learn.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-client-creds-grant-flow#first-case-access-token-request-with-a-shared-secret>
#[derive(Debug)]
pub(crate) struct ClientSecretOAuthProvider {
    token_url: String,
    client_id: String,
    client_secret: String,
    scope: String,
}

impl ClientSecretOAuthProvider {
    /// Create a new provider with a custom OAuth scope (e.g. for Databricks).
    pub(crate) fn new_with_scope(
        client_id: String,
        client_secret: String,
        tenant_id: impl AsRef<str>,
        authority_host: Option<String>,
        scope: &str,
    ) -> Self {
        let authority_host =
            authority_host.unwrap_or_else(|| authority_hosts::AZURE_PUBLIC_CLOUD.to_owned());

        Self {
            token_url: format!(
                "{}/{}/oauth2/v2.0/token",
                authority_host,
                tenant_id.as_ref()
            ),
            client_id,
            client_secret,
            scope: scope.to_owned(),
        }
    }
}

#[async_trait::async_trait]
impl TokenProvider for ClientSecretOAuthProvider {
    type Credential = AzureCredential;

    async fn fetch_token(
        &self,
        client: &Client,
        service: &Arc<dyn HttpService>,
        retry: &RetryConfig,
    ) -> crate::Result<TemporaryToken<Arc<AzureCredential>>> {
        let response: OAuthTokenResponse = client
            .request(Method::POST, &self.token_url)
            .header(ACCEPT, HeaderValue::from_static(CONTENT_TYPE_JSON))
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("scope", self.scope.as_str()),
                ("grant_type", "client_credentials"),
            ])
            .retryable(retry, service.clone())
            .idempotent(true)
            .send()
            .await
            .map_err(|source| Error::TokenRequest { source })?
            .json()
            .await
            .map_err(|source| Error::TokenResponseBody { source })?;

        Ok(TemporaryToken {
            token: Arc::new(AzureCredential::BearerToken(response.access_token)),
            expiry: Some(Instant::now() + Duration::from_secs(response.expires_in)),
        })
    }
}

fn expires_on_string<'de, D>(deserializer: D) -> std::result::Result<Instant, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let v = String::deserialize(deserializer)?;
    let v = v.parse::<u64>().map_err(serde::de::Error::custom)?;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(serde::de::Error::custom)?;

    Ok(Instant::now() + Duration::from_secs(v.saturating_sub(now.as_secs())))
}

#[derive(Debug, Clone, Deserialize)]
struct ImdsTokenResponse {
    pub access_token: String,
    #[serde(deserialize_with = "expires_on_string")]
    pub expires_on: Instant,
}

/// Attempts authentication using a managed identity that has been assigned to the deployment environment.
///
/// Calls the Azure Instance Metadata Service (IMDS) endpoint at
/// `http://169.254.169.254/metadata/identity/oauth2/token` (or a custom
/// endpoint when one is supplied) to obtain a bearer token for the configured
/// Azure AD resource.  Supports user-assigned identities via `client_id`,
/// `object_id`, or `msi_res_id` selectors.
///
/// # References
/// - <https://learn.microsoft.com/en-us/azure/active-directory/managed-identities-azure-resources/how-to-use-vm-token>
/// - <https://learn.microsoft.com/en-gb/azure/active-directory/managed-identities-azure-resources/how-to-use-vm-token#get-a-token-using-http>
#[derive(Debug)]
pub(crate) struct ImdsManagedIdentityProvider {
    msi_endpoint: String,
    client_id: Option<String>,
    object_id: Option<String>,
    msi_res_id: Option<String>,
    resource: String,
}

impl ImdsManagedIdentityProvider {
    pub(crate) fn new(
        client_id: Option<String>,
        object_id: Option<String>,
        msi_res_id: Option<String>,
        msi_endpoint: Option<String>,
    ) -> Self {
        Self::new_with_resource(
            client_id,
            object_id,
            msi_res_id,
            msi_endpoint,
            AZURE_STORAGE_RESOURCE,
        )
    }

    /// Create a new provider with a custom Azure AD resource (e.g. for Databricks).
    pub(crate) fn new_with_resource(
        client_id: Option<String>,
        object_id: Option<String>,
        msi_res_id: Option<String>,
        msi_endpoint: Option<String>,
        resource: &str,
    ) -> Self {
        let msi_endpoint = msi_endpoint
            .unwrap_or_else(|| "http://169.254.169.254/metadata/identity/oauth2/token".to_owned());

        Self {
            msi_endpoint,
            client_id,
            object_id,
            msi_res_id,
            resource: resource.to_owned(),
        }
    }
}

#[async_trait::async_trait]
impl TokenProvider for ImdsManagedIdentityProvider {
    type Credential = AzureCredential;

    async fn fetch_token(
        &self,
        client: &Client,
        service: &Arc<dyn HttpService>,
        retry: &RetryConfig,
    ) -> crate::Result<TemporaryToken<Arc<AzureCredential>>> {
        let mut query_items = vec![
            ("api-version", MSI_API_VERSION),
            ("resource", self.resource.as_str()),
        ];

        let mut identity = None;
        if let Some(client_id) = &self.client_id {
            identity = Some(("client_id", client_id));
        }
        if let Some(object_id) = &self.object_id {
            identity = Some(("object_id", object_id));
        }
        if let Some(msi_res_id) = &self.msi_res_id {
            identity = Some(("msi_res_id", msi_res_id));
        }
        if let Some((key, value)) = identity {
            query_items.push((key, value));
        }

        let mut builder = client
            .request(Method::GET, &self.msi_endpoint)
            .header("metadata", "true")
            .query(&query_items);

        if let Ok(val) = std::env::var(MSI_SECRET_ENV_KEY) {
            builder = builder.header("x-identity-header", val);
        };

        let response: ImdsTokenResponse = builder
            .send_retry(retry, service.clone())
            .await
            .map_err(|source| Error::TokenRequest { source })?
            .json()
            .await
            .map_err(|source| Error::TokenResponseBody { source })?;

        Ok(TemporaryToken {
            token: Arc::new(AzureCredential::BearerToken(response.access_token)),
            expiry: Some(response.expires_on),
        })
    }
}

/// Credential for using workload identity federation.
///
/// Exchanges a federated OIDC token (read from a file, e.g. a Kubernetes
/// projected service-account token) for an Azure AD bearer token using the
/// `client_credentials` grant with a JWT client assertion.  Typically used in
/// AKS workloads where the pod is annotated with a managed identity.
///
/// # References
/// - <https://learn.microsoft.com/en-us/entra/workload-id/workload-identity-federation>
/// - <https://learn.microsoft.com/en-us/azure/active-directory/develop/workload-identity-federation>
#[derive(Debug)]
pub(crate) struct WorkloadIdentityOAuthProvider {
    token_url: String,
    client_id: String,
    federated_token_file: String,
}

impl WorkloadIdentityOAuthProvider {
    pub(crate) fn new(
        client_id: impl Into<String>,
        federated_token_file: impl Into<String>,
        tenant_id: impl AsRef<str>,
        authority_host: Option<String>,
    ) -> Self {
        let authority_host =
            authority_host.unwrap_or_else(|| authority_hosts::AZURE_PUBLIC_CLOUD.to_owned());

        Self {
            token_url: format!(
                "{}/{}/oauth2/v2.0/token",
                authority_host,
                tenant_id.as_ref()
            ),
            client_id: client_id.into(),
            federated_token_file: federated_token_file.into(),
        }
    }
}

#[async_trait::async_trait]
impl TokenProvider for WorkloadIdentityOAuthProvider {
    type Credential = AzureCredential;

    async fn fetch_token(
        &self,
        client: &Client,
        service: &Arc<dyn HttpService>,
        retry: &RetryConfig,
    ) -> crate::Result<TemporaryToken<Arc<AzureCredential>>> {
        let token_str = std::fs::read_to_string(&self.federated_token_file)
            .map_err(|_| Error::FederatedTokenFile)?;

        let response: OAuthTokenResponse = client
            .request(Method::POST, &self.token_url)
            .header(ACCEPT, HeaderValue::from_static(CONTENT_TYPE_JSON))
            .form(&[
                ("client_id", self.client_id.as_str()),
                (
                    "client_assertion_type",
                    "urn:ietf:params:oauth:client-assertion-type:jwt-bearer",
                ),
                ("client_assertion", token_str.as_str()),
                ("scope", AZURE_STORAGE_SCOPE),
                ("grant_type", "client_credentials"),
            ])
            .retryable(retry, service.clone())
            .idempotent(true)
            .send()
            .await
            .map_err(|source| Error::TokenRequest { source })?
            .json()
            .await
            .map_err(|source| Error::TokenResponseBody { source })?;

        Ok(TemporaryToken {
            token: Arc::new(AzureCredential::BearerToken(response.access_token)),
            expiry: Some(Instant::now() + Duration::from_secs(response.expires_in)),
        })
    }
}

mod az_cli_date_format {
    use chrono::{DateTime, TimeZone};
    use serde::{self, Deserialize, Deserializer};

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<chrono::Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let date = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S.%6f")
            .map_err(serde::de::Error::custom)?;
        chrono::Local
            .from_local_datetime(&date)
            .single()
            .ok_or(serde::de::Error::custom(
                "azure cli returned ambiguous expiry date",
            ))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AzureCliTokenResponse {
    pub access_token: String,
    #[serde(with = "az_cli_date_format")]
    pub expires_on: DateTime<chrono::Local>,
    pub token_type: String,
}

#[derive(Default, Debug)]
pub(crate) struct AzureCliCredential {
    cache: TokenCache<Arc<AzureCredential>>,
}

impl AzureCliCredential {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    async fn fetch_token(&self) -> Result<TemporaryToken<Arc<AzureCredential>>> {
        let program = if cfg!(target_os = "windows") {
            "cmd"
        } else {
            "az"
        };
        let mut args = Vec::new();
        if cfg!(target_os = "windows") {
            args.push("/C");
            args.push("az");
        }
        args.push("account");
        args.push("get-access-token");
        args.push("--output");
        args.push("json");
        args.push("--scope");
        args.push(AZURE_STORAGE_SCOPE);

        match Command::new(program).args(args).output() {
            Ok(az_output) if az_output.status.success() => {
                let output = str::from_utf8(&az_output.stdout).map_err(|_| Error::AzureCli {
                    message: "az response is not a valid utf-8 string".to_string(),
                })?;

                let token_response = serde_json::from_str::<AzureCliTokenResponse>(output)
                    .map_err(|source| Error::AzureCliResponse { source })?;

                if !token_response.token_type.eq_ignore_ascii_case("bearer") {
                    return Err(Error::AzureCli {
                        message: format!(
                            "got unexpected token type from azure cli: {0}",
                            token_response.token_type
                        ),
                    });
                }
                let duration =
                    token_response.expires_on.naive_local() - chrono::Local::now().naive_local();
                Ok(TemporaryToken {
                    token: Arc::new(AzureCredential::BearerToken(token_response.access_token)),
                    expiry: Some(
                        Instant::now()
                            + duration.to_std().map_err(|_| Error::AzureCli {
                                message: "az returned invalid lifetime".to_string(),
                            })?,
                    ),
                })
            }
            Ok(az_output) => {
                let message = String::from_utf8_lossy(&az_output.stderr);
                Err(Error::AzureCli {
                    message: message.into(),
                })
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Err(Error::AzureCli {
                    message: "Azure Cli not installed".into(),
                }),
                error_kind => Err(Error::AzureCli {
                    message: format!("io error: {error_kind:?}"),
                }),
            },
        }
    }
}

#[async_trait]
impl CredentialProvider for AzureCliCredential {
    type Credential = AzureCredential;

    async fn get_credential(&self) -> crate::Result<Arc<Self::Credential>> {
        Ok(self.cache.get_or_insert_with(|| self.fetch_token()).await?)
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Client;
    use tempfile::NamedTempFile;

    use super::*;
    use crate::service::ReqwestService;
    use mockito;

    #[tokio::test]
    async fn test_managed_identity() {
        let mut server = mockito::Server::new_async().await;

        unsafe { std::env::set_var(MSI_SECRET_ENV_KEY, "env-secret") };

        let endpoint = server.url();
        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry_config = RetryConfig::default();

        let _mock = server
            .mock("GET", "/metadata/identity/oauth2/token")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "client_id".into(),
                "client_id".into(),
            )]))
            .match_header("x-identity-header", "env-secret")
            .match_header("metadata", "true")
            .with_status(200)
            .with_body(
                r#"
            {
                "access_token": "TOKEN",
                "refresh_token": "",
                "expires_in": "3599",
                "expires_on": "1506484173",
                "not_before": "1506480273",
                "resource": "https://management.azure.com/",
                "token_type": "Bearer"
              }
            "#,
            )
            .create_async()
            .await;

        let credential = ImdsManagedIdentityProvider::new(
            Some("client_id".into()),
            None,
            None,
            Some(format!("{endpoint}/metadata/identity/oauth2/token")),
        );

        let token = credential
            .fetch_token(&client, &service, &retry_config)
            .await
            .unwrap();

        assert_eq!(
            token.token.as_ref(),
            &AzureCredential::BearerToken("TOKEN".into())
        );
    }

    #[tokio::test]
    async fn test_workload_identity() {
        let mut server = mockito::Server::new_async().await;
        let tokenfile = NamedTempFile::new().unwrap();
        let tenant = "tenant";
        std::fs::write(tokenfile.path(), "federated-token").unwrap();

        let endpoint = server.url();
        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry_config = RetryConfig::default();

        let _mock = server
            .mock("POST", format!("/{tenant}/oauth2/v2.0/token").as_str())
            .match_body(mockito::Matcher::Regex("federated-token".into()))
            .with_status(200)
            .with_body(
                r#"
            {
                "access_token": "TOKEN",
                "refresh_token": "",
                "expires_in": 3599,
                "expires_on": "1506484173",
                "not_before": "1506480273",
                "resource": "https://management.azure.com/",
                "token_type": "Bearer"
              }
            "#,
            )
            .create_async()
            .await;

        let credential = WorkloadIdentityOAuthProvider::new(
            "client_id",
            tokenfile.path().to_str().unwrap(),
            tenant,
            Some(endpoint.to_string()),
        );

        let token = credential
            .fetch_token(&client, &service, &retry_config)
            .await
            .unwrap();

        assert_eq!(
            token.token.as_ref(),
            &AzureCredential::BearerToken("TOKEN".into())
        );
    }

    #[tokio::test]
    async fn test_client_secret_happy_path() {
        let mut server = mockito::Server::new_async().await;
        let tenant = "my-tenant";

        let _mock = server
            .mock("POST", format!("/{tenant}/oauth2/v2.0/token").as_str())
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::Regex("client_id=myclientid".into()),
                mockito::Matcher::Regex("grant_type=client_credentials".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"access_token":"AZURE_TOKEN","expires_in":3599,"token_type":"Bearer"}"#)
            .create_async()
            .await;

        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry_config = RetryConfig::default();

        let provider = ClientSecretOAuthProvider::new_with_scope(
            "myclientid".into(),
            "myclientsecret".into(),
            tenant,
            Some(server.url()),
            AZURE_STORAGE_SCOPE,
        );

        let token = provider
            .fetch_token(&client, &service, &retry_config)
            .await
            .unwrap();

        assert_eq!(
            token.token.as_ref(),
            &AzureCredential::BearerToken("AZURE_TOKEN".into())
        );
        assert!(token.expiry.is_some());

        _mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_client_secret_invalid_client() {
        let mut server = mockito::Server::new_async().await;
        let tenant = "bad-tenant";

        let _mock = server
            .mock("POST", format!("/{tenant}/oauth2/v2.0/token").as_str())
            .with_status(400)
            .with_body(
                r#"{"error":"invalid_client","error_description":"Client authentication failed"}"#,
            )
            .create_async()
            .await;

        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry_config = RetryConfig::default();

        let provider = ClientSecretOAuthProvider::new_with_scope(
            "badclientid".into(),
            "badsecret".into(),
            tenant,
            Some(server.url()),
            AZURE_STORAGE_SCOPE,
        );

        let result = provider.fetch_token(&client, &service, &retry_config).await;

        assert!(result.is_err(), "Expected error for 400 response");

        _mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_client_secret_token_refresh() {
        let mut server = mockito::Server::new_async().await;
        let tenant = "refresh-tenant";

        // First call returns a token with very short expiry (1 second)
        let _mock1 = server
            .mock("POST", format!("/{tenant}/oauth2/v2.0/token").as_str())
            .with_status(200)
            .with_body(r#"{"access_token":"FIRST_TOKEN","expires_in":1,"token_type":"Bearer"}"#)
            .create_async()
            .await;

        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry_config = RetryConfig::default();

        let provider = ClientSecretOAuthProvider::new_with_scope(
            "myclientid".into(),
            "myclientsecret".into(),
            tenant,
            Some(server.url()),
            AZURE_STORAGE_SCOPE,
        );

        let token1 = provider
            .fetch_token(&client, &service, &retry_config)
            .await
            .unwrap();
        assert_eq!(
            token1.token.as_ref(),
            &AzureCredential::BearerToken("FIRST_TOKEN".into())
        );
        // Token expiry should be very soon (1 second)
        assert!(token1.expiry.is_some());

        // Second call returns a new token
        let _mock2 = server
            .mock("POST", format!("/{tenant}/oauth2/v2.0/token").as_str())
            .with_status(200)
            .with_body(r#"{"access_token":"SECOND_TOKEN","expires_in":3600,"token_type":"Bearer"}"#)
            .create_async()
            .await;

        let token2 = provider
            .fetch_token(&client, &service, &retry_config)
            .await
            .unwrap();
        assert_eq!(
            token2.token.as_ref(),
            &AzureCredential::BearerToken("SECOND_TOKEN".into())
        );

        _mock1.assert_async().await;
        _mock2.assert_async().await;
    }
}
