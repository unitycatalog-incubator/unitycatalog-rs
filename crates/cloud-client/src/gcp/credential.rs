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

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use futures::TryFutureExt;
use reqwest::{Client, Method};
use ring::signature::RsaKeyPair;
use serde::Deserialize;
use tracing::info;

use crate::retry::RetryExt;
use crate::service::HttpService;
use crate::token::TemporaryToken;
use crate::{RetryConfig, TokenProvider};

pub(crate) const DEFAULT_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";

const DEFAULT_METADATA_HOST: &str = "metadata.google.internal";
const DEFAULT_METADATA_IP: &str = "169.254.169.254";

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Unable to open service account file from {}: {}", path.display(), source)]
    OpenCredentials {
        source: std::io::Error,
        path: PathBuf,
    },

    #[error("Unable to decode service account file: {}", source)]
    DecodeCredentials { source: serde_json::Error },

    #[error("No RSA key found in pem file")]
    MissingKey,

    #[error("Invalid RSA key: {}", source)]
    InvalidKey {
        #[from]
        source: ring::error::KeyRejected,
    },

    #[error("Error signing: {}", source)]
    Sign { source: ring::error::Unspecified },

    #[error("Error encoding jwt payload: {}", source)]
    Encode { source: serde_json::Error },

    #[error("Error performing token request: {}", source)]
    TokenRequest { source: crate::retry::Error },

    #[error("Error getting token response body: {}", source)]
    TokenResponseBody { source: reqwest::Error },
}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        Self::Generic {
            source: Box::new(value),
        }
    }
}

/// A private RSA key for a service account
#[derive(Debug)]
pub struct ServiceAccountKey(RsaKeyPair);

impl ServiceAccountKey {
    /// Parses a pem-encoded RSA key
    pub fn from_pem(encoded: &[u8]) -> Result<Self> {
        use rustls_pemfile::Item;
        use std::io::Cursor;

        let mut cursor = Cursor::new(encoded);
        let mut reader = BufReader::new(&mut cursor);

        // Reading from string is infallible
        match rustls_pemfile::read_one(&mut reader).unwrap() {
            Some(Item::Pkcs8Key(key)) => Self::from_pkcs8(key.secret_pkcs8_der()),
            Some(Item::Pkcs1Key(key)) => Self::from_der(key.secret_pkcs1_der()),
            _ => Err(Error::MissingKey),
        }
    }

    /// Parses an unencrypted PKCS#8-encoded RSA private key.
    pub fn from_pkcs8(key: &[u8]) -> Result<Self> {
        Ok(Self(RsaKeyPair::from_pkcs8(key)?))
    }

    /// Parses an unencrypted PKCS#8-encoded RSA private key.
    pub fn from_der(key: &[u8]) -> Result<Self> {
        Ok(Self(RsaKeyPair::from_der(key)?))
    }

}

/// A Google Cloud Storage Credential
#[derive(Debug, Eq, PartialEq)]
pub struct GcpCredential {
    /// An HTTP bearer token
    pub bearer: String,
}

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Default, serde::Serialize)]
pub(crate) struct JwtHeader<'a> {
    /// The type of JWS: it can only be "JWT" here
    ///
    /// Defined in [RFC7515#4.1.9](https://tools.ietf.org/html/rfc7515#section-4.1.9).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typ: Option<&'a str>,
    /// The algorithm used
    ///
    /// Defined in [RFC7515#4.1.1](https://tools.ietf.org/html/rfc7515#section-4.1.1).
    pub alg: &'a str,
    /// Content type
    ///
    /// Defined in [RFC7519#5.2](https://tools.ietf.org/html/rfc7519#section-5.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cty: Option<&'a str>,
    /// JSON Key URL
    ///
    /// Defined in [RFC7515#4.1.2](https://tools.ietf.org/html/rfc7515#section-4.1.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jku: Option<&'a str>,
    /// Key ID
    ///
    /// Defined in [RFC7515#4.1.4](https://tools.ietf.org/html/rfc7515#section-4.1.4).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<&'a str>,
    /// X.509 URL
    ///
    /// Defined in [RFC7515#4.1.5](https://tools.ietf.org/html/rfc7515#section-4.1.5).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x5u: Option<&'a str>,
    /// X.509 certificate thumbprint
    ///
    /// Defined in [RFC7515#4.1.7](https://tools.ietf.org/html/rfc7515#section-4.1.7).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x5t: Option<&'a str>,
}

