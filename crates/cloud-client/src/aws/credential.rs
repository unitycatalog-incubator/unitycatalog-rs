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

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use bytes::Buf;
use chrono::{DateTime, Utc};
use percent_encoding::utf8_percent_encode;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::{Client, Method, Request, RequestBuilder, StatusCode};
use serde::Deserialize;
use tracing::warn;
use url::Url;

use crate::retry::RetryExt;
use crate::service::HttpService;
use crate::token::{TemporaryToken, TokenCache};
use crate::util::{hex_digest, hex_encode, hmac_sha256};
use crate::{CredentialProvider, Result, RetryConfig, TokenProvider};

use crate::util::STRICT_ENCODE_SET;

/// This is used to maintain the URI path encoding
const STRICT_PATH_ENCODE_SET: percent_encoding::AsciiSet = STRICT_ENCODE_SET.remove(b'/');

type StdError = Box<dyn std::error::Error + Send + Sync>;

/// SHA256 hash of empty string
static EMPTY_SHA256_HASH: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

/// A set of AWS security credentials
#[derive(Debug, Eq, PartialEq)]
pub struct AwsCredential {
    /// AWS_ACCESS_KEY_ID
    pub key_id: String,
    /// AWS_SECRET_ACCESS_KEY
    pub secret_key: String,
    /// AWS_SESSION_TOKEN
    pub token: Option<String>,
}

impl AwsCredential {
    /// Signs a string
    ///
    /// <https://docs.aws.amazon.com/general/latest/gr/sigv4-calculate-signature.html>
    fn sign(&self, to_sign: &str, date: DateTime<Utc>, region: &str, service: &str) -> String {
        let date_string = date.format("%Y%m%d").to_string();
        let date_hmac = hmac_sha256(format!("AWS4{}", self.secret_key), date_string);
        let region_hmac = hmac_sha256(date_hmac, region);
        let service_hmac = hmac_sha256(region_hmac, service);
        let signing_hmac = hmac_sha256(service_hmac, b"aws4_request");
        hex_encode(hmac_sha256(signing_hmac, to_sign).as_ref())
    }
}

/// Authorize a [`Request`] with an [`AwsCredential`] using [AWS SigV4]
///
/// [AWS SigV4]: https://docs.aws.amazon.com/general/latest/gr/sigv4-calculate-signature.html
#[derive(Debug)]
pub struct AwsAuthorizer<'a> {
    date: Option<DateTime<Utc>>,
    credential: &'a AwsCredential,
    service: &'a str,
    region: &'a str,
}

static DATE_HEADER: hyper::header::HeaderName =
    hyper::header::HeaderName::from_static("x-amz-date");
static HASH_HEADER: hyper::header::HeaderName =
    hyper::header::HeaderName::from_static("x-amz-content-sha256");
static TOKEN_HEADER: hyper::header::HeaderName =
    hyper::header::HeaderName::from_static("x-amz-security-token");
const ALGORITHM: &str = "AWS4-HMAC-SHA256";

impl<'a> AwsAuthorizer<'a> {
    /// Create a new [`AwsAuthorizer`]
    pub fn new(credential: &'a AwsCredential, service: &'a str, region: &'a str) -> Self {
        Self {
            credential,
            service,
            region,
            date: None,
        }
    }

    /// Authorize `request` with an optional pre-calculated SHA256 digest by attaching
    /// the relevant [AWS SigV4] headers
    ///
    /// [AWS SigV4]: https://docs.aws.amazon.com/IAM/latest/UserGuide/create-signed-request.html
    pub fn authorize(
        &self,
        request: &mut Request,
        pre_calculated_digest: Option<&[u8]>,
    ) -> crate::Result<()> {
        if let Some(ref token) = self.credential.token {
            let token_val = HeaderValue::from_str(token)?;
            request.headers_mut().insert(&TOKEN_HEADER, token_val);
        }

        let host = &request.url()[url::Position::BeforeHost..url::Position::AfterPort];
        let host_val = HeaderValue::from_str(host)?;
        request.headers_mut().insert("host", host_val);

        let date = self.date.unwrap_or_else(Utc::now);
        let date_str = date.format("%Y%m%dT%H%M%SZ").to_string();
        let date_val = HeaderValue::from_str(&date_str)?;
        request.headers_mut().insert(&DATE_HEADER, date_val);

        let digest = match pre_calculated_digest {
            Some(digest) => hex_encode(digest),
            None => match request.body() {
                None => EMPTY_SHA256_HASH.to_string(),
                Some(body) => match body.as_bytes() {
                    Some(bytes) => hex_digest(bytes),
                    None => EMPTY_SHA256_HASH.to_string(),
                },
            },
        };

        let header_digest = HeaderValue::from_str(&digest)?;
        request.headers_mut().insert(&HASH_HEADER, header_digest);

        let (signed_headers, canonical_headers) = canonicalize_headers(request.headers());

        let scope = self.scope(date);

        let string_to_sign = self.string_to_sign(
            date,
            &scope,
            request.method(),
            request.url(),
            &canonical_headers,
            &signed_headers,
            &digest,
        );

        let signature = self
            .credential
            .sign(&string_to_sign, date, self.region, self.service);

        let authorisation = format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            ALGORITHM, self.credential.key_id, scope, signed_headers, signature
        );

