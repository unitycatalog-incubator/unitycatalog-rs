use std::sync::Arc;

use futures::future::BoxFuture;

use crate::{ClientOptions, RequestSigner, Result, RetryConfig};

use self::credential::{AwsAuthorizer, AwsCredential, CredentialExt};
use crate::CredentialProvider;

mod builder;
pub(crate) mod credential;

pub use builder::*;

pub type AwsCredentialProvider = Arc<dyn CredentialProvider<Credential = AwsCredential>>;

#[derive(Debug, Clone)]
pub struct AmazonConfig {
    pub region: String,
    pub credentials: AwsCredentialProvider,
    pub retry_config: RetryConfig,
    pub client_options: ClientOptions,
    pub skip_signature: bool,
}

impl AmazonConfig {
    pub(crate) async fn get_credential(&self) -> Result<Option<Arc<AwsCredential>>> {
        Ok(match self.skip_signature {
            false => Some(self.credentials.get_credential().await?),
            true => None,
        })
    }
}

impl RequestSigner for AmazonConfig {
    fn sign<'a>(
        &'a self,
        req: reqwest::RequestBuilder,
    ) -> BoxFuture<'a, Result<reqwest::RequestBuilder>> {
        Box::pin(async move {
            if let Some(cred) = self.get_credential().await? {
                let authorizer = AwsAuthorizer::new(&cred, "execute-api", &self.region);
                Ok(req.with_aws_sigv4(Some(authorizer), None))
            } else {
                Ok(req)
            }
        })
    }
}
