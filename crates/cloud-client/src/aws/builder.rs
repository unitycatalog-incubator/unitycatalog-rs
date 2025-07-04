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

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use md5::{Digest, Md5};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;

use super::AmazonConfig;
use crate::aws::credential::{
    InstanceCredentialProvider, SessionProvider, TaskCredentialProvider, WebIdentityProvider,
};
use crate::aws::{AwsCredential, AwsCredentialProvider, Checksum};
use crate::config::ConfigValue;
use crate::{
    ClientConfigKey, ClientOptions, Result, RetryConfig, StaticCredentialProvider,
    TokenCredentialProvider,
};

/// Default metadata endpoint
static DEFAULT_METADATA_ENDPOINT: &str = "http://169.254.169.254";

/// A specialized `Error` for object store-related errors
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Missing bucket name")]
    MissingBucketName,

    #[error("Missing AccessKeyId")]
    MissingAccessKeyId,

    #[error("Missing SecretAccessKey")]
    MissingSecretAccessKey,

    #[error("Unable parse source url. Url: {}, Error: {}", url, source)]
    UnableToParseUrl {
        source: url::ParseError,
        url: String,
    },

    #[error(
        "Unknown url scheme cannot be parsed into storage location: {}",
        scheme
    )]
    UnknownUrlScheme { scheme: String },

    #[error("URL did not match any known pattern for scheme: {}", url)]
    UrlNotRecognised { url: String },

    #[error("Configuration key: '{}' is not known.", key)]
    UnknownConfigurationKey { key: String },

    #[error("Invalid Zone suffix for bucket '{bucket}'")]
    ZoneSuffix { bucket: String },

    #[error(
        "Invalid encryption type: {}. Valid values are \"AES256\", \"sse:kms\", \"sse:kms:dsse\" and \"sse-c\".",
        passed
    )]
    InvalidEncryptionType { passed: String },

    #[error(
        "Invalid encryption header values. Header: {}, source: {}",
        header,
        source
    )]
    InvalidEncryptionHeader {
        header: &'static str,
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
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

/// Configure a connection to Amazon S3 using the specified credentials in
/// the specified Amazon region and bucket.
///
/// # Example
/// ```
/// # let REGION = "foo";
/// # let BUCKET_NAME = "foo";
/// # let ACCESS_KEY_ID = "foo";
/// # let SECRET_KEY = "foo";
/// # use cloud_client::aws::AmazonBuilder;
/// let s3 = AmazonBuilder::new()
///  .with_region(REGION)
///  .with_access_key_id(ACCESS_KEY_ID)
///  .with_secret_access_key(SECRET_KEY)
///  .build();
/// ```
#[derive(Debug, Default, Clone)]
pub struct AmazonBuilder {
    /// Access key id
    access_key_id: Option<String>,
    /// Secret access_key
    secret_access_key: Option<String>,
    /// Region
    region: Option<String>,
    /// Token to use for requests
    token: Option<String>,
    /// Url
    url: Option<String>,
    /// Retry config
    retry_config: RetryConfig,
    /// When set to true, fallback to IMDSv1
    imdsv1_fallback: ConfigValue<bool>,
    /// When set to true, unsigned payload option has to be used
    unsigned_payload: ConfigValue<bool>,
    /// Checksum algorithm which has to be used for object integrity check during upload
    checksum_algorithm: Option<ConfigValue<Checksum>>,
    /// Metadata endpoint, see <https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-instance-metadata.html>
    metadata_endpoint: Option<String>,
    /// Container credentials URL, see <https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task-iam-roles.html>
    container_credentials_relative_uri: Option<String>,
    /// Client options
    client_options: ClientOptions,
    /// Credentials
    credentials: Option<AwsCredentialProvider>,
    /// Skip signing requests
    skip_signature: ConfigValue<bool>,
    /// Ignore tags
    disable_tagging: ConfigValue<bool>,
    /// Encryption (See [`S3EncryptionConfigKey`])
    encryption_type: Option<ConfigValue<S3EncryptionType>>,
    encryption_kms_key_id: Option<String>,
    encryption_bucket_key_enabled: Option<ConfigValue<bool>>,
    /// base64-encoded 256-bit customer encryption key for SSE-C.
    encryption_customer_key_base64: Option<String>,
    /// When set to true, charge requester for bucket operations
    request_payer: ConfigValue<bool>,
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
    /// See [`AmazonBuilder::with_access_key_id`] for details.
    ///
    /// Supported keys:
    /// - `aws_access_key_id`
    /// - `access_key_id`
    AccessKeyId,

    /// Secret Access Key
    ///
    /// See [`AmazonBuilder::with_secret_access_key`] for details.
    ///
    /// Supported keys:
    /// - `aws_secret_access_key`
    /// - `secret_access_key`
    SecretAccessKey,

    /// Region
    ///
    /// See [`AmazonBuilder::with_region`] for details.
    ///
    /// Supported keys:
    /// - `aws_region`
    /// - `region`
    Region,

    /// Default region
    ///
    /// See [`AmazonBuilder::with_region`] for details.
    ///
    /// Supported keys:
    /// - `aws_default_region`
    /// - `default_region`
    DefaultRegion,

    /// Token to use for requests (passed to underlying provider)
    ///
    /// See [`AmazonBuilder::with_token`] for details.
    ///
    /// Supported keys:
    /// - `aws_session_token`
    /// - `aws_token`
    /// - `session_token`
    /// - `token`
    Token,

    /// Fall back to ImdsV1
    ///
    /// See [`AmazonBuilder::with_imdsv1_fallback`] for details.
    ///
    /// Supported keys:
    /// - `aws_imdsv1_fallback`
    /// - `imdsv1_fallback`
    ImdsV1Fallback,

    /// Avoid computing payload checksum when calculating signature.
    ///
    /// See [`AmazonBuilder::with_unsigned_payload`] for details.
    ///
    /// Supported keys:
    /// - `aws_unsigned_payload`
    /// - `unsigned_payload`
    UnsignedPayload,

    /// Set the checksum algorithm for this client
    ///
    /// See [`AmazonBuilder::with_checksum_algorithm`]
    Checksum,

    /// Set the instance metadata endpoint
    ///
    /// See [`AmazonBuilder::with_metadata_endpoint`] for details.
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

    /// Disable tagging objects
    ///
    /// This can be desirable if not supported by the backing store
    ///
    /// Supported keys:
    /// - `aws_disable_tagging`
    /// - `disable_tagging`
    DisableTagging,

    /// Enable Support for S3 Requester Pays
    ///
    /// Supported keys:
    /// - `aws_request_payer`
    /// - `request_payer`
    RequestPayer,

    /// Client options
    Client(ClientConfigKey),

    /// Encryption options
    Encryption(S3EncryptionConfigKey),
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
            Self::UnsignedPayload => "aws_unsigned_payload",
            Self::Checksum => "aws_checksum_algorithm",
            Self::ContainerCredentialsRelativeUri => "aws_container_credentials_relative_uri",
            Self::SkipSignature => "aws_skip_signature",
            Self::DisableTagging => "aws_disable_tagging",
            Self::RequestPayer => "aws_request_payer",
            Self::Client(opt) => opt.as_ref(),
            Self::Encryption(opt) => opt.as_ref(),
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
            "aws_unsigned_payload" | "unsigned_payload" => Ok(Self::UnsignedPayload),
            "aws_checksum_algorithm" | "checksum_algorithm" => Ok(Self::Checksum),
            "aws_container_credentials_relative_uri" => Ok(Self::ContainerCredentialsRelativeUri),
            "aws_skip_signature" | "skip_signature" => Ok(Self::SkipSignature),
            "aws_disable_tagging" | "disable_tagging" => Ok(Self::DisableTagging),
            "aws_request_payer" | "request_payer" => Ok(Self::RequestPayer),
            // Backwards compatibility
            "aws_allow_http" => Ok(Self::Client(ClientConfigKey::AllowHttp)),
            "aws_server_side_encryption" => Ok(Self::Encryption(
                S3EncryptionConfigKey::ServerSideEncryption,
            )),
            "aws_sse_kms_key_id" => Ok(Self::Encryption(S3EncryptionConfigKey::KmsKeyId)),
            "aws_sse_bucket_key_enabled" => {
                Ok(Self::Encryption(S3EncryptionConfigKey::BucketKeyEnabled))
            }
            "aws_sse_customer_key_base64" => Ok(Self::Encryption(
                S3EncryptionConfigKey::CustomerEncryptionKey,
            )),
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
    /// * `AWS_ENDPOINT` -> endpoint
    /// * `AWS_SESSION_TOKEN` -> token
    /// * `AWS_CONTAINER_CREDENTIALS_RELATIVE_URI` -> <https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task-iam-roles.html>
    /// * `AWS_ALLOW_HTTP` -> set to "true" to permit HTTP connections without TLS
    /// # Example
    /// ```
    /// use cloud_client::aws::AmazonBuilder;
    ///
    /// let s3 = AmazonBuilder::from_env()
    ///     .build();
    /// ```
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

    /// Parse available connection info form a well-known storage URL.
    ///
    /// The supported url schemes are:
    ///
    /// - `s3://<bucket>/<path>`
    /// - `s3a://<bucket>/<path>`
    /// - `https://s3.<region>.amazonaws.com/<bucket>`
    /// - `https://<bucket>.s3.<region>.amazonaws.com`
    /// - `https://ACCOUNT_ID.r2.cloudflarestorage.com/bucket`
    ///
    /// Note: Settings derived from the URL will override any others set on this builder
    ///
    /// # Example
    /// ```
    /// use cloud_client::aws::AmazonBuilder;
    ///
    /// let s3 = AmazonBuilder::from_env()
    ///     .with_url("s3://bucket/path")
    ///     .build();
    /// ```
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
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
            AmazonS3ConfigKey::UnsignedPayload => self.unsigned_payload.parse(value),
            AmazonS3ConfigKey::Checksum => {
                self.checksum_algorithm = Some(ConfigValue::Deferred(value.into()))
            }
            AmazonS3ConfigKey::ContainerCredentialsRelativeUri => {
                self.container_credentials_relative_uri = Some(value.into())
            }
            AmazonS3ConfigKey::Client(key) => {
                self.client_options = self.client_options.with_config(key, value)
            }
            AmazonS3ConfigKey::SkipSignature => self.skip_signature.parse(value),
            AmazonS3ConfigKey::DisableTagging => self.disable_tagging.parse(value),
            AmazonS3ConfigKey::RequestPayer => {
                self.request_payer = ConfigValue::Deferred(value.into())
            }
            AmazonS3ConfigKey::Encryption(key) => match key {
                S3EncryptionConfigKey::ServerSideEncryption => {
                    self.encryption_type = Some(ConfigValue::Deferred(value.into()))
                }
                S3EncryptionConfigKey::KmsKeyId => self.encryption_kms_key_id = Some(value.into()),
                S3EncryptionConfigKey::BucketKeyEnabled => {
                    self.encryption_bucket_key_enabled = Some(ConfigValue::Deferred(value.into()))
                }
                S3EncryptionConfigKey::CustomerEncryptionKey => {
                    self.encryption_customer_key_base64 = Some(value.into())
                }
            },
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
            AmazonS3ConfigKey::UnsignedPayload => Some(self.unsigned_payload.to_string()),
            AmazonS3ConfigKey::Checksum => {
                self.checksum_algorithm.as_ref().map(ToString::to_string)
            }
            AmazonS3ConfigKey::Client(key) => self.client_options.get_config_value(key),
            AmazonS3ConfigKey::ContainerCredentialsRelativeUri => {
                self.container_credentials_relative_uri.clone()
            }
            AmazonS3ConfigKey::SkipSignature => Some(self.skip_signature.to_string()),
            AmazonS3ConfigKey::DisableTagging => Some(self.disable_tagging.to_string()),
            AmazonS3ConfigKey::RequestPayer => Some(self.request_payer.to_string()),
            AmazonS3ConfigKey::Encryption(key) => match key {
                S3EncryptionConfigKey::ServerSideEncryption => {
                    self.encryption_type.as_ref().map(ToString::to_string)
                }
                S3EncryptionConfigKey::KmsKeyId => self.encryption_kms_key_id.clone(),
                S3EncryptionConfigKey::BucketKeyEnabled => self
                    .encryption_bucket_key_enabled
                    .as_ref()
                    .map(ToString::to_string),
                S3EncryptionConfigKey::CustomerEncryptionKey => {
                    self.encryption_customer_key_base64.clone()
                }
            },
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
    /// This option has no effect if not using instance credentials
    ///
    /// [IMDSv2]: https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/configuring-instance-metadata-service.html
    /// [SSRF attack]: https://aws.amazon.com/blogs/security/defense-in-depth-open-firewalls-reverse-proxies-ssrf-vulnerabilities-ec2-instance-metadata-service/
    ///
    pub fn with_imdsv1_fallback(mut self) -> Self {
        self.imdsv1_fallback = true.into();
        self
    }

    /// Sets if unsigned payload option has to be used.
    /// See [unsigned payload option](https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-header-based-auth.html)
    /// * false (default): Signed payload option is used, where the checksum for the request body is computed and included when constructing a canonical request.
    /// * true: Unsigned payload option is used. `UNSIGNED-PAYLOAD` literal is included when constructing a canonical request,
    pub fn with_unsigned_payload(mut self, unsigned_payload: bool) -> Self {
        self.unsigned_payload = unsigned_payload.into();
        self
    }

    /// If enabled, [`AmazonS3`] will not fetch credentials and will not sign requests
    ///
    /// This can be useful when interacting with public S3 buckets that deny authorized requests
    pub fn with_skip_signature(mut self, skip_signature: bool) -> Self {
        self.skip_signature = skip_signature.into();
        self
    }

    /// Sets the [checksum algorithm] which has to be used for object integrity check during upload.
    ///
    /// [checksum algorithm]: https://docs.aws.amazon.com/AmazonS3/latest/userguide/checking-object-integrity.html
    pub fn with_checksum_algorithm(mut self, checksum_algorithm: Checksum) -> Self {
        // Convert to String to enable deferred parsing of config
        self.checksum_algorithm = Some(checksum_algorithm.into());
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

    /// If set to `true` will ignore any tags provided to put_opts
    pub fn with_disable_tagging(mut self, ignore: bool) -> Self {
        self.disable_tagging = ignore.into();
        self
    }

    /// Use SSE-KMS for server side encryption.
    pub fn with_sse_kms_encryption(mut self, kms_key_id: impl Into<String>) -> Self {
        self.encryption_type = Some(ConfigValue::Parsed(S3EncryptionType::SseKms));
        if let Some(kms_key_id) = kms_key_id.into().into() {
            self.encryption_kms_key_id = Some(kms_key_id);
        }
        self
    }

    /// Use dual server side encryption for server side encryption.
    pub fn with_dsse_kms_encryption(mut self, kms_key_id: impl Into<String>) -> Self {
        self.encryption_type = Some(ConfigValue::Parsed(S3EncryptionType::DsseKms));
        if let Some(kms_key_id) = kms_key_id.into().into() {
            self.encryption_kms_key_id = Some(kms_key_id);
        }
        self
    }

    /// Use SSE-C for server side encryption.
    /// Must pass the *base64-encoded* 256-bit customer encryption key.
    pub fn with_ssec_encryption(mut self, customer_key_base64: impl Into<String>) -> Self {
        self.encryption_type = Some(ConfigValue::Parsed(S3EncryptionType::SseC));
        self.encryption_customer_key_base64 = customer_key_base64.into().into();
        self
    }

    /// Set whether to enable bucket key for server side encryption. This overrides
    /// the bucket default setting for bucket keys.
    ///
    /// When bucket keys are disabled, each object is encrypted with a unique data key.
    /// When bucket keys are enabled, a single data key is used for the entire bucket,
    /// reducing overhead of encryption.
    pub fn with_bucket_key(mut self, enabled: bool) -> Self {
        self.encryption_bucket_key_enabled = Some(ConfigValue::Parsed(enabled));
        self
    }

    /// Set whether to charge requester for bucket operations.
    ///
    /// <https://docs.aws.amazon.com/AmazonS3/latest/userguide/RequesterPaysBuckets.html>
    pub fn with_request_payer(mut self, enabled: bool) -> Self {
        self.request_payer = ConfigValue::Parsed(enabled);
        self
    }

    /// Create a [`AmazonS3`] instance from the provided values,
    /// consuming `self`.
    pub fn build(self) -> Result<AmazonConfig> {
        let region = self.region.unwrap_or_else(|| "us-east-1".to_string());
        let checksum = self.checksum_algorithm.map(|x| x.get()).transpose()?;

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
            // TODO: Replace with `AmazonBuilder::credentials_from_env`
            info!("Using WebIdentity credential provider");

            let session_name = std::env::var("AWS_ROLE_SESSION_NAME")
                .unwrap_or_else(|_| "WebIdentitySession".to_string());

            let endpoint = format!("https://sts.{region}.amazonaws.com");

            // Disallow non-HTTPs requests
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

            Arc::new(TokenCredentialProvider::new(
                token,
                client,
                self.retry_config.clone(),
            )) as _
        } else if let Some(uri) = self.container_credentials_relative_uri {
            info!("Using Task credential provider");
            Arc::new(TaskCredentialProvider {
                url: format!("http://169.254.170.2{uri}"),
                retry: self.retry_config.clone(),
                // The instance metadata endpoint is access over HTTP
                client: self.client_options.clone().with_allow_http(true).client()?,
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

            Arc::new(TokenCredentialProvider::new(
                token,
                self.client_options.metadata_client()?,
                self.retry_config.clone(),
            )) as _
        };

        let encryption_headers = if let Some(encryption_type) = self.encryption_type {
            S3EncryptionHeaders::try_new(
                &encryption_type.get()?,
                self.encryption_kms_key_id,
                self.encryption_bucket_key_enabled
                    .map(|val| val.get())
                    .transpose()?,
                self.encryption_customer_key_base64,
            )?
        } else {
            S3EncryptionHeaders::default()
        };

        Ok(AmazonConfig {
            region,
            credentials,
            session_provider: None,
            retry_config: self.retry_config,
            client_options: self.client_options,
            sign_payload: !self.unsigned_payload.get()?,
            skip_signature: self.skip_signature.get()?,
            disable_tagging: self.disable_tagging.get()?,
            checksum,
            encryption_headers,
            request_payer: self.request_payer.get()?,
        })
    }
}

/// Encryption configuration options for S3.
///
/// These options are used to configure server-side encryption for S3 objects.
/// To configure them, pass them to [`AmazonBuilder::with_config`].
///
/// [SSE-S3]: https://docs.aws.amazon.com/AmazonS3/latest/userguide/UsingServerSideEncryption.html
/// [SSE-KMS]: https://docs.aws.amazon.com/AmazonS3/latest/userguide/UsingKMSEncryption.html
/// [DSSE-KMS]: https://docs.aws.amazon.com/AmazonS3/latest/userguide/UsingDSSEncryption.html
/// [SSE-C]: https://docs.aws.amazon.com/AmazonS3/latest/userguide/ServerSideEncryptionCustomerKeys.html
#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy, Serialize, Deserialize)]
#[non_exhaustive]
pub enum S3EncryptionConfigKey {
    /// Type of encryption to use. If set, must be one of "AES256" (SSE-S3), "aws:kms" (SSE-KMS), "aws:kms:dsse" (DSSE-KMS) or "sse-c".
    ServerSideEncryption,
    /// The KMS key ID to use for server-side encryption. If set, ServerSideEncryption
    /// must be "aws:kms" or "aws:kms:dsse".
    KmsKeyId,
    /// If set to true, will use the bucket's default KMS key for server-side encryption.
    /// If set to false, will disable the use of the bucket's default KMS key for server-side encryption.
    BucketKeyEnabled,

    /// The base64 encoded, 256-bit customer encryption key to use for server-side encryption.
    /// If set, ServerSideEncryption must be "sse-c".
    CustomerEncryptionKey,
}

impl AsRef<str> for S3EncryptionConfigKey {
    fn as_ref(&self) -> &str {
        match self {
            Self::ServerSideEncryption => "aws_server_side_encryption",
            Self::KmsKeyId => "aws_sse_kms_key_id",
            Self::BucketKeyEnabled => "aws_sse_bucket_key_enabled",
            Self::CustomerEncryptionKey => "aws_sse_customer_key_base64",
        }
    }
}

#[derive(Debug, Clone)]
enum S3EncryptionType {
    S3,
    SseKms,
    DsseKms,
    SseC,
}

impl crate::config::Parse for S3EncryptionType {
    fn parse(s: &str) -> Result<Self> {
        match s {
            "AES256" => Ok(Self::S3),
            "aws:kms" => Ok(Self::SseKms),
            "aws:kms:dsse" => Ok(Self::DsseKms),
            "sse-c" => Ok(Self::SseC),
            _ => Err(Error::InvalidEncryptionType { passed: s.into() }.into()),
        }
    }
}

impl From<&S3EncryptionType> for &'static str {
    fn from(value: &S3EncryptionType) -> Self {
        match value {
            S3EncryptionType::S3 => "AES256",
            S3EncryptionType::SseKms => "aws:kms",
            S3EncryptionType::DsseKms => "aws:kms:dsse",
            S3EncryptionType::SseC => "sse-c",
        }
    }
}

impl std::fmt::Display for S3EncryptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.into())
    }
}

