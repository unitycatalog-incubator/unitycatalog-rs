use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use base64::Engine as _;
use reqwest::{Client, Method};
use serde::Deserialize;

use crate::gcp::GcpCredentialProvider;
use crate::retry::RetryExt;
use crate::service::HttpService;
use crate::token::TemporaryToken;
use crate::{RetryConfig, TokenProvider};

/// Bearer token credential used by all Databricks auth paths.
#[derive(Debug, Eq, PartialEq)]
pub struct DatabricksCredential {
    pub bearer: String,
}

/// Response shape shared by all Databricks OIDC token endpoints.
#[derive(Deserialize, Debug)]
pub(crate) struct DatabricksTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Error performing Databricks token request: {}", source)]
    TokenRequest { source: crate::retry::Error },

    #[error("Error reading Databricks token response body: {}", source)]
    TokenResponseBody { source: reqwest::Error },
}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        Self::Generic {
            source: Box::new(value),
        }
    }
}

/// OAuth M2M token provider for AWS/GCP Databricks-hosted services.
///
/// POSTs to `{host}/oidc/v1/token` with `grant_type=client_credentials` using
/// a `client_id` / `client_secret` pair.  The returned bearer token is cached
/// until it approaches expiry.  This is the recommended auth method for
/// service-to-service calls against Databricks REST APIs.
///
/// # References
/// - <https://docs.databricks.com/en/dev-tools/auth/oauth-m2m.html>
#[derive(Debug)]
pub(crate) struct DatabricksM2MProvider {
    pub token_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub scope: String,
}

#[async_trait]
impl TokenProvider for DatabricksM2MProvider {
    type Credential = DatabricksCredential;

    async fn fetch_token(
        &self,
        client: &Client,
        service: &Arc<dyn HttpService>,
        retry: &RetryConfig,
    ) -> crate::Result<TemporaryToken<Arc<DatabricksCredential>>> {
        let response: DatabricksTokenResponse = client
            .request(Method::POST, &self.token_url)
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("scope", self.scope.as_str()),
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
            token: Arc::new(DatabricksCredential {
                bearer: response.access_token,
            }),
            expiry: Some(Instant::now() + Duration::from_secs(response.expires_in)),
        })
    }
}

/// GCP service-account bearer → Databricks OIDC token exchange provider.
///
/// Uses `grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer` to exchange
/// a GCP service account JWT for a Databricks OIDC access token.
#[derive(Debug)]
pub(crate) struct DatabricksGcpTokenExchangeProvider {
    pub token_url: String,
    pub gcp_provider: GcpCredentialProvider,
}

#[async_trait]
impl TokenProvider for DatabricksGcpTokenExchangeProvider {
    type Credential = DatabricksCredential;

    async fn fetch_token(
        &self,
        client: &Client,
        service: &Arc<dyn HttpService>,
        retry: &RetryConfig,
    ) -> crate::Result<TemporaryToken<Arc<DatabricksCredential>>> {
        let gcp_cred = self.gcp_provider.get_credential().await?;

        let response: DatabricksTokenResponse = client
            .request(Method::POST, &self.token_url)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", gcp_cred.bearer.as_str()),
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
            token: Arc::new(DatabricksCredential {
                bearer: response.access_token,
            }),
            expiry: Some(Instant::now() + Duration::from_secs(response.expires_in)),
        })
    }
}

/// Decode the `exp` claim from a JWT payload and return an `Instant` representing the expiry.
///
/// Does not verify the signature — we are the intended recipient.
pub(crate) fn jwt_exp(token: &str) -> Option<Instant> {
    let payload = token.split('.').nth(1)?;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .ok()?;
    let v: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    let exp = v["exp"].as_u64()?;
    let now_secs = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    let secs_remaining = exp.checked_sub(now_secs)?;
    Some(Instant::now() + Duration::from_secs(secs_remaining))
}

/// OIDC token provider that reads a token from an environment variable on each fetch.
///
/// The env var value is re-read on every call so rotated tokens are picked up
/// automatically without restarting the process.  The JWT `exp` claim is
/// parsed (without verifying the signature) to set the [`TemporaryToken`]
/// expiry so the surrounding [`TokenCache`] knows when to refresh.  Useful in
/// CI/CD pipelines or Kubernetes environments where a sidecar injects a fresh
/// OIDC token into an environment variable.
#[derive(Debug)]
pub(crate) struct OidcEnvTokenProvider {
    /// Name of the environment variable holding the OIDC JWT.
    pub env_var: String,
}

#[async_trait]
impl TokenProvider for OidcEnvTokenProvider {
    type Credential = DatabricksCredential;

