/// A convenience type for declaring Results in the Delta Sharing libraries.
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Entity not found.")]
    NotFound,

    #[error("Invalid table location: {0}")]
    InvalidTableLocation(String),

    #[error("Invalid Argument: {0}")]
    InvalidArgument(String),

    #[error("Invalid identifier: {0}")]
    InvalidIdentifier(#[from] uuid::Error),

    #[error("Invalid predicate: {0}")]
    InvalidPredicate(String),

    #[error("Generic error: {0}")]
    Generic(String),

    #[error(transparent)]
    SerDe(#[from] serde_json::Error),

    #[error("invalid url: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("Reqwuest error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[cfg(feature = "axum")]
    #[error("Axum path: {0}")]
    AxumPath(#[from] axum::extract::rejection::PathRejection),

    #[cfg(feature = "axum")]
    #[error("Axum query: {0}")]
    AxumQuery(#[from] axum::extract::rejection::QueryRejection),
}

impl Error {
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }

    pub fn invalid_argument(msg: impl Into<String>) -> Self {
        Self::InvalidArgument(msg.into())
    }

    pub fn invalid_predicate(msg: impl Into<String>) -> Self {
        Self::InvalidPredicate(msg.into())
    }
}

#[cfg(feature = "axum")]
mod server {
    use axum::extract::Json;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};
    use tracing::error;

    use super::Error;
    use crate::models::ErrorResponse;

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
                Error::NotFound => (
                    StatusCode::NOT_FOUND,
                    "The requested resource does not exist.",
                ),
                Error::InvalidTableLocation(location) => {
                    let message = format!("Invalid table location: {}", location);
                    error!("{}", message);
                    INTERNAL_ERROR
                }
                Error::InvalidArgument(message) => {
                    error!("Invalid argument: {}", message);
                    INVALID_ARGUMENT
                }
                Error::InvalidUrl(_) => {
                    error!("Invalid url");
                    INTERNAL_ERROR
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
                Error::InvalidIdentifier(_) => {
                    error!("Invalid uuid identifier");
                    INTERNAL_ERROR
                }
                Error::SerDe(_) => {
                    error!("Invalid table log encountered");
                    INTERNAL_ERROR
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
