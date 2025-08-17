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

//! A shared HTTP client implementation incorporating retries

use futures::future::BoxFuture;
use reqwest::header::LOCATION;
use reqwest::{Client, Request, Response, StatusCode};
use std::error::Error as StdError;
use std::time::{Duration, Instant};
use tracing::info;

use crate::backoff::{Backoff, BackoffConfig};

/// Retry request error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "Received redirect without LOCATION, this normally indicates an incorrectly configured region"
    )]
    BareRedirect,

    #[error("Server error, body contains Error, with status {status}: {}", body.as_deref().unwrap_or("No Body"))]
    Server {
        status: StatusCode,
        body: Option<String>,
    },

    #[error("Client error with status {status}: {}", body.as_deref().unwrap_or("No Body"))]
    Client {
        status: StatusCode,
        body: Option<String>,
    },

    #[error(
        "Error after {retries} retries in {elapsed:?}, max_retries:{max_retries}, retry_timeout:{retry_timeout:?}, source:{source}"
    )]
    Reqwest {
        retries: usize,
        max_retries: usize,
        elapsed: Duration,
        retry_timeout: Duration,
        source: reqwest::Error,
    },
}

impl Error {
    /// Returns the status code associated with this error if any
    pub fn status(&self) -> Option<StatusCode> {
        match self {
            Self::BareRedirect => None,
            Self::Server { status, .. } => Some(*status),
            Self::Client { status, .. } => Some(*status),
            Self::Reqwest { source, .. } => source.status(),
        }
    }

    /// Returns the error body if any
    pub fn body(&self) -> Option<&str> {
        match self {
            Self::Client { body, .. } => body.as_deref(),
            Self::Server { body, .. } => body.as_deref(),
            Self::BareRedirect => None,
            Self::Reqwest { .. } => None,
        }
    }

    pub fn error(self) -> crate::Error {
        match self.status() {
            Some(StatusCode::NOT_FOUND) => crate::Error::NotFound {
                source: Box::new(self),
            },
            Some(StatusCode::NOT_MODIFIED) => crate::Error::NotModified {
                source: Box::new(self),
            },
            Some(StatusCode::PRECONDITION_FAILED) => crate::Error::Precondition {
                source: Box::new(self),
            },
            Some(StatusCode::CONFLICT) => crate::Error::AlreadyExists {
                source: Box::new(self),
            },
            Some(StatusCode::FORBIDDEN) => crate::Error::PermissionDenied {
                source: Box::new(self),
            },
            Some(StatusCode::UNAUTHORIZED) => crate::Error::Unauthenticated {
                source: Box::new(self),
            },
            _ => crate::Error::Generic {
                source: Box::new(self),
            },
        }
    }
}

impl From<Error> for std::io::Error {
    fn from(err: Error) -> Self {
        use std::io::ErrorKind;
        match &err {
            Error::Client {
                status: StatusCode::NOT_FOUND,
                ..
            } => Self::new(ErrorKind::NotFound, err),
            Error::Client {
                status: StatusCode::BAD_REQUEST,
                ..
            } => Self::new(ErrorKind::InvalidInput, err),
            Error::Client {
                status: StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN,
                ..
            } => Self::new(ErrorKind::PermissionDenied, err),
            Error::Reqwest { source, .. } if source.is_timeout() => {
                Self::new(ErrorKind::TimedOut, err)
            }
            Error::Reqwest { source, .. } if source.is_connect() => {
                Self::new(ErrorKind::NotConnected, err)
            }
            _ => Self::other(err),
        }
    }
}

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

/// The configuration for how to respond to request errors
///
/// The following categories of error will be retried:
///
/// * 5xx server errors
/// * Connection errors
/// * Dropped connections
/// * Timeouts for [safe] / read-only requests
///
/// Requests will be retried up to some limit, using exponential
/// backoff with jitter. See [`BackoffConfig`] for more information
///
/// [safe]: https://datatracker.ietf.org/doc/html/rfc7231#section-4.2.1
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// The backoff configuration
    pub backoff: BackoffConfig,

    /// The maximum number of times to retry a request
    ///
    /// Set to 0 to disable retries
    pub max_retries: usize,

    /// The maximum length of time from the initial request
    /// after which no further retries will be attempted
    ///
    /// This not only bounds the length of time before a server
    /// error will be surfaced to the application, but also bounds
    /// the length of time a request's credentials must remain valid.
    ///
    /// As requests are retried without renewing credentials or
    /// regenerating request payloads, this number should be kept
    /// below 5 minutes to avoid errors due to expired credentials
    /// and/or request payloads
    pub retry_timeout: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            backoff: Default::default(),
            max_retries: 10,
            retry_timeout: Duration::from_secs(3 * 60),
        }
    }
}

pub(crate) struct RetryableRequest {
    client: Client,
    request: Request,

    max_retries: usize,
    retry_timeout: Duration,
    backoff: Backoff,

    sensitive: bool,
    idempotent: Option<bool>,
    retry_on_conflict: bool,
}

