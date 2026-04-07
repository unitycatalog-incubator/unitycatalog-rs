use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::runtime::Handle;

/// An abstraction over HTTP request execution.
///
/// This trait allows decoupling request building (which uses `reqwest::RequestBuilder`)
/// from request execution. The primary use case is [`SpawnService`], which spawns
/// execution on a dedicated I/O runtime -- useful when the CPU runtime has I/O disabled.
pub trait HttpService: Debug + Send + Sync + 'static {
    /// Execute an HTTP request, returning the response.
    fn call(
        &self,
        request: reqwest::Request,
    ) -> Pin<Box<dyn Future<Output = Result<reqwest::Response, reqwest::Error>> + Send + '_>>;
}

/// Default [`HttpService`] that delegates directly to [`reqwest::Client::execute`].
#[derive(Debug, Clone)]
pub struct ReqwestService(reqwest::Client);

impl ReqwestService {
    pub fn new(client: reqwest::Client) -> Self {
        Self(client)
    }
}

impl HttpService for ReqwestService {
    fn call(
        &self,
        request: reqwest::Request,
    ) -> Pin<Box<dyn Future<Output = Result<reqwest::Response, reqwest::Error>> + Send + '_>> {
        Box::pin(self.0.execute(request))
    }
}

/// An [`HttpService`] that spawns each request on a separate tokio runtime.
///
/// This is useful when the calling runtime (e.g. a CPU-bound DataFusion runtime)
/// may have I/O disabled. All HTTP I/O -- including credential refresh -- is
/// routed through the provided runtime handle.
///
/// # Example
///
/// ```ignore
/// use cloud_client::CloudClient;
///
/// let io_runtime = tokio::runtime::Runtime::new().unwrap();
/// let client = CloudClient::new_with_token("tok")
///     .with_runtime(io_runtime.handle().clone());
/// ```
#[derive(Debug)]
pub struct SpawnService {
    inner: Arc<dyn HttpService>,
    handle: Handle,
}

impl SpawnService {
    pub fn new(inner: Arc<dyn HttpService>, handle: Handle) -> Self {
        Self { inner, handle }
    }
}

impl HttpService for SpawnService {
    fn call(
        &self,
        request: reqwest::Request,
    ) -> Pin<Box<dyn Future<Output = Result<reqwest::Response, reqwest::Error>> + Send + '_>> {
        let inner = Arc::clone(&self.inner);
        let handle = self.handle.clone();
        Box::pin(async move {
            match handle.spawn(async move { inner.call(request).await }).await {
                Ok(result) => result,
                Err(join_err) => {
                    // The spawned I/O task panicked or was cancelled. This is a
                    // programming error (e.g. tokio runtime was shut down), not a
                    // transient network failure, so we re-panic with context rather
                    // than silently swallowing the cause.
                    panic!("I/O runtime task failed: {join_err}")
                }
            }
        })
    }
}

/// Create an [`HttpService`] for the given client, optionally wrapping in [`SpawnService`].
pub(crate) fn make_service(
    client: reqwest::Client,
    runtime: Option<&Handle>,
) -> Arc<dyn HttpService> {
    let base: Arc<dyn HttpService> = Arc::new(ReqwestService::new(client));
    match runtime {
        Some(handle) => Arc::new(SpawnService::new(base, handle.clone())),
        None => base,
    }
}
