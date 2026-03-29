//! Authentication middleware for Delta Sharing server.
use std::marker::PhantomData;
use std::task::{Context, Poll};

use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use futures_util::{FutureExt, future::BoxFuture};
use tower::{Layer, Service};
use unitycatalog_common::Result;

use crate::policy::Principal;

/// Authenticator for authenticating requests to a sharing server.
///
/// `I` is the identity type inserted into request extensions. It must be
/// `Clone + Send + Sync + 'static` so it can be stored in axum extensions.
pub trait Authenticator<I: Clone + Send + Sync + 'static>: Send + Sync + 'static {
    /// Authenticate a request and return an identity value.
    ///
    /// This method should return the identity of the caller, or an error if the
    /// request is not authenticated or the identity cannot be determined.
    fn authenticate(&self, request: &Request) -> Result<I>;
}

/// Authenticator that always marks the recipient as anonymous.
///
/// This is the default authenticator used when no authentication is configured.
/// The server is designed to run behind a reverse proxy (e.g., nginx, Envoy) that
/// handles authentication and injects the authenticated identity (e.g., via an
/// `X-Forwarded-User` header). A production `Authenticator` implementation should
/// read that header and return `Principal::User(name)`.
///
/// TODO: implement a `ReverseProxyAuthenticator` that extracts `Principal` from
/// the `X-Forwarded-User` (or similar) header set by the upstream proxy.
#[derive(Clone)]
pub struct AnonymousAuthenticator;

impl Authenticator<Principal> for AnonymousAuthenticator {
    fn authenticate(&self, _: &Request) -> Result<Principal> {
        // TODO: extract from reverse proxy middleware (e.g., X-Forwarded-User header)
        Ok(Principal::anonymous())
    }
}

/// Middleware that authenticates requests using the given [`Authenticator`].
#[derive(Clone)]
pub struct AuthenticationMiddleware<S, T, I = Principal> {
    inner: S,
    authenticator: T,
    _identity: PhantomData<I>,
}

#[allow(unused)]
impl<S, T, I> AuthenticationMiddleware<S, T, I> {
    /// Create new [`AuthenticationMiddleware`].
    pub fn new(inner: S, authenticator: T) -> Self {
        Self {
            inner,
            authenticator,
            _identity: PhantomData,
        }
    }

    /// Create a new [`AuthenticationLayer`] with the given [`Authenticator`].
    ///
    /// This is a convenience method that is equivalent to calling [`AuthenticationLayer::new`].
    pub fn layer(authenticator: T) -> AuthenticationLayer<T, I> {
        AuthenticationLayer::new(authenticator)
    }
}

impl<S, T, I> Service<Request> for AuthenticationMiddleware<S, T, I>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
    T: Authenticator<I>,
    I: Clone + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        match self.authenticator.authenticate(&req) {
            Ok(identity) => {
                req.extensions_mut().insert(identity);
                self.inner.call(req).boxed()
            }
            Err(e) => async { Ok(crate::Error::from(e).into_response()) }.boxed(),
        }
    }
}

/// Layer that applies the [`AuthenticationMiddleware`].
#[derive(Clone)]
pub struct AuthenticationLayer<T, I = Principal> {
    authenticator: T,
    _identity: PhantomData<I>,
}

impl<T, I> AuthenticationLayer<T, I> {
    /// Create a new [`AuthenticationLayer`] with the provided [`Authenticator`].
    pub fn new(authenticator: T) -> Self {
        Self {
            authenticator,
            _identity: PhantomData,
        }
    }
}

impl<S, T, I> Layer<S> for AuthenticationLayer<T, I>
where
    T: Clone + Send + Sync + 'static,
    I: Clone + Send + Sync + 'static,
{
    type Service = AuthenticationMiddleware<S, T, I>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthenticationMiddleware {
            inner,
            authenticator: self.authenticator.clone(),
            _identity: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::{StatusCode, header};
    use tower::{ServiceBuilder, ServiceExt};

    use super::*;

    async fn check_recipient(req: Request) -> Result<Response<Body>> {
        assert!(matches!(
            req.extensions().get::<Principal>(),
            Some(Principal::Anonymous) | Some(Principal::User(_))
        ));
        Ok(Response::new(req.into_body()))
    }

    #[tokio::test]
    async fn test_authentication_middleware() {
        let authenticator = AnonymousAuthenticator {};
        let mut service = ServiceBuilder::new()
            .layer(AuthenticationLayer::new(authenticator))
            .service_fn(check_recipient);

        let request = Request::get("/")
            .header(header::AUTHORIZATION, "Bearer foo")
            .body(Body::empty())
            .unwrap();

        let response = service.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let request = Request::get("/").body(Body::empty()).unwrap();
        let response = service.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
