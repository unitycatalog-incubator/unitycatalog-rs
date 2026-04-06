use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::future::BoxFuture;
use reqwest::Client;

use crate::service::HttpService;
use crate::{Result, RetryConfig, TemporaryToken, TokenCache};

/// Provides credentials for use when signing requests.
///
/// Implementors are responsible for fetching, refreshing, and returning a
/// credential of an associated type (e.g. an AWS SigV4 key set, an Azure bearer
/// token, or a GCP service-account token).  The cloud-client crate ships
/// ready-made implementations for every supported provider; see the per-cloud
/// `credential` modules for details.
#[async_trait]
pub trait CredentialProvider: std::fmt::Debug + Send + Sync {
    /// The type of credential returned by this provider
    type Credential;

    /// Return a credential
    async fn get_credential(&self) -> Result<Arc<Self::Credential>>;
}

/// A static set of credentials
#[derive(Debug)]
pub struct StaticCredentialProvider<T> {
    credential: Arc<T>,
}

impl<T> StaticCredentialProvider<T> {
    /// A [`CredentialProvider`] for a static credential of type `T`
    pub fn new(credential: T) -> Self {
        Self {
            credential: Arc::new(credential),
        }
    }
}

#[async_trait]
impl<T> CredentialProvider for StaticCredentialProvider<T>
where
    T: std::fmt::Debug + Send + Sync,
{
    type Credential = T;

    async fn get_credential(&self) -> Result<Arc<T>> {
        Ok(Arc::clone(&self.credential))
    }
}

/// A [`CredentialProvider`] that fetches temporary tokens via HTTP
#[derive(Debug)]
pub(crate) struct TokenCredentialProvider<T: TokenProvider> {
    inner: T,
    client: Client,
    service: Arc<dyn HttpService>,
    retry: RetryConfig,
    cache: TokenCache<Arc<T::Credential>>,
}

impl<T: TokenProvider> TokenCredentialProvider<T> {
    pub(crate) fn new(
        inner: T,
        client: Client,
        service: Arc<dyn HttpService>,
        retry: RetryConfig,
    ) -> Self {
        Self {
            inner,
            client,
            service,
            retry,
            cache: Default::default(),
        }
    }

    /// Override the minimum remaining TTL for a cached token to be used
    pub(crate) fn with_min_ttl(mut self, min_ttl: Duration) -> Self {
        self.cache = self.cache.with_min_ttl(min_ttl);
        self
    }
}

#[async_trait]
impl<T: TokenProvider> CredentialProvider for TokenCredentialProvider<T> {
    type Credential = T::Credential;

    async fn get_credential(&self) -> Result<Arc<Self::Credential>> {
        self.cache
            .get_or_insert_with(|| {
                self.inner
                    .fetch_token(&self.client, &self.service, &self.retry)
            })
            .await
    }
}

/// Applies authentication to a built reqwest request.
///
/// Each cloud provider and Databricks implements this trait.
/// `send()` in `CloudClient` calls `self.signer.sign(builder).await?`.
///
/// # References
/// - <https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html>
pub trait RequestSigner: Debug + Send + Sync {
    /// Sign a [`reqwest::RequestBuilder`], returning the modified builder.
    fn sign<'a>(
        &'a self,
        req: reqwest::RequestBuilder,
    ) -> BoxFuture<'a, Result<reqwest::RequestBuilder>>;
}

/// Tries signers in order; uses the first that succeeds without error.
///
/// This enables Databricks-style fallback chains without bespoke builder logic.
#[derive(Debug)]
pub struct SignerChain {
    signers: Vec<Arc<dyn RequestSigner>>,
}

impl SignerChain {
    pub fn new(signers: Vec<Arc<dyn RequestSigner>>) -> Self {
        Self { signers }
    }
}

impl RequestSigner for SignerChain {
    fn sign<'a>(
        &'a self,
        req: reqwest::RequestBuilder,
    ) -> BoxFuture<'a, Result<reqwest::RequestBuilder>> {
        Box::pin(async move {
            let mut current = req;
            for signer in &self.signers {
                current = signer.sign(current).await?;
            }
            Ok(current)
        })
    }
}

#[async_trait]
pub(crate) trait TokenProvider: std::fmt::Debug + Send + Sync {
    type Credential: std::fmt::Debug + Send + Sync;

    async fn fetch_token(
        &self,
        client: &Client,
        service: &Arc<dyn HttpService>,
        retry: &RetryConfig,
    ) -> Result<TemporaryToken<Arc<Self::Credential>>>;
}