        let authorization_val = HeaderValue::from_str(&authorisation)?;
        request
            .headers_mut()
            .insert(&AUTHORIZATION, authorization_val);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn string_to_sign(
        &self,
        date: DateTime<Utc>,
        scope: &str,
        request_method: &Method,
        url: &Url,
        canonical_headers: &str,
        signed_headers: &str,
        digest: &str,
    ) -> String {
        // Each path segment must be URI-encoded twice (except for Amazon S3 which only gets
        // URI-encoded once).
        // see https://docs.aws.amazon.com/general/latest/gr/sigv4-create-canonical-request.html
        let canonical_uri = match self.service {
            "s3" => url.path().to_string(),
            _ => utf8_percent_encode(url.path(), &STRICT_PATH_ENCODE_SET).to_string(),
        };

        let canonical_query = canonicalize_query(url);

        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            request_method.as_str(),
            canonical_uri,
            canonical_query,
            canonical_headers,
            signed_headers,
            digest
        );

        let hashed_canonical_request = hex_digest(canonical_request.as_bytes());

        format!(
            "{}\n{}\n{}\n{}",
            ALGORITHM,
            date.format("%Y%m%dT%H%M%SZ"),
            scope,
            hashed_canonical_request
        )
    }

    fn scope(&self, date: DateTime<Utc>) -> String {
        format!(
            "{}/{}/{}/aws4_request",
            date.format("%Y%m%d"),
            self.region,
            self.service
        )
    }
}

pub(crate) trait CredentialExt {
    /// Sign a request <https://docs.aws.amazon.com/general/latest/gr/sigv4_signing.html>
    fn with_aws_sigv4(
        self,
        authorizer: Option<AwsAuthorizer<'_>>,
        payload_sha256: Option<&[u8]>,
    ) -> Self;
}

impl CredentialExt for RequestBuilder {
    fn with_aws_sigv4(
        self,
        authorizer: Option<AwsAuthorizer<'_>>,
        payload_sha256: Option<&[u8]>,
    ) -> Self {
        match authorizer {
            Some(authorizer) => {
                let (client, request) = self.build_split();
                let mut request = request.expect("request valid");
                authorizer
                    .authorize(&mut request, payload_sha256)
                    .expect("credential values must be valid header characters");

                Self::from_parts(client, request)
            }
            None => self,
        }
    }
}

/// Canonicalizes query parameters into the AWS canonical form
///
/// <https://docs.aws.amazon.com/general/latest/gr/sigv4-create-canonical-request.html>
fn canonicalize_query(url: &Url) -> String {
    use std::fmt::Write;

    let capacity = match url.query() {
        Some(q) if !q.is_empty() => q.len(),
        _ => return String::new(),
    };
    let mut encoded = String::with_capacity(capacity + 1);

    let mut headers = url.query_pairs().collect::<Vec<_>>();
    headers.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

    let mut first = true;
    for (k, v) in headers {
        if !first {
            encoded.push('&');
        }
        first = false;
        let _ = write!(
            encoded,
            "{}={}",
            utf8_percent_encode(k.as_ref(), &STRICT_ENCODE_SET),
            utf8_percent_encode(v.as_ref(), &STRICT_ENCODE_SET)
        );
    }
    encoded
}