/// A sequence of headers to be sent for write requests that specify server-side
/// encryption.
///
/// Whether these headers are sent depends on both the kind of encryption set
/// and the kind of request being made.
#[derive(Default, Clone, Debug)]
pub(super) struct S3EncryptionHeaders(pub HeaderMap);

impl S3EncryptionHeaders {
    fn try_new(
        encryption_type: &S3EncryptionType,
        encryption_kms_key_id: Option<String>,
        bucket_key_enabled: Option<bool>,
        encryption_customer_key_base64: Option<String>,
    ) -> Result<Self> {
        let mut headers = HeaderMap::new();
        match encryption_type {
            S3EncryptionType::S3 | S3EncryptionType::SseKms | S3EncryptionType::DsseKms => {
                headers.insert(
                    "x-amz-server-side-encryption",
                    HeaderValue::from_static(encryption_type.into()),
                );
                if let Some(key_id) = encryption_kms_key_id {
                    headers.insert(
                        "x-amz-server-side-encryption-aws-kms-key-id",
                        key_id
                            .try_into()
                            .map_err(|err| Error::InvalidEncryptionHeader {
                                header: "kms-key-id",
                                source: Box::new(err),
                            })?,
                    );
                }
                if let Some(bucket_key_enabled) = bucket_key_enabled {
                    headers.insert(
                        "x-amz-server-side-encryption-bucket-key-enabled",
                        HeaderValue::from_static(if bucket_key_enabled { "true" } else { "false" }),
                    );
                }
            }
            S3EncryptionType::SseC => {
                headers.insert(
                    "x-amz-server-side-encryption-customer-algorithm",
                    HeaderValue::from_static("AES256"),
                );
                if let Some(key) = encryption_customer_key_base64 {
                    let mut header_value: HeaderValue =
                        key.clone()
                            .try_into()
                            .map_err(|err| Error::InvalidEncryptionHeader {
                                header: "x-amz-server-side-encryption-customer-key",
                                source: Box::new(err),
                            })?;
                    header_value.set_sensitive(true);
                    headers.insert("x-amz-server-side-encryption-customer-key", header_value);

                    let decoded_key = BASE64_STANDARD.decode(key.as_bytes()).map_err(|err| {
                        Error::InvalidEncryptionHeader {
                            header: "x-amz-server-side-encryption-customer-key",
                            source: Box::new(err),
                        }
                    })?;
                    let mut hasher = Md5::new();
                    hasher.update(decoded_key);
                    let md5 = BASE64_STANDARD.encode(hasher.finalize());
                    let mut md5_header_value: HeaderValue =
                        md5.try_into()
                            .map_err(|err| Error::InvalidEncryptionHeader {
                                header: "x-amz-server-side-encryption-customer-key-MD5",
                                source: Box::new(err),
                            })?;
                    md5_header_value.set_sensitive(true);
                    headers.insert(
                        "x-amz-server-side-encryption-customer-key-MD5",
                        md5_header_value,
                    );
                } else {
                    return Err(Error::InvalidEncryptionHeader {
                        header: "x-amz-server-side-encryption-customer-key",
                        source: Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Missing customer key",
                        )),
                    }
                    .into());
                }
            }
        }
        Ok(Self(headers))
    }
}