#[derive(serde::Serialize)]
struct TokenClaims<'a> {
    iss: &'a str,
    sub: &'a str,
    scope: &'a str,
    exp: u64,
    iat: u64,
}

#[derive(serde::Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

/// Self-signed JWT (JSON Web Token).
///
/// # References
/// - <https://google.aip.dev/auth/4111>
#[derive(Debug)]
pub(crate) struct SelfSignedJwt {
    issuer: String,
    scope: String,
    private_key: ServiceAccountKey,
    key_id: String,
}

impl SelfSignedJwt {
    /// Create a new [`SelfSignedJwt`]
    pub(crate) fn new(
        key_id: String,
        issuer: String,
        private_key: ServiceAccountKey,
        scope: String,
    ) -> Result<Self> {
        Ok(Self {
            issuer,
            scope,
            private_key,
            key_id,
        })
    }
}

#[async_trait]
impl TokenProvider for SelfSignedJwt {
    type Credential = GcpCredential;

    /// Fetch a fresh token
    async fn fetch_token(
        &self,
        _client: &Client,
        _service: &Arc<dyn HttpService>,
        _retry: &RetryConfig,
    ) -> crate::Result<TemporaryToken<Arc<GcpCredential>>> {
        let now = seconds_since_epoch();
        let exp = now + 3600;

        let claims = TokenClaims {
            iss: &self.issuer,
            sub: &self.issuer,
            scope: &self.scope,
            iat: now,
            exp,
        };

        let jwt_header = b64_encode_obj(&JwtHeader {
            alg: "RS256",
            typ: Some("JWT"),
            kid: Some(&self.key_id),
            ..Default::default()
        })?;

        let claim_str = b64_encode_obj(&claims)?;
        let message = [jwt_header.as_ref(), claim_str.as_ref()].join(".");
        let mut sig_bytes = vec![0; self.private_key.0.public().modulus_len()];
        self.private_key
            .0
            .sign(
                &ring::signature::RSA_PKCS1_SHA256,
                &ring::rand::SystemRandom::new(),
                message.as_bytes(),
                &mut sig_bytes,
            )
            .map_err(|source| Error::Sign { source })?;

        let signature = BASE64_URL_SAFE_NO_PAD.encode(sig_bytes);
        let bearer = [message, signature].join(".");

        Ok(TemporaryToken {
            token: Arc::new(GcpCredential { bearer }),
            expiry: Some(Instant::now() + Duration::from_secs(3600)),
        })
    }
}

fn read_credentials_file<T>(service_account_path: impl AsRef<std::path::Path>) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let file = File::open(&service_account_path).map_err(|source| {
        let path = service_account_path.as_ref().to_owned();
        Error::OpenCredentials { source, path }
    })?;
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).map_err(|source| Error::DecodeCredentials { source })
}

/// A deserialized `service-account-********.json`-file.
///
/// Holds the RSA private key and associated metadata needed to generate
/// self-signed JWTs for authenticating as a GCP service account without
/// going through the OAuth 2.0 token endpoint.  Call [`token_provider`] to
/// obtain a [`SelfSignedJwt`] that can be used directly as a
/// [`TokenProvider`].
///
/// # References
/// - <https://developers.google.com/identity/protocols/oauth2/service-account>
///
/// [`token_provider`]: ServiceAccountCredentials::token_provider
#[derive(serde::Deserialize, Debug, Clone)]
pub(crate) struct ServiceAccountCredentials {
    /// The private key in RSA format.
    pub private_key: String,