/// Canonicalizes headers into the AWS Canonical Form.
///
/// <https://docs.aws.amazon.com/general/latest/gr/sigv4-create-canonical-request.html>
fn canonicalize_headers(header_map: &HeaderMap) -> (String, String) {
    // Use owned Strings for values since header bytes may not be valid UTF-8;
    // we convert using lossy UTF-8 to avoid panicking on unusual header values.
    let mut headers = BTreeMap::<&str, Vec<String>>::new();
    let mut value_count = 0;
    let mut value_bytes = 0;
    let mut key_bytes = 0;

    for (key, value) in header_map {
        let key = key.as_str();
        if ["authorization", "content-length", "user-agent"].contains(&key) {
            continue;
        }

        let value = String::from_utf8_lossy(value.as_bytes()).into_owned();
        key_bytes += key.len();
        value_bytes += value.len();
        value_count += 1;
        headers.entry(key).or_default().push(value);
    }

    let mut signed_headers = String::with_capacity(key_bytes + headers.len());
    let mut canonical_headers =
        String::with_capacity(key_bytes + value_bytes + headers.len() + value_count);

    for (header_idx, (name, values)) in headers.into_iter().enumerate() {
        if header_idx != 0 {
            signed_headers.push(';');
        }

        signed_headers.push_str(name);
        canonical_headers.push_str(name);
        canonical_headers.push(':');
        for (value_idx, value) in values.into_iter().enumerate() {
            if value_idx != 0 {
                canonical_headers.push(',');
            }
            canonical_headers.push_str(value.trim());
        }
        canonical_headers.push('\n');
    }

    (signed_headers, canonical_headers)
}

/// Credentials sourced from the instance metadata service
///
/// Fetches short-lived `AwsCredential` values from the EC2 Instance Metadata
/// Service (IMDS).  Supports both IMDSv2 (token-protected) and IMDSv1 as a
/// fallback when `imdsv1_fallback` is set.
///
/// # References
/// - <https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-instance-metadata.html>
/// - <https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/configuring-instance-metadata-service.html>
#[derive(Debug)]
pub(crate) struct InstanceCredentialProvider {
    pub imdsv1_fallback: bool,
    pub metadata_endpoint: String,
}

#[async_trait]
impl TokenProvider for InstanceCredentialProvider {
    type Credential = AwsCredential;

    async fn fetch_token(
        &self,
        client: &Client,
        service: &Arc<dyn HttpService>,
        retry: &RetryConfig,
    ) -> Result<TemporaryToken<Arc<AwsCredential>>> {
        instance_creds(
            client,
            service,
            retry,
            &self.metadata_endpoint,
            self.imdsv1_fallback,
        )
        .await
        .map_err(|source| crate::Error::Generic { source })
    }
}

/// Credentials sourced using `AssumeRoleWithWebIdentity`.
///
/// Exchanges an OIDC token (e.g. a Kubernetes projected service-account token)
/// for temporary AWS credentials by calling the STS
/// `AssumeRoleWithWebIdentity` API.  Commonly used in EKS pod identity
/// scenarios where the token file path is supplied via the
/// `AWS_WEB_IDENTITY_TOKEN_FILE` environment variable.
///
/// # References
/// - <https://docs.aws.amazon.com/STS/latest/APIReference/API_AssumeRoleWithWebIdentity.html>
/// - <https://docs.aws.amazon.com/eks/latest/userguide/iam-roles-for-service-accounts-technical-overview.html>
#[derive(Debug)]
pub(crate) struct WebIdentityProvider {
    pub token_path: String,
    pub role_arn: String,
    pub session_name: String,
    pub endpoint: String,
}

#[async_trait]
impl TokenProvider for WebIdentityProvider {
    type Credential = AwsCredential;

