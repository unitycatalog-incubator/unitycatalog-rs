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

use std::str::FromStr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;
use tracing::info;

use super::AmazonConfig;
use crate::aws::credential::{
    AssumeRoleProvider, InstanceCredentialProvider, TaskCredentialProvider, WebIdentityProvider,
};
use crate::aws::{AwsCredential, AwsCredentialProvider};
use crate::config::ConfigValue;
use crate::service::make_service;
use crate::{
    ClientConfigKey, ClientOptions, Result, RetryConfig, StaticCredentialProvider,
    TokenCredentialProvider,
};

static DEFAULT_METADATA_ENDPOINT: &str = "http://169.254.169.254";

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Missing AccessKeyId")]
    MissingAccessKeyId,

    #[error("Missing SecretAccessKey")]
    MissingSecretAccessKey,

    #[error("Configuration key: '{}' is not known.", key)]
    UnknownConfigurationKey { key: String },
}

impl From<Error> for crate::Error {
    fn from(source: Error) -> Self {
        match source {
            Error::UnknownConfigurationKey { key } => Self::UnknownConfigurationKey { key },
            _ => Self::Generic {
                source: Box::new(source),
            },
        }
    }
}

/// Configure AWS authentication credentials.
///
/// # Example
/// ```
/// # let REGION = "foo";
/// # let ACCESS_KEY_ID = "foo";
/// # let SECRET_KEY = "foo";
/// # use cloud_client::aws::AmazonBuilder;
/// let config = AmazonBuilder::new()
///  .with_region(REGION)
///  .with_access_key_id(ACCESS_KEY_ID)
///  .with_secret_access_key(SECRET_KEY)
///  .build(None);
/// ```
#[derive(Debug, Default, Clone)]
pub struct AmazonBuilder {
    access_key_id: Option<String>,
    secret_access_key: Option<String>,
    region: Option<String>,
    token: Option<String>,
    retry_config: RetryConfig,
    imdsv1_fallback: ConfigValue<bool>,
    metadata_endpoint: Option<String>,
    container_credentials_relative_uri: Option<String>,
    client_options: ClientOptions,
    credentials: Option<AwsCredentialProvider>,
    skip_signature: ConfigValue<bool>,
    /// IAM role ARN to assume via STS `AssumeRole`.
    role_arn: Option<String>,
    /// Session name for the assumed role (defaults to `"AssumeRoleSession"`).
    role_session_name: Option<String>,
    /// STS endpoint override for `AssumeRole` (defaults to regional STS).
    sts_endpoint: Option<String>,
}

/// Configuration keys for [`AmazonBuilder`]
///
/// Configuration via keys can be done via [`AmazonBuilder::with_config`]
///
/// # Example
/// ```
/// # use cloud_client::aws::{AmazonBuilder, AmazonS3ConfigKey};
/// let builder = AmazonBuilder::new()
///     .with_config("aws_access_key_id".parse().unwrap(), "my-access-key-id")
///     .with_config(AmazonS3ConfigKey::DefaultRegion, "my-default-region");
/// ```
#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AmazonS3ConfigKey {
    /// AWS Access Key
    ///
    /// Supported keys:
    /// - `aws_access_key_id`
    /// - `access_key_id`
    AccessKeyId,

    /// Secret Access Key
    ///
    /// Supported keys:
    /// - `aws_secret_access_key`
    /// - `secret_access_key`
    SecretAccessKey,

    /// Region
    ///
    /// Supported keys:
    /// - `aws_region`
    /// - `region`
    Region,

    /// Default region
    ///
    /// Supported keys:
    /// - `aws_default_region`
    /// - `default_region`
    DefaultRegion,

    /// Token to use for requests (passed to underlying provider)
    ///
    /// Supported keys:
    /// - `aws_session_token`
    /// - `aws_token`
    /// - `session_token`
    /// - `token`
    Token,

    /// Fall back to ImdsV1
    ///
    /// Supported keys:
    /// - `aws_imdsv1_fallback`
    /// - `imdsv1_fallback`
    ImdsV1Fallback,

    /// Set the instance metadata endpoint
    ///
    /// Supported keys:
    /// - `aws_metadata_endpoint`
    /// - `metadata_endpoint`
    MetadataEndpoint,

    /// Set the container credentials relative URI
    ///
    /// <https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task-iam-roles.html>
    ContainerCredentialsRelativeUri,

    /// Skip signing request
    SkipSignature,

    /// IAM role ARN to assume via STS `AssumeRole`.
    ///
    /// Supported keys:
    /// - `aws_role_arn`
    /// - `role_arn`
    RoleArn,

    /// Session name for the assumed role.
    ///
    /// Supported keys:
    /// - `aws_role_session_name`
    /// - `role_session_name`
    RoleSessionName,

    /// STS endpoint override for `AssumeRole`.
    ///
    /// Supported keys:
    /// - `aws_sts_endpoint`
    /// - `sts_endpoint`
    StsEndpoint,

    /// Client options
    Client(ClientConfigKey),
}

