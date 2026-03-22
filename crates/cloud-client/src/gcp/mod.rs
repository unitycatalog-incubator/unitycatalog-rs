use std::sync::Arc;

use futures::future::BoxFuture;

use self::credential::GcpCredential;
use crate::CredentialProvider;
use crate::{ClientOptions, RequestSigner, Result, RetryConfig};

pub use builder::*;

mod builder;
pub(crate) mod credential;

pub type GcpCredentialProvider = Arc<dyn CredentialProvider<Credential = GcpCredential>>;

#[derive(Debug, Clone)]
pub struct GoogleConfig {
    pub credentials: GcpCredentialProvider,

    pub retry_config: RetryConfig,

    pub client_options: ClientOptions,
}

impl GoogleConfig {
    pub(crate) fn new(
        credentials: GcpCredentialProvider,
        retry_config: RetryConfig,
        client_options: ClientOptions,
    ) -> Self {
        Self {
            credentials,
            retry_config,
            client_options,
        }
    }

    pub(crate) async fn get_credential(&self) -> Result<Arc<GcpCredential>> {
        self.credentials.get_credential().await
    }
}

impl RequestSigner for GoogleConfig {
    fn sign<'a>(
        &'a self,
        req: reqwest::RequestBuilder,
    ) -> BoxFuture<'a, Result<reqwest::RequestBuilder>> {
        Box::pin(async move {
            let credential = self.get_credential().await?;
            Ok(req.bearer_auth(&credential.bearer))
        })
    }
}
