use std::sync::Arc;

use futures::future::BoxFuture;

use self::credential::AzureCredential;
use crate::{ClientOptions, CredentialProvider, RequestSigner, Result, RetryConfig};

mod builder;
pub(crate) mod credential;

pub(crate) use self::credential::*;
pub use builder::*;

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