    /// The private key ID
    pub private_key_id: String,

    /// The email address associated with the service account.
    pub client_email: String,

    /// Disable oauth and use empty tokens.
    #[serde(default)]
    pub disable_oauth: bool,
}

impl ServiceAccountCredentials {
    /// Create a new [`ServiceAccountCredentials`] from a file.
    pub(crate) fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        read_credentials_file(path)
    }

    /// Create a new [`ServiceAccountCredentials`] from a string.
    pub(crate) fn from_key(key: &str) -> Result<Self> {
        serde_json::from_str(key).map_err(|source| Error::DecodeCredentials { source })
    }

    /// Create a [`SelfSignedJwt`] from this credentials struct.
    ///
    /// We use a scope of [`DEFAULT_SCOPE`] as opposed to an audience
    /// as GCS appears to not support audience
    ///
    /// # References
    /// - <https://stackoverflow.com/questions/63222450/service-account-authorization-without-oauth-can-we-get-file-from-google-cloud/71834557#71834557>
    /// - <https://www.codejam.info/2022/05/google-cloud-service-account-authorization-without-oauth.html>
    pub(crate) fn token_provider(self) -> crate::Result<SelfSignedJwt> {
        Ok(SelfSignedJwt::new(
            self.private_key_id,
            self.client_email,
            ServiceAccountKey::from_pem(self.private_key.as_bytes())?,
            DEFAULT_SCOPE.to_string(),
        )?)
    }
}

/// Returns the number of seconds since unix epoch.
///
/// Returns 0 if the system clock is set before the Unix epoch (January 1, 1970),
/// which would cause JWT tokens to have invalid `iat`/`exp` values and be rejected
/// by the token endpoint.
fn seconds_since_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn b64_encode_obj<T: serde::Serialize>(obj: &T) -> Result<String> {
    let string = serde_json::to_string(obj).map_err(|source| Error::Encode { source })?;
    Ok(BASE64_URL_SAFE_NO_PAD.encode(string))
}

/// A provider that uses the Google Cloud Platform metadata server to fetch a token.
///
/// Queries the GCE/GKE metadata server at
/// `http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token`
/// (falling back to the IP `169.254.169.254`) to obtain a short-lived OAuth 2.0
/// bearer token for the VM's attached service account.  Respects the
/// `GCE_METADATA_HOST`, `GCE_METADATA_ROOT`, and `GCE_METADATA_IP` environment
/// variables for custom endpoint overrides.
///
/// # References
/// - <https://cloud.google.com/compute/docs/access/authenticate-workloads>
/// - <https://cloud.google.com/docs/authentication/get-id-token#metadata-server>
#[derive(Debug, Default)]
pub(crate) struct InstanceCredentialProvider {}

/// Make a request to the metadata server to fetch a token, using a a given hostname.
async fn make_metadata_request(
    client: &Client,
    service: &Arc<dyn HttpService>,
    hostname: &str,
    retry: &RetryConfig,
) -> crate::Result<TokenResponse> {
    let url =
        format!("http://{hostname}/computeMetadata/v1/instance/service-accounts/default/token");
    let response: TokenResponse = client
        .request(Method::GET, url)
        .header("Metadata-Flavor", "Google")
        .query(&[("audience", "https://www.googleapis.com/oauth2/v4/token")])
        .send_retry(retry, service.clone())
        .await
        .map_err(|source| Error::TokenRequest { source })?
        .json()
        .await
        .map_err(|source| Error::TokenResponseBody { source })?;
    Ok(response)
}

#[async_trait]
impl TokenProvider for InstanceCredentialProvider {
    type Credential = GcpCredential;