impl AsRef<str> for AmazonS3ConfigKey {
    fn as_ref(&self) -> &str {
        match self {
            Self::AccessKeyId => "aws_access_key_id",
            Self::SecretAccessKey => "aws_secret_access_key",
            Self::Region => "aws_region",
            Self::Token => "aws_session_token",
            Self::ImdsV1Fallback => "aws_imdsv1_fallback",
            Self::DefaultRegion => "aws_default_region",
            Self::MetadataEndpoint => "aws_metadata_endpoint",
            Self::ContainerCredentialsRelativeUri => "aws_container_credentials_relative_uri",
            Self::SkipSignature => "aws_skip_signature",
            Self::RoleArn => "aws_role_arn",
            Self::RoleSessionName => "aws_role_session_name",
            Self::StsEndpoint => "aws_sts_endpoint",
            Self::Client(opt) => opt.as_ref(),
        }
    }
}

impl FromStr for AmazonS3ConfigKey {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "aws_access_key_id" | "access_key_id" => Ok(Self::AccessKeyId),
            "aws_secret_access_key" | "secret_access_key" => Ok(Self::SecretAccessKey),
            "aws_default_region" | "default_region" => Ok(Self::DefaultRegion),
            "aws_region" | "region" => Ok(Self::Region),
            "aws_session_token" | "aws_token" | "session_token" | "token" => Ok(Self::Token),
            "aws_imdsv1_fallback" | "imdsv1_fallback" => Ok(Self::ImdsV1Fallback),
            "aws_metadata_endpoint" | "metadata_endpoint" => Ok(Self::MetadataEndpoint),
            "aws_container_credentials_relative_uri" => Ok(Self::ContainerCredentialsRelativeUri),
            "aws_skip_signature" | "skip_signature" => Ok(Self::SkipSignature),
            "aws_role_arn" | "role_arn" => Ok(Self::RoleArn),
            "aws_role_session_name" | "role_session_name" => Ok(Self::RoleSessionName),
            "aws_sts_endpoint" | "sts_endpoint" => Ok(Self::StsEndpoint),
            "aws_allow_http" => Ok(Self::Client(ClientConfigKey::AllowHttp)),
            _ => match s.strip_prefix("aws_").unwrap_or(s).parse() {
                Ok(key) => Ok(Self::Client(key)),
                Err(_) => Err(Error::UnknownConfigurationKey { key: s.into() }.into()),
            },
        }
    }
}

impl AmazonBuilder {
    /// Create a new [`AmazonBuilder`] with default values.
    pub fn new() -> Self {
        Default::default()
    }

    /// Fill the [`AmazonBuilder`] with regular AWS environment variables
    ///
    /// Variables extracted from environment:
    /// * `AWS_ACCESS_KEY_ID` -> access_key_id
    /// * `AWS_SECRET_ACCESS_KEY` -> secret_access_key
    /// * `AWS_DEFAULT_REGION` -> region
    /// * `AWS_SESSION_TOKEN` -> token
    /// * `AWS_CONTAINER_CREDENTIALS_RELATIVE_URI` -> <https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task-iam-roles.html>
    /// * `AWS_ALLOW_HTTP` -> set to "true" to permit HTTP connections without TLS
    pub fn from_env() -> Self {
        let mut builder: Self = Default::default();

        for (os_key, os_value) in std::env::vars_os() {
            if let (Some(key), Some(value)) = (os_key.to_str(), os_value.to_str()) {
                if key.starts_with("AWS_") {
                    if let Ok(config_key) = key.to_ascii_lowercase().parse() {
                        builder = builder.with_config(config_key, value);
                    }
                }
            }
        }

        builder
    }