    async fn fetch_token(
        &self,
        _client: &Client,
        _service: &Arc<dyn HttpService>,
        _retry: &RetryConfig,
    ) -> crate::Result<TemporaryToken<Arc<DatabricksCredential>>> {
        let token = std::env::var(&self.env_var).map_err(|_| crate::Error::Generic {
            source: format!(
                "env-oidc: environment variable {:?} is not set",
                self.env_var
            )
            .into(),
        })?;
        let expiry = jwt_exp(&token);
        Ok(TemporaryToken {
            token: Arc::new(DatabricksCredential { bearer: token }),
            expiry,
        })
    }
}

/// OIDC token provider that reads a token from a file on each fetch.
///
/// The file is re-read on every call so kubelet-rotated tokens are picked up
/// automatically without restarting the process.  The JWT `exp` claim is
/// parsed (without verifying the signature) to set the [`TemporaryToken`]
/// expiry so the surrounding [`TokenCache`] knows when to refresh.  The
/// canonical use-case is Kubernetes workload-identity federation where the
/// kubelet writes a fresh projected service-account token to a well-known
/// path (e.g. `/var/run/secrets/azure/tokens/azure-identity-token`).
#[derive(Debug)]
pub(crate) struct OidcFileTokenProvider {
    /// Path to the file containing the OIDC JWT.
    pub filepath: String,
}

#[async_trait]
impl TokenProvider for OidcFileTokenProvider {
    type Credential = DatabricksCredential;