    /// Fetch a token from the metadata server.
    /// Since the connection is local we need to enable http access and don't actually use the client object passed in.
    /// Respects the `GCE_METADATA_HOST`, `GCE_METADATA_ROOT`, and `GCE_METADATA_IP`
    /// environment variables.
    ///
    /// References: <https://googleapis.dev/python/google-auth/latest/reference/google.auth.environment_vars.html>
    async fn fetch_token(
        &self,
        client: &Client,
        service: &Arc<dyn HttpService>,
        retry: &RetryConfig,
    ) -> crate::Result<TemporaryToken<Arc<GcpCredential>>> {
        let metadata_host = if let Ok(host) = env::var("GCE_METADATA_HOST") {
            host
        } else if let Ok(host) = env::var("GCE_METADATA_ROOT") {
            host
        } else {
            DEFAULT_METADATA_HOST.to_string()
        };
        let metadata_ip = if let Ok(ip) = env::var("GCE_METADATA_IP") {
            ip
        } else {
            DEFAULT_METADATA_IP.to_string()
        };

        info!("fetching token from metadata server");
        let response = make_metadata_request(client, service, &metadata_host, retry)
            .or_else(|_| make_metadata_request(client, service, &metadata_ip, retry))
            .await?;

        let token = TemporaryToken {
            token: Arc::new(GcpCredential {
                bearer: response.access_token,
            }),
            expiry: Some(Instant::now() + Duration::from_secs(response.expires_in)),
        };
        Ok(token)
    }
}

/// A deserialized `application_default_credentials.json`-file.
///
/// # References
/// - <https://cloud.google.com/docs/authentication/application-default-credentials#personal>
/// - <https://google.aip.dev/auth/4110>
#[derive(serde::Deserialize, Clone)]
#[serde(tag = "type")]
pub(crate) enum ApplicationDefaultCredentials {
    /// Service Account.
    ///
    /// # References
    /// - <https://google.aip.dev/auth/4112>
    #[serde(rename = "service_account")]
    ServiceAccount(ServiceAccountCredentials),
    /// Authorized user via "gcloud CLI Integration".
    ///
    /// # References
    /// - <https://google.aip.dev/auth/4113>
    #[serde(rename = "authorized_user")]
    AuthorizedUser(AuthorizedUserCredentials),
    /// External account credentials (Workload Identity Federation).
    ///
    /// # References
    /// - <https://cloud.google.com/iam/docs/workload-identity-federation>
    #[serde(rename = "external_account")]
    ExternalAccount(ExternalAccountCredentials),
}

impl ApplicationDefaultCredentials {
    const CREDENTIALS_PATH: &'static str = if cfg!(windows) {
        "gcloud/application_default_credentials.json"
    } else {
        ".config/gcloud/application_default_credentials.json"
    };

    // Create a new application default credential in the following situations:
    //  1. a file is passed in and the type matches.
    //  2. without argument if the well-known configuration file is present.
    pub(crate) fn read(path: Option<&str>) -> Result<Option<Self>, Error> {
        if let Some(path) = path {
            return read_credentials_file::<Self>(path).map(Some);
        }

        let home_var = if cfg!(windows) { "APPDATA" } else { "HOME" };
        if let Some(home) = env::var_os(home_var) {
            let path = Path::new(&home).join(Self::CREDENTIALS_PATH);

            // It's expected for this file to not exist unless it has been explicitly configured by the user.
            if path.exists() {
                return read_credentials_file::<Self>(path).map(Some);
            }
        }
        Ok(None)
    }
}

const DEFAULT_TOKEN_GCP_URI: &str = "https://accounts.google.com/o/oauth2/token";

/// <https://google.aip.dev/auth/4113>
#[derive(Debug, Deserialize, Clone)]
pub(crate) struct AuthorizedUserCredentials {
    client_id: String,
    client_secret: String,
    refresh_token: String,
}

#[async_trait]
impl TokenProvider for AuthorizedUserCredentials {
    type Credential = GcpCredential;