impl From<S3EncryptionHeaders> for HeaderMap {
    fn from(headers: S3EncryptionHeaders) -> Self {
        headers.0
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
            ("aws_unsigned_payload", "true".to_string()),
            ("aws_checksum_algorithm", "sha256".to_string()),
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
        assert_eq!(
            builder.checksum_algorithm.unwrap().get().unwrap(),
            Checksum::SHA256
        );
        assert!(builder.unsigned_payload.get().unwrap());
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
            .with_config(AmazonS3ConfigKey::Token, &aws_session_token)
            .with_config(AmazonS3ConfigKey::UnsignedPayload, "true")
            .with_config("aws_server_side_encryption".parse().unwrap(), "AES256")
            .with_config("aws_sse_kms_key_id".parse().unwrap(), "some_key_id")
            .with_config("aws_sse_bucket_key_enabled".parse().unwrap(), "true")
            .with_config(
                "aws_sse_customer_key_base64".parse().unwrap(),
                "some_customer_key",
            );

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
        assert_eq!(
            builder
                .get_config_value(&AmazonS3ConfigKey::UnsignedPayload)
                .unwrap(),
            "true"
        );
        assert_eq!(
            builder
                .get_config_value(&"aws_server_side_encryption".parse().unwrap())
                .unwrap(),
            "AES256"
        );
        assert_eq!(
            builder
                .get_config_value(&"aws_sse_kms_key_id".parse().unwrap())
                .unwrap(),
            "some_key_id"
        );
        assert_eq!(
            builder
                .get_config_value(&"aws_sse_bucket_key_enabled".parse().unwrap())
                .unwrap(),
            "true"
        );
        assert_eq!(
            builder
                .get_config_value(&"aws_sse_customer_key_base64".parse().unwrap())
                .unwrap(),
            "some_customer_key"
        );
    }

    #[test]
    fn s3_default_region() {
        let config = AmazonBuilder::new().build().unwrap();
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
            .build();

        assert!(s3.is_ok());

        //let err = AmazonBuilder::new()
        //    .with_access_key_id("access_key_id")
        //    .with_secret_access_key("secret_access_key")
        //    .with_region("region")
        //    .with_allow_http(true)
        //    .with_proxy_url("asdf://example.com")
        //    .build()
        //    .unwrap_err()
        //    .to_string();

        //assert_eq!("Generic HTTP client error: builder error", err);
    }

    #[test]
    fn test_invalid_config() {
        let err = AmazonBuilder::new()
            .with_config(AmazonS3ConfigKey::ImdsV1Fallback, "enabled")
            .with_region("region")
            .build()
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
