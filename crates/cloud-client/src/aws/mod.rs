use std::sync::Arc;

use futures::future::BoxFuture;

use crate::service::make_service;
use crate::token::TemporaryToken;
use crate::{ClientOptions, RequestSigner, Result, RetryConfig, TokenProvider};

use self::credential::{AssumeRoleProvider, AwsAuthorizer, CredentialExt};
use crate::CredentialProvider;

mod builder;
pub(crate) mod credential;

pub use builder::*;
pub use credential::AwsCredential;

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

/// Assume an AWS IAM role and return temporary credentials.
///
/// Uses the server's ambient AWS credentials (environment variables, instance
/// profile, EKS WebIdentity, etc.) as base credentials, then calls
/// `STS:AssumeRole` to exchange them for temporary credentials scoped to
/// `role_arn`.
///
/// `region` controls which regional STS endpoint is used (defaults to
/// `"us-east-1"`). Pass an optional `sts_endpoint` to override the endpoint
/// URL (useful for LocalStack or other STS emulators in tests).
///
/// Pass an optional `policy` (JSON string) to further restrict the assumed
/// credentials via an inline session policy. The policy is intersected with
/// the role's own policy and can only reduce, never expand, permissions.
///
/// This is used by the Unity Catalog server to vend temporary AWS credentials
/// for external locations backed by an `AwsIamRoleConfig` credential.
pub async fn assume_role(
    role_arn: &str,
    region: &str,
    sts_endpoint: Option<&str>,
    policy: Option<String>,
) -> Result<TemporaryToken<Arc<AwsCredential>>> {
    // Resolve base credentials from the environment (same chain as AmazonBuilder::build).
    let base_config = AmazonBuilder::from_env().with_region(region).build(None)?;
    let base_credentials: AwsCredentialProvider = base_config.credentials;

    let endpoint = sts_endpoint
        .map(|s| s.to_owned())
        .unwrap_or_else(|| format!("https://sts.{region}.amazonaws.com"));

    let provider = AssumeRoleProvider {
        role_arn: role_arn.to_owned(),
        session_name: "UnityCatalogVending".to_owned(),
        endpoint,
        base_credentials,
        region: region.to_owned(),
        policy,
    };

    let client = ClientOptions::default().client()?;
    let service = make_service(client.clone(), None);
    provider
        .fetch_token(&client, &service, &RetryConfig::default())
        .await
}