    async fn fetch_token(
        &self,
        client: &Client,
        service: &Arc<dyn HttpService>,
        retry: &RetryConfig,
    ) -> crate::Result<TemporaryToken<Arc<GcpCredential>>> {
        let response = client
            .request(Method::POST, DEFAULT_TOKEN_GCP_URI)
            .form(&[
                ("grant_type", "refresh_token"),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
                ("refresh_token", &self.refresh_token),
            ])
            .retryable(retry, service.clone())
            .idempotent(true)
            .send()
            .await
            .map_err(|source| Error::TokenRequest { source })?
            .json::<TokenResponse>()
            .await
            .map_err(|source| Error::TokenResponseBody { source })?;

        Ok(TemporaryToken {
            token: Arc::new(GcpCredential {
                bearer: response.access_token,
            }),
            expiry: Some(Instant::now() + Duration::from_secs(response.expires_in)),
        })
    }
}

/// External credentials file format used by Workload Identity Federation.
///
/// This corresponds to the JSON file generated by `gcloud iam workload-identity-pools
/// create-cred-config` and pointed to by `GOOGLE_APPLICATION_CREDENTIALS`.
///
/// # References
/// - <https://cloud.google.com/iam/docs/workload-identity-federation>
/// - <https://google.aip.dev/auth/4117>
#[derive(Debug, Deserialize, Clone)]
pub(crate) struct ExternalAccountCredentials {
    /// The STS token exchange URL (e.g. `https://sts.googleapis.com/v1/token`)
    pub token_url: String,
    /// The audience for the STS exchange (workload identity pool resource name)
    pub audience: String,
    /// The requested token type after exchange (always `urn:ietf:params:oauth:token-type:access_token`)
    pub requested_token_type: String,
    /// Source that provides the external subject token
    pub credential_source: CredentialSource,
    /// Optional service account to impersonate after STS exchange
    pub service_account_impersonation_url: Option<String>,
    /// Requested scopes (defaults to DEFAULT_SCOPE)
    pub scopes: Option<String>,
}

/// Describes where to read the external credential token from.
#[derive(Debug, Deserialize, Clone)]
pub(crate) struct CredentialSource {
    /// Path to a file containing the external credential token
    pub file: Option<String>,
    /// Environment variable containing the external credential token
    pub environment_id: Option<String>,
}

/// STS token exchange response.
#[derive(Debug, Deserialize)]
struct StsTokenResponse {
    access_token: String,
    expires_in: u64,
}

/// SA impersonation response (`iamcredentials.googleapis.com`).
#[derive(Debug, Deserialize)]
struct ImpersonateTokenResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    /// RFC 3339 expiry time, e.g. `"2099-01-01T00:00:00Z"`
    #[serde(rename = "expireTime")]
    expire_time: String,
}

/// Two-step GCP Workload Identity Federation provider.
///
/// # Step 1 — STS token exchange
/// POST `https://sts.googleapis.com/v1/token` with the external subject token
/// (read from a file or env var) to obtain a federated access token.
///
/// # Step 2 — Service Account impersonation (optional)
/// If `service_account_impersonation_url` is set, POST to
/// `https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/<SA>:generateAccessToken`
/// using the federated token to obtain a short-lived SA token.
///
/// # References
/// - <https://cloud.google.com/iam/docs/workload-identity-federation>
/// - <https://cloud.google.com/iam/docs/reference/sts/rest/v1/TopLevel/token>
/// - <https://cloud.google.com/iam/docs/reference/credentials/rest/v1/projects.serviceAccounts/generateAccessToken>
#[derive(Debug)]
pub(crate) struct GcpWorkloadIdentityProvider {
    pub credentials: ExternalAccountCredentials,
    pub sts_endpoint: String,
}

#[async_trait]
impl TokenProvider for GcpWorkloadIdentityProvider {
    type Credential = GcpCredential;