    async fn fetch_token(
        &self,
        _client: &Client,
        _service: &Arc<dyn HttpService>,
        _retry: &RetryConfig,
    ) -> crate::Result<TemporaryToken<Arc<DatabricksCredential>>> {
        let token = std::fs::read_to_string(&self.filepath).map_err(|e| crate::Error::Generic {
            source: format!(
                "file-oidc: failed to read token from {:?}: {e}",
                self.filepath
            )
            .into(),
        })?;
        let token = token.trim().to_owned();
        let expiry = jwt_exp(&token);
        Ok(TemporaryToken {
            token: Arc::new(DatabricksCredential { bearer: token }),
            expiry,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::ReqwestService;

    #[tokio::test]
    async fn test_m2m_fetch_token() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock("POST", "/oidc/v1/token")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::Regex("grant_type=client_credentials".into()),
                mockito::Matcher::Regex("client_id=myid".into()),
                mockito::Matcher::Regex("scope=unity-catalog".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"access_token":"DBTOKEN","expires_in":3600,"token_type":"Bearer"}"#)
            .create_async()
            .await;

        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry = RetryConfig::default();

        let provider = DatabricksM2MProvider {
            token_url: format!("{}/oidc/v1/token", server.url()),
            client_id: "myid".into(),
            client_secret: "mysecret".into(),
            scope: "unity-catalog".into(),
        };

        let token = provider
            .fetch_token(&client, &service, &retry)
            .await
            .unwrap();
        assert_eq!(token.token.bearer, "DBTOKEN");
        assert!(token.expiry.is_some());
    }

    /// Build a minimal JWT with a given `exp` Unix timestamp (no signature).
    fn make_jwt(exp: u64) -> String {
        let header = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(r#"{"alg":"none","typ":"JWT"}"#);
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(format!(r#"{{"sub":"test","exp":{exp}}}"#));
        format!("{header}.{payload}.sig")
    }

    #[test]
    fn test_jwt_exp_future() {
        let future_exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600;
        let token = make_jwt(future_exp);
        let expiry = jwt_exp(&token);
        assert!(expiry.is_some());
        // Should expire roughly 1 hour from now (allow ±5s).
        let remaining = expiry.unwrap().duration_since(Instant::now());
        assert!(remaining.as_secs() > 3590 && remaining.as_secs() <= 3600);
    }

    #[test]
    fn test_jwt_exp_already_expired() {
        // exp in the past → checked_sub underflows → None
        let past_exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .saturating_sub(60);
        let token = make_jwt(past_exp);
        assert!(jwt_exp(&token).is_none());
    }

    #[tokio::test]
    async fn test_env_oidc_provider() {
        let future_exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600;
        let jwt = make_jwt(future_exp);
        let env_var = "TEST_OIDC_TOKEN_UNIQUE_KEY_12345";
        // SAFETY: single-threaded test, no concurrent env access.
        unsafe { std::env::set_var(env_var, &jwt) };

        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry = RetryConfig::default();

        let provider = OidcEnvTokenProvider {
            env_var: env_var.to_owned(),
        };
        let token = provider
            .fetch_token(&client, &service, &retry)
            .await
            .unwrap();
        assert_eq!(token.token.bearer, jwt);
        assert!(token.expiry.is_some());

        // SAFETY: single-threaded test, no concurrent env access.
        unsafe { std::env::remove_var(env_var) };
    }

    #[tokio::test]
    async fn test_file_oidc_provider() {
        use std::io::Write as _;
        let future_exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600;
        let jwt = make_jwt(future_exp);

        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(tmp, "{jwt}").unwrap();

        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry = RetryConfig::default();

        let provider = OidcFileTokenProvider {
            filepath: tmp.path().to_str().unwrap().to_owned(),
        };
        let token = provider
            .fetch_token(&client, &service, &retry)
            .await
            .unwrap();
        assert_eq!(token.token.bearer, jwt);
        assert!(token.expiry.is_some());
    }

    #[tokio::test]
    async fn test_m2m_token_refresh() {
        let mut server = mockito::Server::new_async().await;

        // First call
        let _mock1 = server
            .mock("POST", "/oidc/v1/token")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::Regex("grant_type=client_credentials".into()),
                mockito::Matcher::Regex("client_id=refresh-client".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"access_token":"TOKEN_FIRST","expires_in":1,"token_type":"Bearer"}"#)
            .create_async()
            .await;

        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry = RetryConfig::default();

        let provider = DatabricksM2MProvider {
            token_url: format!("{}/oidc/v1/token", server.url()),
            client_id: "refresh-client".into(),
            client_secret: "refresh-secret".into(),
            scope: "unity-catalog".into(),
        };

        let token1 = provider
            .fetch_token(&client, &service, &retry)
            .await
            .unwrap();
        assert_eq!(token1.token.bearer, "TOKEN_FIRST");
        // expires_in = 1 second — expiry should be very soon
        assert!(token1.expiry.is_some());

        // Second call — simulate token refresh after expiry
        let _mock2 = server
            .mock("POST", "/oidc/v1/token")
            .with_status(200)
            .with_body(r#"{"access_token":"TOKEN_SECOND","expires_in":3600,"token_type":"Bearer"}"#)
            .create_async()
            .await;

        let token2 = provider
            .fetch_token(&client, &service, &retry)
            .await
            .unwrap();
        assert_eq!(token2.token.bearer, "TOKEN_SECOND");

        _mock1.assert_async().await;
        _mock2.assert_async().await;
    }

    #[tokio::test]
    async fn test_uc_credential_vending_aws() {
        // Test that DatabricksM2MProvider correctly fetches tokens which
        // can then be used as bearer tokens for UC credential vending calls.
        let mut server = mockito::Server::new_async().await;

        let _token_mock = server
            .mock("POST", "/oidc/v1/token")
            .with_status(200)
            .with_body(
                r#"{"access_token":"UC_BEARER_TOKEN","expires_in":3600,"token_type":"Bearer"}"#,
            )
            .create_async()
            .await;

        let _creds_mock = server
            .mock("POST", "/api/2.1/unity-catalog/temporary-table-credentials")
            .match_header("Authorization", "Bearer UC_BEARER_TOKEN")
            .with_status(200)
            .with_body(
                r#"{
                "aws_temp_credentials": {
                    "access_key_id": "ASIATESTVENDING",
                    "secret_access_key": "vendingsecret",
                    "session_token": "vendingtoken"
                },
                "expiration_time": "2099-01-01T00:00:00Z"
            }"#,
            )
            .create_async()
            .await;

        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry = RetryConfig::default();

        let provider = DatabricksM2MProvider {
            token_url: format!("{}/oidc/v1/token", server.url()),
            client_id: "uc-client".into(),
            client_secret: "uc-secret".into(),
            scope: "unity-catalog".into(),
        };

        let token = provider
            .fetch_token(&client, &service, &retry)
            .await
            .unwrap();

        // Simulate using the token to call the credential vending endpoint
        let vend_resp = client
            .request(
                Method::POST,
                format!(
                    "{}/api/2.1/unity-catalog/temporary-table-credentials",
                    server.url()
                ),
            )
            .bearer_auth(&token.token.bearer)
            .send()
            .await
            .unwrap();

        assert!(vend_resp.status().is_success());

        let body: serde_json::Value = vend_resp.json().await.unwrap();
        assert_eq!(
            body["aws_temp_credentials"]["access_key_id"],
            "ASIATESTVENDING"
        );

        _token_mock.assert_async().await;
        _creds_mock.assert_async().await;
    }
}
