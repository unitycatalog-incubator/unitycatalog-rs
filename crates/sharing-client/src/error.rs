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
        source: cloud_client::Error,
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

    #[cfg(feature = "server")]
    #[error("Axum path: {0}")]
    AxumPath(#[from] axum::extract::rejection::PathRejection),

    #[cfg(feature = "server")]
    #[error("Axum query: {0}")]
    AxumQuery(#[from] axum::extract::rejection::QueryRejection),
}

impl Error {
    pub fn generic(message: impl ToString) -> Self {
        Error::Generic(message.to_string())
    }

    pub fn invalid_predicate(msg: impl ToString) -> Self {
        Self::InvalidPredicate(msg.to_string())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error_code: String,
    pub message: String,
}

#[cfg(feature = "server")]
mod server {
    use axum::extract::Json;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};
    use tracing::error;

    use super::{Error, ErrorResponse};

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
            let (status, message) = match self {
                Error::Common { source } => return source.into_response(),
                Error::DeltaKernel { source } => {
                    error!("Kernel error: {}", source);
                    INVALID_ARGUMENT
                }
                Error::InvalidArgument(message) => {
                    error!("Invalid argument: {}", message);
                    INVALID_ARGUMENT
                }
                Error::ClientError { source } => {
                    error!("Client error: {}", source);
                    INVALID_ARGUMENT
                }
                Error::MalformedUrl { source } => {
                    error!("Malformed URL: {}", source);
                    INVALID_ARGUMENT
                }
                Error::MalformedResponse { source } => {
                    error!("Malformed response: {}", source);
                    INVALID_ARGUMENT
                }
                Error::RequestError(error) => {
                    let message = format!("Request error: {}", error);
                    error!("{}", message);
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
                // TODO(roeap): what codes should these have?
                #[cfg(feature = "axum")]
                Error::AxumPath(rejection) => {
                    let message = format!("Axum path: {}", rejection);
                    error!("{}", message);
                    INTERNAL_ERROR
                }
                #[cfg(feature = "axum")]
                Error::AxumQuery(rejection) => {
                    let message = format!("Axum query: {}", rejection);
                    error!("{}", message);
                    INTERNAL_ERROR
                }
            };

            (
                status,
                Json(ErrorResponse {
                    error_code: status.to_string(),
                    message: message.to_string(),
                }),
            )
                .into_response()
        }
    }
}