    /// Set an option on the builder via a key - value pair.
    pub fn with_config(mut self, key: AmazonS3ConfigKey, value: impl Into<String>) -> Self {
        match key {
            AmazonS3ConfigKey::AccessKeyId => self.access_key_id = Some(value.into()),
            AmazonS3ConfigKey::SecretAccessKey => self.secret_access_key = Some(value.into()),
            AmazonS3ConfigKey::Region => self.region = Some(value.into()),
            AmazonS3ConfigKey::Token => self.token = Some(value.into()),
            AmazonS3ConfigKey::ImdsV1Fallback => self.imdsv1_fallback.parse(value),
            AmazonS3ConfigKey::DefaultRegion => {
                self.region = self.region.or_else(|| Some(value.into()))
            }
            AmazonS3ConfigKey::MetadataEndpoint => self.metadata_endpoint = Some(value.into()),
            AmazonS3ConfigKey::ContainerCredentialsRelativeUri => {
                self.container_credentials_relative_uri = Some(value.into())
            }
            AmazonS3ConfigKey::Client(key) => {
                self.client_options = self.client_options.with_config(key, value)
            }
            AmazonS3ConfigKey::SkipSignature => self.skip_signature.parse(value),
            AmazonS3ConfigKey::RoleArn => self.role_arn = Some(value.into()),
            AmazonS3ConfigKey::RoleSessionName => self.role_session_name = Some(value.into()),
            AmazonS3ConfigKey::StsEndpoint => self.sts_endpoint = Some(value.into()),
        };
        self
    }

    /// Get config value via a [`AmazonS3ConfigKey`].
    pub fn get_config_value(&self, key: &AmazonS3ConfigKey) -> Option<String> {
        match key {
            AmazonS3ConfigKey::AccessKeyId => self.access_key_id.clone(),
            AmazonS3ConfigKey::SecretAccessKey => self.secret_access_key.clone(),
            AmazonS3ConfigKey::Region | AmazonS3ConfigKey::DefaultRegion => self.region.clone(),
            AmazonS3ConfigKey::Token => self.token.clone(),
            AmazonS3ConfigKey::ImdsV1Fallback => Some(self.imdsv1_fallback.to_string()),
            AmazonS3ConfigKey::MetadataEndpoint => self.metadata_endpoint.clone(),
            AmazonS3ConfigKey::Client(key) => self.client_options.get_config_value(key),
            AmazonS3ConfigKey::ContainerCredentialsRelativeUri => {
                self.container_credentials_relative_uri.clone()
            }
            AmazonS3ConfigKey::SkipSignature => Some(self.skip_signature.to_string()),
            AmazonS3ConfigKey::RoleArn => self.role_arn.clone(),
            AmazonS3ConfigKey::RoleSessionName => self.role_session_name.clone(),
            AmazonS3ConfigKey::StsEndpoint => self.sts_endpoint.clone(),
        }
    }

    /// Set the AWS Access Key
    pub fn with_access_key_id(mut self, access_key_id: impl Into<String>) -> Self {
        self.access_key_id = Some(access_key_id.into());
        self
    }

    /// Set the AWS Secret Access Key
    pub fn with_secret_access_key(mut self, secret_access_key: impl Into<String>) -> Self {
        self.secret_access_key = Some(secret_access_key.into());
        self
    }

    /// Set the AWS Session Token to use for requests
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Set the region, defaults to `us-east-1`
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Set the credential provider overriding any other options
    pub fn with_credentials(mut self, credentials: AwsCredentialProvider) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Sets what protocol is allowed. If `allow_http` is :
    /// * false (default):  Only HTTPS are allowed
    /// * true:  HTTP and HTTPS are allowed
    pub fn with_allow_http(mut self, allow_http: bool) -> Self {
        self.client_options = self.client_options.with_allow_http(allow_http);
        self
    }

