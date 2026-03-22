use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use reqwest::{Client, Method};
use serde::Deserialize;

use crate::gcp::GcpCredentialProvider;
use crate::retry::RetryExt;
use crate::service::HttpService;
use crate::token::TemporaryToken;
use crate::{CredentialProvider, RetryConfig, TokenProvider};

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

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

/// OAuth M2M token provider for AWS/GCP Databricks-hosted services.
///
/// POSTs to `{host}/oidc/v1/token` with `grant_type=client_credentials`.
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
}