    async fn fetch_token(
        &self,
        client: &Client,
        service: &Arc<dyn HttpService>,
        retry: &RetryConfig,
    ) -> Result<TemporaryToken<Arc<AwsCredential>>> {
        web_identity(
            client,
            service,
            retry,
            &self.token_path,
            &self.role_arn,
            &self.session_name,
            &self.endpoint,
        )
        .await
        .map_err(|source| crate::Error::Generic { source })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct InstanceCredentials {
    access_key_id: String,
    secret_access_key: String,
    token: String,
    expiration: DateTime<Utc>,
}

impl From<InstanceCredentials> for AwsCredential {
    fn from(s: InstanceCredentials) -> Self {
        Self {
            key_id: s.access_key_id,
            secret_key: s.secret_access_key,
            token: Some(s.token),
        }
    }
}

/// <https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/iam-roles-for-amazon-ec2.html#instance-metadata-security-credentials>
async fn instance_creds(
    client: &Client,
    service: &Arc<dyn HttpService>,
    retry_config: &RetryConfig,
    endpoint: &str,
    imdsv1_fallback: bool,
) -> Result<TemporaryToken<Arc<AwsCredential>>, StdError> {
    const CREDENTIALS_PATH: &str = "latest/meta-data/iam/security-credentials";
    const AWS_EC2_METADATA_TOKEN_HEADER: &str = "X-aws-ec2-metadata-token";

    let token_url = format!("{endpoint}/latest/api/token");

    let token_result = client
        .request(Method::PUT, token_url)
        .header("X-aws-ec2-metadata-token-ttl-seconds", "600") // 10 minute TTL
        .retryable(retry_config, service.clone())
        .idempotent(true)
        .send()
        .await;

    let token = match token_result {
        Ok(t) => Some(t.text().await?),
        Err(e) if imdsv1_fallback && matches!(e.status(), Some(StatusCode::FORBIDDEN)) => {
            warn!("received 403 from metadata endpoint, falling back to IMDSv1");
            None
        }
        Err(e) => return Err(e.into()),
    };

    let role_url = format!("{endpoint}/{CREDENTIALS_PATH}/");
    let mut role_request = client.request(Method::GET, role_url);

    if let Some(token) = &token {
        role_request = role_request.header(AWS_EC2_METADATA_TOKEN_HEADER, token);
    }

    let role = role_request
        .send_retry(retry_config, service.clone())
        .await?
        .text()
        .await?;

    let creds_url = format!("{endpoint}/{CREDENTIALS_PATH}/{role}");
    let mut creds_request = client.request(Method::GET, creds_url);
    if let Some(token) = &token {
        creds_request = creds_request.header(AWS_EC2_METADATA_TOKEN_HEADER, token);
    }

    let creds: InstanceCredentials = creds_request
        .send_retry(retry_config, service.clone())
        .await?
        .json()
        .await?;

    let now = Utc::now();
    let ttl = (creds.expiration - now).to_std().unwrap_or_default();
    Ok(TemporaryToken {
        token: Arc::new(creds.into()),
        expiry: Some(Instant::now() + ttl),
    })
}

/// Exchanges long-term or instance credentials for temporary credentials by
/// calling the STS [`AssumeRole`] API.  The caller's base credentials are used
/// to SigV4-sign the STS request.
///
/// Wire this provider via [`AmazonBuilder::with_role_arn`] or set the
/// `AWS_ROLE_ARN` / `AWS_ROLE_SESSION_NAME` environment variables alongside a
/// static or instance credential source.
///
/// # References
/// - <https://docs.aws.amazon.com/STS/latest/APIReference/API_AssumeRole.html>
/// - <https://docs.aws.amazon.com/IAM/latest/UserGuide/id_roles_use_switch-role-api.html>
#[derive(Debug)]
pub(crate) struct AssumeRoleProvider {
    pub role_arn: String,
    pub session_name: String,
    pub endpoint: String,
    pub base_credentials: Arc<dyn CredentialProvider<Credential = AwsCredential>>,
    pub region: String,
    /// Optional inline session policy (URL-encoded JSON).
    /// Intersected with the role's own policy to further restrict access.
    pub policy: Option<String>,
}

#[async_trait]
impl TokenProvider for AssumeRoleProvider {
    type Credential = AwsCredential;

    async fn fetch_token(
        &self,
        client: &Client,
        service: &Arc<dyn HttpService>,
        retry: &RetryConfig,
    ) -> Result<TemporaryToken<Arc<AwsCredential>>> {
        let base_cred = self
            .base_credentials
            .get_credential()
            .await
            .map_err(|source| crate::Error::Generic {
                source: Box::new(source),
            })?;

        assume_role(
            client,
            service,
            retry,
            &base_cred,
            &self.region,
            &self.role_arn,
            &self.session_name,
            &self.endpoint,
            self.policy.as_deref(),
        )
        .await
        .map_err(|source| crate::Error::Generic { source })
    }
}

/// Calls `STS:AssumeRole` and returns temporary credentials.
///
/// The request is SigV4-signed using the provided `base_cred`.
///
/// # References
/// - <https://docs.aws.amazon.com/STS/latest/APIReference/API_AssumeRole.html>
async fn assume_role(
    client: &Client,
    service: &Arc<dyn HttpService>,
    retry_config: &RetryConfig,
    base_cred: &AwsCredential,
    region: &str,
    role_arn: &str,
    session_name: &str,
    endpoint: &str,
    policy: Option<&str>,
) -> Result<TemporaryToken<Arc<AwsCredential>>, StdError> {
    let mut query: Vec<(&str, &str)> = vec![
        ("Action", "AssumeRole"),
        ("DurationSeconds", "3600"),
        ("RoleArn", role_arn),
        ("RoleSessionName", session_name),
        ("Version", "2011-06-15"),
    ];
    if let Some(p) = policy {
        query.push(("Policy", p));
    }
    let bytes = client
        .request(Method::POST, endpoint)
        .query(&query)
        .with_aws_sigv4(Some(AwsAuthorizer::new(base_cred, "sts", region)), None)
        .retryable(retry_config, service.clone())
        .idempotent(true)
        .send()
        .await?
        .bytes()
        .await?;

    let resp: AssumeRoleXmlResponse = quick_xml::de::from_reader(bytes.reader())
        .map_err(|e| format!("Invalid AssumeRole response: {e}"))?;

    let creds = resp.assume_role_result.credentials;
    let now = Utc::now();
    let ttl = (creds.expiration - now).to_std().unwrap_or_default();

    Ok(TemporaryToken {
        token: Arc::new(creds.into()),
        expiry: Some(Instant::now() + ttl),
    })
}

/// XML response for `AssumeRole`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AssumeRoleXmlResponse {
    assume_role_result: AssumeRoleResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AssumeRoleResponse {
    assume_role_with_web_identity_result: AssumeRoleResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AssumeRoleResult {
    credentials: SessionCredentials,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SessionCredentials {
    session_token: String,
    secret_access_key: String,
    access_key_id: String,
    expiration: DateTime<Utc>,
}

impl From<SessionCredentials> for AwsCredential {
    fn from(s: SessionCredentials) -> Self {
        Self {
            key_id: s.access_key_id,
            secret_key: s.secret_access_key,
            token: Some(s.session_token),
        }
    }
}

/// <https://docs.aws.amazon.com/eks/latest/userguide/iam-roles-for-service-accounts-technical-overview.html>
async fn web_identity(
    client: &Client,
    service: &Arc<dyn HttpService>,
    retry_config: &RetryConfig,
    token_path: &str,
    role_arn: &str,
    session_name: &str,
    endpoint: &str,
) -> Result<TemporaryToken<Arc<AwsCredential>>, StdError> {
    let token = std::fs::read_to_string(token_path)
        .map_err(|e| format!("Failed to read token file '{token_path}': {e}"))?;

    let bytes = client
        .request(Method::POST, endpoint)
        .query(&[
            ("Action", "AssumeRoleWithWebIdentity"),
            ("DurationSeconds", "3600"),
            ("RoleArn", role_arn),
            ("RoleSessionName", session_name),
            ("Version", "2011-06-15"),
            ("WebIdentityToken", &token),
        ])
        .retryable(retry_config, service.clone())
        .idempotent(true)
        .sensitive(true)
        .send()
        .await?
        .bytes()
        .await?;

    let resp: AssumeRoleResponse = quick_xml::de::from_reader(bytes.reader())
        .map_err(|e| format!("Invalid AssumeRoleWithWebIdentity response: {e}"))?;

    let creds = resp.assume_role_with_web_identity_result.credentials;
    let now = Utc::now();
    let ttl = (creds.expiration - now).to_std().unwrap_or_default();

    Ok(TemporaryToken {
        token: Arc::new(creds.into()),
        expiry: Some(Instant::now() + ttl),
    })
}

/// Credentials sourced from a task IAM role.
///
/// Fetches short-lived `AwsCredential` values from the ECS task-metadata
/// credential endpoint.  The URL is resolved from environment variables in
/// this priority order:
///
/// 1. `AWS_CONTAINER_CREDENTIALS_FULL_URI` — absolute URL (used by EKS Pod
///    Identity and Lambda)
/// 2. `AWS_CONTAINER_CREDENTIALS_RELATIVE_URI` — path appended to
///    `http://169.254.170.2` (classic ECS task role)
///
/// When `AWS_CONTAINER_AUTHORIZATION_TOKEN_FILE` is set the file contents are
/// sent as the `Authorization` request header, enabling EKS Pod Identity.
/// Results are cached in the embedded [`TokenCache`] until they approach expiry.
///
/// # References
/// - <https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task-iam-roles.html>
/// - <https://docs.aws.amazon.com/eks/latest/userguide/pod-id-how-it-works.html>
#[derive(Debug)]
pub(crate) struct TaskCredentialProvider {
    pub url: String,
    /// Optional authorization token sent as the `Authorization` header.
    /// Used by EKS Pod Identity (`AWS_CONTAINER_AUTHORIZATION_TOKEN_FILE`).
    pub auth_token_file: Option<String>,
    pub retry: RetryConfig,
    pub client: Client,
    pub service: Arc<dyn HttpService>,
    pub cache: TokenCache<Arc<AwsCredential>>,
}

#[async_trait]
impl CredentialProvider for TaskCredentialProvider {
    type Credential = AwsCredential;

    async fn get_credential(&self) -> Result<Arc<AwsCredential>> {
        self.cache
            .get_or_insert_with(|| {
                task_credential(
                    &self.client,
                    &self.service,
                    &self.retry,
                    &self.url,
                    self.auth_token_file.as_deref(),
                )
            })
            .await
            .map_err(|source| crate::Error::Generic { source })
    }
}

/// <https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task-iam-roles.html>
async fn task_credential(
    client: &Client,
    service: &Arc<dyn HttpService>,
    retry: &RetryConfig,
    url: &str,
    auth_token_file: Option<&str>,
) -> Result<TemporaryToken<Arc<AwsCredential>>, StdError> {
    let mut req = client.get(url);

    // EKS Pod Identity requires an Authorization header read from a file
    if let Some(token_file) = auth_token_file {
        let token = std::fs::read_to_string(token_file)
            .map_err(|e| format!("Failed to read auth token file '{token_file}': {e}"))?;
        req = req.header(reqwest::header::AUTHORIZATION, token.trim());
    }

    let creds: InstanceCredentials = req.send_retry(retry, service.clone()).await?.json().await?;

    let now = Utc::now();
    let ttl = (creds.expiration - now).to_std().unwrap_or_default();
    Ok(TemporaryToken {
        token: Arc::new(creds.into()),
        expiry: Some(Instant::now() + ttl),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::ReqwestService;
    use reqwest::{Client, Method};
    use std::env;

    #[test]
    fn test_sign_with_signed_payload() {
        let client = Client::new();

        let credential = AwsCredential {
            key_id: "AKIAIOSFODNN7EXAMPLE".to_string(), // gitleaks:allow
            secret_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
            token: None,
        };

        let date = DateTime::parse_from_rfc3339("2022-08-06T18:01:34Z")
            .unwrap()
            .with_timezone(&Utc);

        let mut request = client
            .request(Method::GET, "https://ec2.amazon.com/")
            .build()
            .unwrap();

        let signer = AwsAuthorizer {
            date: Some(date),
            credential: &credential,
            service: "ec2",
            region: "us-east-1",
        };

        signer.authorize(&mut request, None).unwrap();
        assert_eq!(
            request.headers().get(&AUTHORIZATION).unwrap(),
            "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20220806/us-east-1/ec2/aws4_request, SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature=a3c787a7ed37f7fdfbfd2d7056a3d7c9d85e6d52a2bfbec73793c0be6e7862d4"
        )
        // gitleaks:allow
    }

    #[test]
    fn test_sign_port() {
        let client = Client::new();

        let credential = AwsCredential {
            key_id: "H20ABqCkLZID4rLe".to_string(),
            secret_key: "jMqRDgxSsBqqznfmddGdu1TmmZOJQxdM".to_string(),
            token: None,
        };

        let date = DateTime::parse_from_rfc3339("2022-08-09T13:05:25Z")
            .unwrap()
            .with_timezone(&Utc);

        let mut request = client
            .request(Method::GET, "http://localhost:9000/tsm-schemas")
            .query(&[
                ("delimiter", "/"),
                ("encoding-type", "url"),
                ("list-type", "2"),
                ("prefix", ""),
            ])
            .build()
            .unwrap();

        let authorizer = AwsAuthorizer {
            date: Some(date),
            credential: &credential,
            service: "s3",
            region: "us-east-1",
        };

        authorizer.authorize(&mut request, None).unwrap();
        assert_eq!(
            request.headers().get(&AUTHORIZATION).unwrap(),
            "AWS4-HMAC-SHA256 Credential=H20ABqCkLZID4rLe/20220809/us-east-1/s3/aws4_request, SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature=9ebf2f92872066c99ac94e573b4e1b80f4dbb8a32b1e8e23178318746e7d1b4d"
        )
    }

    #[tokio::test]
    async fn test_instance_metadata() {
        if env::var("TEST_INTEGRATION").is_err() {
            eprintln!("skipping AWS integration test");
            return;
        }

        let endpoint = env::var("EC2_METADATA_ENDPOINT").unwrap();
        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry_config = RetryConfig::default();

        let resp = client
            .request(Method::GET, format!("{endpoint}/latest/meta-data/ami-id"))
            .send()
            .await
            .unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "Ensure metadata endpoint is set to only allow IMDSv2"
        );

        let creds = instance_creds(&client, &service, &retry_config, &endpoint, false)
            .await
            .unwrap();

        let id = &creds.token.key_id;
        let secret = &creds.token.secret_key;
        let token = creds.token.token.as_ref().unwrap();

        assert!(!id.is_empty());
        assert!(!secret.is_empty());
        assert!(!token.is_empty())
    }
    #[tokio::test]
    async fn test_mock() {
        let mut server = mockito::Server::new_async().await;

        const IMDSV2_HEADER: &str = "X-aws-ec2-metadata-token";

        let secret_access_key = "SECRET";
        let access_key_id = "KEYID";
        let token = "TOKEN";

        let endpoint = server.url();
        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry_config = RetryConfig::default();

        // Test IMDSv2
        let _mock1 = server
            .mock("PUT", "/latest/api/token")
            .with_status(200)
            .with_body("cupcakes")
            .create_async()
            .await;

        let _mock2 = server
            .mock("GET", "/latest/meta-data/iam/security-credentials/")
            .match_header(IMDSV2_HEADER, "cupcakes")
            .with_status(200)
            .with_body("myrole")
            .create_async()
            .await;

        let _mock3 = server
            .mock("GET", "/latest/meta-data/iam/security-credentials/myrole")
            .match_header(IMDSV2_HEADER, "cupcakes")
            .with_status(200)
            .with_body(r#"{"AccessKeyId":"KEYID","Code":"Success","Expiration":"2022-08-30T10:51:04Z","LastUpdated":"2022-08-30T10:21:04Z","SecretAccessKey":"SECRET","Token":"TOKEN","Type":"AWS-HMAC"}"#)
            .create_async()
            .await;

        let creds = instance_creds(&client, &service, &retry_config, &endpoint, true)
            .await
            .unwrap();

        assert_eq!(creds.token.token.as_deref().unwrap(), token);
        assert_eq!(&creds.token.key_id, access_key_id);
        assert_eq!(&creds.token.secret_key, secret_access_key);

        // Test IMDSv1 fallback
        let _mock4 = server
            .mock("PUT", "/latest/api/token")
            .with_status(403)
            .with_body("")
            .create_async()
            .await;

        let _mock5 = server
            .mock("GET", "/latest/meta-data/iam/security-credentials/")
            .with_status(200)
            .with_body("myrole")
            .create_async()
            .await;

        let _mock6 = server
            .mock("GET", "/latest/meta-data/iam/security-credentials/myrole")
            .with_status(200)
            .with_body(r#"{"AccessKeyId":"KEYID","Code":"Success","Expiration":"2022-08-30T10:51:04Z","LastUpdated":"2022-08-30T10:21:04Z","SecretAccessKey":"SECRET","Token":"TOKEN","Type":"AWS-HMAC"}"#)
            .create_async()
            .await;

        let creds = instance_creds(&client, &service, &retry_config, &endpoint, true)
            .await
            .unwrap();

        assert_eq!(creds.token.token.as_deref().unwrap(), token);
        assert_eq!(&creds.token.key_id, access_key_id);
        assert_eq!(&creds.token.secret_key, secret_access_key);

        // Test IMDSv1 fallback disabled
        let _mock7 = server
            .mock("PUT", "/latest/api/token")
            .with_status(403)
            .with_body("")
            .create_async()
            .await;

        // Should fail
        instance_creds(&client, &service, &retry_config, &endpoint, false)
            .await
            .unwrap_err();
    }

    const STS_WEB_IDENTITY_XML: &str = r#"<AssumeRoleWithWebIdentityResponse xmlns="https://sts.amazonaws.com/doc/2011-06-15/"><AssumeRoleWithWebIdentityResult><Credentials><AccessKeyId>AKIAIOSFODNN7EXAMPLE</AccessKeyId><SecretAccessKey>wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY</SecretAccessKey><SessionToken>FwoGZXIvYXdzEJr//////////wEaDHer8</SessionToken><Expiration>2099-01-01T00:00:00Z</Expiration></Credentials></AssumeRoleWithWebIdentityResult></AssumeRoleWithWebIdentityResponse>"#; // gitleaks:allow

    const STS_ASSUME_ROLE_XML: &str = r#"<AssumeRoleResponse xmlns="https://sts.amazonaws.com/doc/2011-06-15/"><AssumeRoleResult><Credentials><AccessKeyId>AKIAIOSFODNN7EXAMPLE</AccessKeyId><SecretAccessKey>wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY</SecretAccessKey><SessionToken>FwoGZXIvYXdzEJr//////////wEaDHer8</SessionToken><Expiration>2099-01-01T00:00:00Z</Expiration></Credentials></AssumeRoleResult></AssumeRoleResponse>"#; // gitleaks:allow

    #[tokio::test]
    async fn test_web_identity_provider() {
        use std::io::Write as _;
        let mut server = mockito::Server::new_async().await;

        // Write a fake JWT to a temp file
        let mut token_file = tempfile::NamedTempFile::new().unwrap();
        write!(token_file, "fake-jwt-token").unwrap();

        let sts_url = server.url();

        let _mock = server
            .mock("POST", "/")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("Action".into(), "AssumeRoleWithWebIdentity".into()),
                mockito::Matcher::UrlEncoded(
                    "RoleArn".into(),
                    "arn:aws:iam::123456789012:role/TestRole".into(),
                ),
                mockito::Matcher::UrlEncoded("WebIdentityToken".into(), "fake-jwt-token".into()),
            ]))
            .with_status(200)
            .with_body(STS_WEB_IDENTITY_XML)
            .create_async()
            .await;

        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry_config = RetryConfig::default();

        let provider = WebIdentityProvider {
            token_path: token_file.path().to_str().unwrap().to_owned(),
            role_arn: "arn:aws:iam::123456789012:role/TestRole".into(),
            session_name: "test-session".into(),
            endpoint: sts_url,
        };

        let creds = provider
            .fetch_token(&client, &service, &retry_config)
            .await
            .unwrap();

        assert_eq!(creds.token.key_id, "AKIAIOSFODNN7EXAMPLE"); // gitleaks:allow
        assert!(creds.token.token.is_some());
        assert!(creds.expiry.is_some());

        _mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_task_credential_provider() {
        let mut server = mockito::Server::new_async().await;

        let endpoint = server.url();

        let _mock = server
            .mock("GET", "/v2/credentials/test")
            .with_status(200)
            .with_body(r#"{"AccessKeyId":"TASKID","Code":"Success","Expiration":"2099-01-01T00:00:00Z","LastUpdated":"2022-08-30T10:21:04Z","SecretAccessKey":"TASKSECRET","Token":"TASKTOKEN","Type":"AWS-HMAC"}"#)
            .create_async()
            .await;

        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry = RetryConfig::default();

        let provider = TaskCredentialProvider {
            url: format!("{}/v2/credentials/test", endpoint),
            auth_token_file: None,
            retry: retry.clone(),
            client: client.clone(),
            service: service.clone(),
            cache: Default::default(),
        };

        let creds = provider.get_credential().await.unwrap();

        assert_eq!(creds.key_id, "TASKID");
        assert_eq!(creds.secret_key, "TASKSECRET");
        assert_eq!(creds.token.as_deref(), Some("TASKTOKEN"));

        _mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_task_credential_with_auth_token() {
        use std::io::Write as _;
        let mut server = mockito::Server::new_async().await;

        // Write auth token to a temp file
        let mut auth_file = tempfile::NamedTempFile::new().unwrap();
        write!(auth_file, "my-pod-identity-token").unwrap();

        let endpoint = server.url();

        let _mock = server
            .mock("GET", "/v2/credentials/eks")
            .match_header("Authorization", "my-pod-identity-token")
            .with_status(200)
            .with_body(r#"{"AccessKeyId":"EKSID","Code":"Success","Expiration":"2099-01-01T00:00:00Z","LastUpdated":"2022-08-30T10:21:04Z","SecretAccessKey":"EKSSECRET","Token":"EKSTOKEN","Type":"AWS-HMAC"}"#)
            .create_async()
            .await;

        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry = RetryConfig::default();

        let provider = TaskCredentialProvider {
            url: format!("{}/v2/credentials/eks", endpoint),
            auth_token_file: Some(auth_file.path().to_str().unwrap().to_owned()),
            retry: retry.clone(),
            client: client.clone(),
            service: service.clone(),
            cache: Default::default(),
        };

        let creds = provider.get_credential().await.unwrap();

        assert_eq!(creds.key_id, "EKSID");
        assert_eq!(creds.token.as_deref(), Some("EKSTOKEN"));

        _mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_assume_role_provider() {
        use crate::StaticCredentialProvider;
        let mut server = mockito::Server::new_async().await;

        let sts_endpoint = server.url();

        let _mock = server
            .mock("POST", "/")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("Action".into(), "AssumeRole".into()),
                mockito::Matcher::UrlEncoded(
                    "RoleArn".into(),
                    "arn:aws:iam::123456789012:role/AssumedRole".into(),
                ),
            ]))
            .with_status(200)
            .with_body(STS_ASSUME_ROLE_XML)
            .create_async()
            .await;

        let base_cred = AwsCredential {
            key_id: "BASEKEYID".to_string(),
            secret_key: "BASESECRET".to_string(),
            token: None,
        };
        let base_provider: Arc<dyn CredentialProvider<Credential = AwsCredential>> =
            Arc::new(StaticCredentialProvider::new(base_cred));

        let client = Client::new();
        let service: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client.clone()));
        let retry_config = RetryConfig::default();

        let provider = AssumeRoleProvider {
            role_arn: "arn:aws:iam::123456789012:role/AssumedRole".into(),
            session_name: "test-assume-session".into(),
            endpoint: format!("{}/", sts_endpoint),
            base_credentials: base_provider,
            region: "us-east-1".into(),
            policy: None,
        };

        let token = provider
            .fetch_token(&client, &service, &retry_config)
            .await
            .unwrap();

        assert_eq!(token.token.key_id, "AKIAIOSFODNN7EXAMPLE"); // gitleaks:allow
        assert!(token.token.token.is_some());
        assert!(token.expiry.is_some());

        _mock.assert_async().await;
    }
}
