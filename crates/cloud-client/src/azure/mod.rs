use std::sync::Arc;

use self::credential::AzureCredential;
use crate::{ClientOptions, CredentialProvider, Result, RetryConfig};

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
