pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Delta Kernel Error: {source}")]
    DeltaKernel {
        #[from]
        source: delta_kernel::Error,
    },

    #[error("Common Error: {source}")]
    Common {
        #[from]
        source: unitycatalog_common::Error,
    },

    #[error("Client Error: {source}")]
    ClientError {
        #[from]
        source: olai_http::Error,
    },

    #[error("Malformed response: {source}")]
    MalformedResponse {
        #[from]
        source: serde_json::Error,
    },

    #[error("Malformed url: {source}")]
    MalformedUrl {
        #[from]
        source: url::ParseError,
    },

    #[error("Reqwuest error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Invalid Argument: {0}")]
    InvalidArgument(String),

    #[error("Invalid predicate: {0}")]
    InvalidPredicate(String),

    #[error("Generic error: {0}")]
    Generic(String),
}

/// Consume an error HTTP response and turn it into an [`Error`].
///
/// Used by the generated clients on non-success statuses. The sharing client's
/// error type is coarse, so the status and raw body are folded into
/// [`Error::Generic`] for diagnostics.
pub(crate) async fn parse_error_response(response: reqwest::Response) -> Error {
    let status = response.status();
    match response.bytes().await {
        Ok(body) => Error::Generic(format!(
            "request failed with status {status}: {}",
            String::from_utf8_lossy(&body)
        )),
        Err(e) => Error::RequestError(e),
    }
}

impl Error {
    pub fn generic(message: impl ToString) -> Self {
        Error::Generic(message.to_string())
    }

    pub fn invalid_predicate(msg: impl ToString) -> Self {
        Self::InvalidPredicate(msg.to_string())
    }

    /// Returns a machine-readable error code matching the UC API spec.
    pub fn error_code(&self) -> &str {
        match self {
            Error::InvalidArgument(_) | Error::InvalidPredicate(_) | Error::MalformedUrl { .. } => {
                "INVALID_PARAMETER_VALUE"
            }
            Error::Common { source } => source.error_code(),
            _ => "INTERNAL_ERROR",
        }
    }
}

#[cfg(feature = "server")]
mod server {
    use axum::extract::Json;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};
    use tracing::error;
    use unitycatalog_common::ErrorResponse;

    use super::Error;

    const INTERNAL_ERROR: (StatusCode, &str) = (
        StatusCode::INTERNAL_SERVER_ERROR,
        "The request is not handled correctly due to a server error.",
    );

    const INVALID_ARGUMENT: (StatusCode, &str) = (
        StatusCode::BAD_REQUEST,
        "Invalid argument provided in the request.",
    );

    impl IntoResponse for Error {
        fn into_response(self) -> Response {
            let error_code = self.error_code().to_string();
            let (status, message) = match self {
                Error::Common { source } => {
                    let (status, message) = source.response_parts();
                    return (
                        status,
                        Json(ErrorResponse {
                            error_code,
                            message: message.to_string(),
                        }),
                    )
                        .into_response();
                }
                Error::DeltaKernel { source } => {
                    error!("Delta Kernel error: {}", source);
                    INTERNAL_ERROR
                }
                Error::InvalidArgument(message) => {
                    error!("Invalid argument: {}", message);
                    INVALID_ARGUMENT
                }
                Error::ClientError { source } => {
                    error!("Client error: {}", source);
                    INTERNAL_ERROR
                }
                Error::MalformedUrl { source } => {
                    error!("Malformed URL: {}", source);
                    INTERNAL_ERROR
                }
                Error::MalformedResponse { source } => {
                    error!("Malformed response: {}", source);
                    INTERNAL_ERROR
                }
                Error::RequestError(error) => {
                    error!("Request error: {}", error);
                    INTERNAL_ERROR
                }
                Error::InvalidPredicate(msg) => {
                    error!("Invalid predicate: {}", msg);
                    (
                        StatusCode::BAD_REQUEST,
                        "Invalid predicate provided in the request.",
                    )
                }
                Error::Generic(message) => {
                    error!("Generic error: {}", message);
                    INTERNAL_ERROR
                }
            };

            (
                status,
                Json(ErrorResponse {
                    error_code,
                    message: message.to_string(),
                }),
            )
                .into_response()
        }
    }
}