impl RetryableRequest {
    /// Set whether this request is idempotent
    ///
    /// An idempotent request will be retried on timeout even if the request
    /// method is not [safe](https://datatracker.ietf.org/doc/html/rfc7231#section-4.2.1)
    pub(crate) fn idempotent(self, idempotent: bool) -> Self {
        Self {
            idempotent: Some(idempotent),
            ..self
        }
    }

    /// Set whether this request should be retried on a 409 Conflict response.
    pub(crate) fn retry_on_conflict(self, retry_on_conflict: bool) -> Self {
        Self {
            retry_on_conflict,
            ..self
        }
    }

    /// Set whether this request contains sensitive data
    ///
    /// This will avoid printing out the URL in error messages
    #[allow(unused)]
    pub(crate) fn sensitive(self, sensitive: bool) -> Self {
        Self { sensitive, ..self }
    }

    pub(crate) async fn send(self) -> Result<Response> {
        let max_retries = self.max_retries;
        let retry_timeout = self.retry_timeout;
        let mut retries = 0;
        let now = Instant::now();

        let mut backoff = self.backoff;
        let is_idempotent = self
            .idempotent
            .unwrap_or_else(|| self.request.method().is_safe());

        let sanitize_err = move |e: reqwest::Error| match self.sensitive {
            true => e.without_url(),
            false => e,
        };

        loop {
            let request = self
                .request
                .try_clone()
                .expect("request body must be cloneable");

            match self.client.execute(request).await {
                Ok(r) => match r.error_for_status_ref() {
                    Ok(_) if r.status().is_success() => {
                        return Ok(r);
                    }
                    Ok(r) if r.status() == StatusCode::NOT_MODIFIED => {
                        return Err(Error::Client {
                            body: None,
                            status: StatusCode::NOT_MODIFIED,
                        });
                    }
                    Ok(r) => {
                        let is_bare_redirect =
                            r.status().is_redirection() && !r.headers().contains_key(LOCATION);
                        return match is_bare_redirect {
                            true => Err(Error::BareRedirect),
                            // Not actually sure if this is reachable, but here for completeness
                            false => Err(Error::Client {
                                body: None,
                                status: r.status(),
                            }),
                        };
                    }
                    Err(e) => {
                        let e = sanitize_err(e);
                        let status = r.status();
                        if retries == max_retries
                            || now.elapsed() > retry_timeout
                            || !(status.is_server_error()
                                || (self.retry_on_conflict && status == StatusCode::CONFLICT))
                        {
                            return Err(match status.is_client_error() {
                                true => match r.text().await {
                                    Ok(body) => Error::Client {
                                        body: Some(body).filter(|b| !b.is_empty()),
                                        status,
                                    },
                                    Err(e) => Error::Reqwest {
                                        retries,
                                        max_retries,
                                        elapsed: now.elapsed(),
                                        retry_timeout,
                                        source: e,
                                    },
                                },
                                false => Error::Reqwest {
                                    retries,
                                    max_retries,
                                    elapsed: now.elapsed(),
                                    retry_timeout,
                                    source: e,
                                },
                            });
                        }

                        let sleep = backoff.next();
                        retries += 1;
                        info!(
                            "Encountered server error, backing off for {} seconds, retry {} of {}: {}",
                            sleep.as_secs_f32(),
                            retries,
                            max_retries,
                            e,
                        );
                        tokio::time::sleep(sleep).await;
                    }
                },
                Err(e) => {
                    let e = sanitize_err(e);

                    let mut do_retry = false;
                    if e.is_connect()
                        || e.is_body()
                        || (e.is_request() && !e.is_timeout())
                        || (is_idempotent && e.is_timeout())
                    {
                        do_retry = true
                    } else {
                        let mut source = e.source();
                        while let Some(e) = source {
                            if let Some(e) = e.downcast_ref::<hyper::Error>() {
                                do_retry = e.is_closed()
                                    || e.is_incomplete_message()
                                    || e.is_body_write_aborted()
                                    || (is_idempotent && e.is_timeout());
                                break;
                            }
                            if let Some(e) = e.downcast_ref::<std::io::Error>() {
                                if e.kind() == std::io::ErrorKind::TimedOut {
                                    do_retry = is_idempotent;
                                } else {
                                    do_retry = matches!(
                                        e.kind(),
                                        std::io::ErrorKind::ConnectionReset
                                            | std::io::ErrorKind::ConnectionAborted
                                            | std::io::ErrorKind::BrokenPipe
                                            | std::io::ErrorKind::UnexpectedEof
                                    );
                                }
                                break;
                            }
                            source = e.source();
                        }
                    }

                    if retries == max_retries || now.elapsed() > retry_timeout || !do_retry {
                        return Err(Error::Reqwest {
                            retries,
                            max_retries,
                            elapsed: now.elapsed(),
                            retry_timeout,
                            source: e,
                        });
                    }
                    let sleep = backoff.next();
                    retries += 1;
                    info!(
                        "Encountered transport error backing off for {} seconds, retry {} of {}: {}",
                        sleep.as_secs_f32(),
                        retries,
                        max_retries,
                        e,
                    );
                    tokio::time::sleep(sleep).await;
                }
            }
        }
    }
}