    async fn fetch_token(
        &self,
        client: &Client,
        service: &Arc<dyn HttpService>,
        retry: &RetryConfig,
    ) -> crate::Result<TemporaryToken<Arc<GcpCredential>>> {
        // Read external subject token from file or env var
        let subject_token = if let Some(file) = &self.credentials.credential_source.file {
            std::fs::read_to_string(file).map_err(|e| crate::Error::Generic {
                source: Box::new(e),
            })?
        } else if let Some(env_id) = &self.credentials.credential_source.environment_id {
            std::env::var(env_id).map_err(|e| crate::Error::Generic {
                source: Box::new(e),
            })?
        } else {
            return Err(crate::Error::Generic {
                source: "No credential source (file or environment_id) configured".into(),
            });
        };

        let scope = self
            .credentials
            .scopes
            .as_deref()
            .unwrap_or(DEFAULT_SCOPE);

        // Step 1: STS token exchange
        let sts_resp: StsTokenResponse = client
            .request(Method::POST, &self.sts_endpoint)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:token-exchange"),
                ("audience", &self.credentials.audience),
                ("scope", scope),
                ("requested_token_type", &self.credentials.requested_token_type),
                ("subject_token", &subject_token),
                ("subject_token_type", "urn:ietf:params:oauth:token-type:jwt"),
            ])
            .retryable(retry, service.clone())
            .idempotent(true)
            .send()
            .await
            .map_err(|source| Error::TokenRequest { source })?
            .json()
            .await
            .map_err(|source| Error::TokenResponseBody { source })?;

        // Step 2: Optional SA impersonation
        if let Some(imp_url) = &self.credentials.service_account_impersonation_url {
            let imp_resp: ImpersonateTokenResponse = client
                .request(Method::POST, imp_url)
                .bearer_auth(&sts_resp.access_token)
                .json(&serde_json::json!({
                    "scope": [scope],
                    "lifetime": "3600s"
                }))
                .retryable(retry, service.clone())
                .idempotent(true)
                .send()
                .await
                .map_err(|source| Error::TokenRequest { source })?
                .json()
                .await
                .map_err(|source| Error::TokenResponseBody { source })?;

            // Parse RFC 3339 expiry; fall back to 1 hour
            let expiry = chrono::DateTime::parse_from_rfc3339(&imp_resp.expire_time)
                .ok()
                .map(|dt| {
                    let secs = dt.timestamp().saturating_sub(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64,
                    );
                    std::time::Instant::now()
                        + Duration::from_secs(secs.max(0) as u64)
                })
                .unwrap_or_else(|| std::time::Instant::now() + Duration::from_secs(3600));

            Ok(TemporaryToken {
                token: Arc::new(GcpCredential {
                    bearer: imp_resp.access_token,
                }),
                expiry: Some(expiry),
            })
        } else {
            Ok(TemporaryToken {
                token: Arc::new(GcpCredential {
                    bearer: sts_resp.access_token,
                }),
                expiry: Some(
                    std::time::Instant::now() + Duration::from_secs(sts_resp.expires_in),
                ),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write as _;

    use reqwest::Client;

    use super::*;
    use crate::service::ReqwestService;

    #[tokio::test]
    async fn test_metadata_server_token() {
        let mut server = mockito::Server::new_async().await;
        let host_port = server.host_with_port();

        // GCE_METADATA_HOST accepts `host:port` (no scheme).
        // Also set GCE_METADATA_IP so the fallback also resolves to our mock server.
        // SAFETY: single-threaded test environment; no concurrent env mutation.
        unsafe {
            std::env::set_var("GCE_METADATA_HOST", &host_port);
            std::env::set_var("GCE_METADATA_IP", &host_port);
        };

        let _mock = server
            .mock("GET", "/computeMetadata/v1/instance/service-accounts/default/token")
            .match_header("Metadata-Flavor", "Google")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_body(r#"{"access_token":"GCP_META_TOKEN","expires_in":3600,"token_type":"Bearer"}"#)
            .create_async()
            .await;

        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry = RetryConfig::default();

        let provider = InstanceCredentialProvider::default();
        let token = provider
            .fetch_token(&client, &service, &retry)
            .await
            .unwrap();

        assert_eq!(token.token.bearer, "GCP_META_TOKEN");
        assert!(token.expiry.is_some());

        unsafe {
            std::env::remove_var("GCE_METADATA_HOST");
            std::env::remove_var("GCE_METADATA_IP");
        };
        _mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_workload_identity_sts_only() {
        let mut server = mockito::Server::new_async().await;

        // Write external subject token to a temp file
        let mut token_file = tempfile::NamedTempFile::new().unwrap();
        write!(token_file, "external-subject-jwt").unwrap();

        let sts_url = format!("{}/v1/token", server.url());

        let _sts_mock = server
            .mock("POST", "/v1/token")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::Regex("grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Atoken-exchange".into()),
                mockito::Matcher::Regex("subject_token=external-subject-jwt".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"access_token":"FEDERATED_TOKEN","expires_in":3600,"token_type":"Bearer"}"#)
            .create_async()
            .await;

        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry = RetryConfig::default();

        let credentials = ExternalAccountCredentials {
            token_url: sts_url.clone(),
            audience: "//iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/my-pool/providers/my-provider".into(),
            requested_token_type: "urn:ietf:params:oauth:token-type:access_token".into(),
            credential_source: CredentialSource {
                file: Some(token_file.path().to_str().unwrap().to_owned()),
                environment_id: None,
            },
            service_account_impersonation_url: None,
            scopes: None,
        };

        let provider = GcpWorkloadIdentityProvider {
            credentials,
            sts_endpoint: sts_url,
        };

        let token = provider
            .fetch_token(&client, &service, &retry)
            .await
            .unwrap();

        assert_eq!(token.token.bearer, "FEDERATED_TOKEN");
        assert!(token.expiry.is_some());

        _sts_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_workload_identity_with_impersonation() {
        let mut server = mockito::Server::new_async().await;

        // Write external subject token to a temp file
        let mut token_file = tempfile::NamedTempFile::new().unwrap();
        write!(token_file, "external-subject-jwt").unwrap();

        let sts_url = format!("{}/v1/token", server.url());
        let imp_url = format!("{}/v1/impersonate", server.url());

        let _sts_mock = server
            .mock("POST", "/v1/token")
            .with_status(200)
            .with_body(r#"{"access_token":"FEDERATED_TOKEN","expires_in":3600,"token_type":"Bearer"}"#)
            .create_async()
            .await;

        let _imp_mock = server
            .mock("POST", "/v1/impersonate")
            .match_header("Authorization", "Bearer FEDERATED_TOKEN")
            .with_status(200)
            .with_body(r#"{"accessToken":"SA_TOKEN","expireTime":"2099-01-01T00:00:00Z"}"#)
            .create_async()
            .await;

        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry = RetryConfig::default();

        let credentials = ExternalAccountCredentials {
            token_url: sts_url.clone(),
            audience: "//iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/my-pool/providers/my-provider".into(),
            requested_token_type: "urn:ietf:params:oauth:token-type:access_token".into(),
            credential_source: CredentialSource {
                file: Some(token_file.path().to_str().unwrap().to_owned()),
                environment_id: None,
            },
            service_account_impersonation_url: Some(imp_url),
            scopes: None,
        };

        let provider = GcpWorkloadIdentityProvider {
            credentials,
            sts_endpoint: sts_url,
        };

        let token = provider
            .fetch_token(&client, &service, &retry)
            .await
            .unwrap();

        assert_eq!(token.token.bearer, "SA_TOKEN");
        assert!(token.expiry.is_some());

        _sts_mock.assert_async().await;
        _imp_mock.assert_async().await;
    }
}