    /// Set the retry configuration
    pub fn with_retry(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }

    /// By default instance credentials will only be fetched over [IMDSv2], as AWS recommends
    /// against having IMDSv1 enabled on EC2 instances as it is vulnerable to [SSRF attack]
    ///
    /// However, certain deployment environments, such as those running old versions of kube2iam,
    /// may not support IMDSv2. This option will enable automatic fallback to using IMDSv1
    /// if the token endpoint returns a 403 error indicating that IMDSv2 is not supported.
    ///
    /// [IMDSv2]: https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/configuring-instance-metadata-service.html
    /// [SSRF attack]: https://aws.amazon.com/blogs/security/defense-in-depth-open-firewalls-reverse-proxies-ssrf-vulnerabilities-ec2-instance-metadata-service/
    pub fn with_imdsv1_fallback(mut self) -> Self {
        self.imdsv1_fallback = true.into();
        self
    }

    /// If enabled, requests will not be signed.
    pub fn with_skip_signature(mut self, skip_signature: bool) -> Self {
        self.skip_signature = skip_signature.into();
        self
    }

    /// Set the [instance metadata endpoint](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-instance-metadata.html),
    /// used primarily within AWS EC2.
    ///
    /// This defaults to the IPv4 endpoint: http://169.254.169.254. One can alternatively use the IPv6
    /// endpoint http://fd00:ec2::254.
    pub fn with_metadata_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.metadata_endpoint = Some(endpoint.into());
        self
    }

    /// Assume the given IAM role via STS `AssumeRole` after obtaining base credentials.
    ///
    /// When set, the builder resolves base credentials (static, IMDS, or WebIdentity)
    /// and then exchanges them for temporary credentials scoped to `role_arn`.
    ///
    /// # References
    /// - <https://docs.aws.amazon.com/STS/latest/APIReference/API_AssumeRole.html>
    pub fn with_role_arn(mut self, role_arn: impl Into<String>) -> Self {
        self.role_arn = Some(role_arn.into());
        self
    }

    /// Set the session name used in `AssumeRole` requests (defaults to `"AssumeRoleSession"`).
    pub fn with_role_session_name(mut self, session_name: impl Into<String>) -> Self {
        self.role_session_name = Some(session_name.into());
        self
    }

    /// Override the STS endpoint used for `AssumeRole` (defaults to the regional endpoint).
    pub fn with_sts_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.sts_endpoint = Some(endpoint.into());
        self
    }

    /// Set the proxy_url to be used by the underlying client
    pub fn with_proxy_url(mut self, proxy_url: impl Into<String>) -> Self {
        self.client_options = self.client_options.with_proxy_url(proxy_url);
        self
    }

    /// Set a trusted proxy CA certificate
    pub fn with_proxy_ca_certificate(mut self, proxy_ca_certificate: impl Into<String>) -> Self {
        self.client_options = self
            .client_options
            .with_proxy_ca_certificate(proxy_ca_certificate);
        self
    }

    /// Set a list of hosts to exclude from proxy connections
    pub fn with_proxy_excludes(mut self, proxy_excludes: impl Into<String>) -> Self {
        self.client_options = self.client_options.with_proxy_excludes(proxy_excludes);
        self
    }

    /// Sets the client options, overriding any already set
    pub fn with_client_options(mut self, options: ClientOptions) -> Self {
        self.client_options = options;
        self
    }

    /// Build an [`AmazonConfig`] from the provided values, consuming `self`.
    ///
    /// If `runtime` is provided, all HTTP I/O (including credential refresh)
    /// will be spawned on the given runtime handle.
    pub fn build(self, runtime: Option<&Handle>) -> Result<AmazonConfig> {
        let region = self.region.unwrap_or_else(|| "us-east-1".to_string());

        let credentials = if let Some(credentials) = self.credentials {
            credentials
        } else if self.access_key_id.is_some() || self.secret_access_key.is_some() {
            match (self.access_key_id, self.secret_access_key, self.token) {
                (Some(key_id), Some(secret_key), token) => {
                    info!("Using Static credential provider");
                    let credential = AwsCredential {
                        key_id,
                        secret_key,
                        token,
                    };
                    Arc::new(StaticCredentialProvider::new(credential)) as _
                }
                (None, Some(_), _) => return Err(Error::MissingAccessKeyId.into()),
                (Some(_), None, _) => return Err(Error::MissingSecretAccessKey.into()),
                (None, None, _) => unreachable!(),
            }
        } else if let (Ok(token_path), Ok(role_arn)) = (
            std::env::var("AWS_WEB_IDENTITY_TOKEN_FILE"),
            std::env::var("AWS_ROLE_ARN"),
        ) {
            info!("Using WebIdentity credential provider");

            let session_name = std::env::var("AWS_ROLE_SESSION_NAME")
                .unwrap_or_else(|_| "WebIdentitySession".to_string());

            let endpoint = format!("https://sts.{region}.amazonaws.com");

            let client = self
                .client_options
                .clone()
                .with_allow_http(false)
                .client()?;

            let token = WebIdentityProvider {
                token_path,
                session_name,
                role_arn,
                endpoint,
            };

            let service = make_service(client.clone(), runtime);
            Arc::new(TokenCredentialProvider::new(
                token,
                client,
                service,
                self.retry_config.clone(),
            )) as _
        } else if let Ok(full_uri) = std::env::var("AWS_CONTAINER_CREDENTIALS_FULL_URI") {
            // EKS Pod Identity and Lambda use a full absolute URI
            info!("Using Task credential provider (full URI)");
            let auth_token_file = std::env::var("AWS_CONTAINER_AUTHORIZATION_TOKEN_FILE").ok();
            let client = self.client_options.clone().with_allow_http(true).client()?;
            let service = make_service(client.clone(), runtime);
            Arc::new(TaskCredentialProvider {
                url: full_uri,
                auth_token_file,
                retry: self.retry_config.clone(),
                client,
                service,
                cache: Default::default(),
            }) as _
        } else if let Some(uri) = self.container_credentials_relative_uri {
            info!("Using Task credential provider (relative URI)");
            let auth_token_file = std::env::var("AWS_CONTAINER_AUTHORIZATION_TOKEN_FILE").ok();
            let client = self.client_options.clone().with_allow_http(true).client()?;
            let service = make_service(client.clone(), runtime);
            Arc::new(TaskCredentialProvider {
                url: format!("http://169.254.170.2{uri}"),
                auth_token_file,
                retry: self.retry_config.clone(),
                client,
                service,
                cache: Default::default(),
            }) as _
        } else {
            info!("Using Instance credential provider");

            let token = InstanceCredentialProvider {
                imdsv1_fallback: self.imdsv1_fallback.get()?,
                metadata_endpoint: self
                    .metadata_endpoint
                    .unwrap_or_else(|| DEFAULT_METADATA_ENDPOINT.into()),
            };

            let client = self.client_options.metadata_client()?;
            let service = make_service(client.clone(), runtime);
            Arc::new(TokenCredentialProvider::new(
                token,
                client,
                service,
                self.retry_config.clone(),
            )) as _
        };

        // Optionally wrap base credentials with AssumeRole if a role ARN is configured.
        let credentials = if let Some(role_arn) = self.role_arn {
            info!("Wrapping credentials with AssumeRole provider");
            let session_name = self
                .role_session_name
                .unwrap_or_else(|| "AssumeRoleSession".to_string());
            let endpoint = self
                .sts_endpoint
                .unwrap_or_else(|| format!("https://sts.{region}.amazonaws.com"));
            let client = self
                .client_options
                .clone()
                .with_allow_http(false)
                .client()?;
            let service = make_service(client.clone(), runtime);
            Arc::new(TokenCredentialProvider::new(
                AssumeRoleProvider {
                    role_arn,
                    session_name,
                    endpoint,
                    base_credentials: credentials,
                    region: region.clone(),
                    policy: None,
                },
                client,
                service,
                self.retry_config.clone(),
            )) as _
        } else {
            credentials
        };

        Ok(AmazonConfig {
            region,
            credentials,
            retry_config: self.retry_config,
            client_options: self.client_options,
            skip_signature: self.skip_signature.get()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn s3_test_config_from_map() {
        let aws_access_key_id = "object_store:fake_access_key_id".to_string();
        let aws_secret_access_key = "object_store:fake_secret_key".to_string();
        let aws_default_region = "object_store:fake_default_region".to_string();
        let aws_session_token = "object_store:fake_session_token".to_string();
        let options = HashMap::from([
            ("aws_access_key_id", aws_access_key_id.clone()),
            ("aws_secret_access_key", aws_secret_access_key),
            ("aws_default_region", aws_default_region.clone()),
            ("aws_session_token", aws_session_token.clone()),
        ]);

        let builder = options
            .into_iter()
            .fold(AmazonBuilder::new(), |builder, (key, value)| {
                builder.with_config(key.parse().unwrap(), value)
            })
            .with_config(AmazonS3ConfigKey::SecretAccessKey, "new-secret-key");

        assert_eq!(builder.access_key_id.unwrap(), aws_access_key_id.as_str());
        assert_eq!(builder.secret_access_key.unwrap(), "new-secret-key");
        assert_eq!(builder.region.unwrap(), aws_default_region);
        assert_eq!(builder.token.unwrap(), aws_session_token);
    }

    #[test]
    fn s3_test_config_get_value() {
        let aws_access_key_id = "object_store:fake_access_key_id".to_string();
        let aws_secret_access_key = "object_store:fake_secret_key".to_string();
        let aws_default_region = "object_store:fake_default_region".to_string();
        let aws_session_token = "object_store:fake_session_token".to_string();

        let builder = AmazonBuilder::new()
            .with_config(AmazonS3ConfigKey::AccessKeyId, &aws_access_key_id)
            .with_config(AmazonS3ConfigKey::SecretAccessKey, &aws_secret_access_key)
            .with_config(AmazonS3ConfigKey::DefaultRegion, &aws_default_region)
            .with_config(AmazonS3ConfigKey::Token, &aws_session_token);

        assert_eq!(
            builder
                .get_config_value(&AmazonS3ConfigKey::AccessKeyId)
                .unwrap(),
            aws_access_key_id
        );
        assert_eq!(
            builder
                .get_config_value(&AmazonS3ConfigKey::SecretAccessKey)
                .unwrap(),
            aws_secret_access_key
        );
        assert_eq!(
            builder
                .get_config_value(&AmazonS3ConfigKey::DefaultRegion)
                .unwrap(),
            aws_default_region
        );
        assert_eq!(
            builder.get_config_value(&AmazonS3ConfigKey::Token).unwrap(),
            aws_session_token
        );
    }

    #[test]
    fn s3_default_region() {
        let config = AmazonBuilder::new().build(None).unwrap();
        assert_eq!(config.region, "us-east-1");
    }

    #[tokio::test]
    async fn s3_test_proxy_url() {
        let s3 = AmazonBuilder::new()
            .with_access_key_id("access_key_id")
            .with_secret_access_key("secret_access_key")
            .with_region("region")
            .with_allow_http(true)
            .with_proxy_url("https://example.com")
            .build(None);

        assert!(s3.is_ok());
    }

    #[test]
    fn test_invalid_config() {
        let err = AmazonBuilder::new()
            .with_config(AmazonS3ConfigKey::ImdsV1Fallback, "enabled")
            .with_region("region")
            .build(None)
            .unwrap_err()
            .to_string();

        assert_eq!(err, "Generic error: failed to parse \"enabled\" as boolean");
    }

    #[test]
    fn aws_test_client_opts() {
        let key = "AWS_PROXY_URL";
        if let Ok(config_key) = key.to_ascii_lowercase().parse() {
            assert_eq!(
                AmazonS3ConfigKey::Client(ClientConfigKey::ProxyUrl),
                config_key
            );
        } else {
            panic!("{key} not propagated as ClientConfigKey");
        }
    }
}