pub(crate) trait RetryExt {
    /// Return a [`RetryableRequest`]
    fn retryable(self, config: &RetryConfig) -> RetryableRequest;

    /// Dispatch a request with the given retry configuration
    ///
    /// # Panic
    ///
    /// This will panic if the request body is a stream
    fn send_retry(self, config: &RetryConfig) -> BoxFuture<'static, Result<Response>>;
}

impl RetryExt for reqwest::RequestBuilder {
    fn retryable(self, config: &RetryConfig) -> RetryableRequest {
        let (client, request) = self.build_split();
        let request = request.expect("request must be valid");

        RetryableRequest {
            client,
            request,
            max_retries: config.max_retries,
            retry_timeout: config.retry_timeout,
            backoff: Backoff::new(&config.backoff),
            sensitive: false,
            idempotent: None,
            retry_on_conflict: false,
        }
    }

    fn send_retry(self, config: &RetryConfig) -> BoxFuture<'static, Result<Response>> {
        let request = self.retryable(config);
        Box::pin(async move { request.send().await })
    }
}

#[cfg(test)]
mod tests {
    use super::RetryConfig;
    use crate::retry::{Error, RetryExt};
    use hyper::Response;
    use hyper::header::LOCATION;
    use mockito;
    use reqwest::{Client, Method, StatusCode};
    use std::time::Duration;

    #[tokio::test]
    async fn test_retry() {
        let mut server = mockito::Server::new_async().await;

        let retry = RetryConfig {
            backoff: Default::default(),
            max_retries: 2,
            retry_timeout: Duration::from_secs(1000),
        };

        let client = Client::builder()
            .timeout(Duration::from_millis(100))
            .build()
            .unwrap();

        let server_url = server.url();
        let do_request = || client.request(Method::GET, &server_url).send_retry(&retry);

        // Simple request should work
        let _mock1 = server
            .mock("GET", "/")
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let r = do_request().await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);

        // Returns client errors immediately with status message
        let _mock2 = server
            .mock("GET", "/")
            .with_status(400)
            .with_body("cupcakes")
            .create_async()
            .await;

        let e = do_request().await.unwrap_err();
        assert_eq!(e.status().unwrap(), StatusCode::BAD_REQUEST);
        assert_eq!(e.body(), Some("cupcakes"));
        assert_eq!(
            e.to_string(),
            "Client error with status 400 Bad Request: cupcakes"
        );

        // Handles client errors with no payload
        let _mock3 = server
            .mock("GET", "/")
            .with_status(400)
            .with_body("")
            .create_async()
            .await;

        let e = do_request().await.unwrap_err();
        assert_eq!(e.status().unwrap(), StatusCode::BAD_REQUEST);
        assert_eq!(e.body(), None);
        assert_eq!(
            e.to_string(),
            "Client error with status 400 Bad Request: No Body"
        );

        // Should retry server error request
        let _mock4 = server
            .mock("GET", "/")
            .with_status(502)
            .with_body("")
            .create_async()
            .await;

        let _mock5 = server
            .mock("GET", "/")
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let r = do_request().await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);

        // Accepts 204 status code
        let _mock6 = server
            .mock("GET", "/")
            .with_status(204)
            .with_body("")
            .create_async()
            .await;

        let r = do_request().await.unwrap();
        assert_eq!(r.status(), StatusCode::NO_CONTENT);

        // Follows 302 redirects
        let _mock7 = server
            .mock("GET", "/")
            .with_status(302)
            .with_header("location", "/foo")
            .with_body("")
            .create_async()
            .await;

        let _mock8 = server
            .mock("GET", "/foo")
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let r = do_request().await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        assert_eq!(r.url().path(), "/foo");

        // Handles redirect missing location
        let _mock_redirect = server
            .mock("GET", "/")
            .with_status(302)
            .with_body("")
            .create_async()
            .await;

        let e = do_request().await.unwrap_err();
        assert!(matches!(e, Error::BareRedirect));
        assert_eq!(
            e.to_string(),
            "Received redirect without LOCATION, this normally indicates an incorrectly configured region"
        );

        // Test timeout scenarios with delays
        let _timeout_mock1 = server
            .mock("GET", "/")
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let _timeout_mock2 = server
            .mock("GET", "/")
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        do_request().await.unwrap();

        // Test sensitive URL handling
        let sensitive_url = format!("{}/SENSITIVE", server_url);
        let _sensitive_mock = server
            .mock("GET", "/SENSITIVE")
            .with_status(502)
            .with_body("ignored")
            .create_async()
            .await;

        let res = client
            .request(Method::GET, &sensitive_url)
            .send_retry(&retry)
            .await;
        let err = res.unwrap_err().to_string();
        assert!(err.contains("SENSITIVE"), "{err}");

        // Test sensitive request with retryable that strips URL
        let _sensitive_mock2 = server
            .mock("GET", "/SENSITIVE")
            .with_status(502)
            .with_body("ignored")
            .create_async()
            .await;

        let req = client
            .request(Method::GET, &sensitive_url)
            .retryable(&retry)
            .sensitive(true);
        let err = req.send().await.unwrap_err().to_string();
        assert!(!err.contains("SENSITIVE"), "{err}");
    }
}
