use std::sync::Arc;

use crate::{ClientOptions, Result, RetryConfig};

use self::credential::AwsCredential;
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
    pub sign_payload: bool,
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
