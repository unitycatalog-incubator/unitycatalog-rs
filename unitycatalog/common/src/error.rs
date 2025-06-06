#[cfg(feature = "axum")]
use axum::extract::rejection::{PathRejection, QueryRejection};
#[cfg(feature = "grpc")]
use tonic::Status;

// A convenience type for declaring Results in the Delta Sharing libraries.
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("kernel error: {0}")]
    Kernel(#[from] delta_kernel::Error),

    #[error("Entity not found.")]
    NotFound,

    #[error("No or invalid token provided.")]
    Unauthenticated,

    #[error("Recipient is not allowed to read the entity.")]
    NotAllowed,

    #[error("Already exists")]
    AlreadyExists,

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

    #[error("Failed to extract recipient from request")]
    MissingRecipient,

    #[error("ObjectStore error: {0}")]
    ObjectStore(#[from] delta_kernel::object_store::Error),

    #[error(transparent)]
    SerDe(#[from] serde_json::Error),

    #[error(transparent)]
    Client(#[from] cloud_client::Error),

    #[error("invalid url: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("Reqwuest error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("DataFusion error: {0}")]
    DataFusion(#[from] datafusion::error::DataFusionError),

    #[error("Arrow error: {0}")]
    Arrow(#[from] datafusion::arrow::error::ArrowError),

    #[cfg(feature = "axum")]
    #[error("Axum path: {0}")]
    AxumPath(#[from] PathRejection),

    #[cfg(feature = "axum")]
    #[error("Axum query: {0}")]
    AxumQuery(#[from] QueryRejection),
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

#[cfg(feature = "grpc")]
impl From<Error> for Status {
    fn from(e: Error) -> Self {
        match e {
            Error::NotFound => Status::not_found("The requested resource does not exist."),
            Error::NotAllowed => {
                Status::permission_denied("The request is forbidden from being fulfilled.")
            }
            Error::Unauthenticated => Status::unauthenticated(
                "The request is unauthenticated. The bearer token is missing or incorrect.",
            ),
            Error::Kernel(error) => Status::internal(error.to_string()),
            Error::SerDe(_) => Status::internal("Encountered invalid table log."),
            Error::InvalidTableLocation(location) => {
                Status::internal(format!("Invalid table location: {}", location))
            }
            Error::MissingRecipient => {
                Status::invalid_argument("Failed to extract recipient from request")
            }
            Error::DataFusion(error) => Status::internal(error.to_string()),
            Error::Arrow(error) => Status::internal(error.to_string()),
            Error::InvalidPredicate(msg) => Status::invalid_argument(msg),
            Error::AlreadyExists => Status::already_exists("The resource already exists."),
            Error::InvalidIdentifier(_) => Status::internal("Invalid uuid identifier"),
            Error::InvalidArgument(message) => Status::invalid_argument(message),
            Error::Generic(message) => Status::internal(message),
            Error::Client(error) => Status::internal(error.to_string()),
            Error::InvalidUrl(_) => Status::internal("Invalid url"),
            Error::ObjectStore(_) => Status::internal("ObjectStore error"),
            Error::RequestError(error) => Status::internal(error.to_string()),
            #[cfg(feature = "axum")]
            Error::AxumPath(rejection) => Status::internal(format!("Axum path: {}", rejection)),
            #[cfg(feature = "axum")]
            Error::AxumQuery(rejection) => Status::internal(format!("Axum query: {}", rejection)),
        }
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
                Error::NotAllowed => (
                    StatusCode::FORBIDDEN,
                    "The request is forbidden from being fulfilled.",
                ),
                Error::AlreadyExists => (StatusCode::CONFLICT, "The resource already exists."),
                Error::Unauthenticated => (
                    StatusCode::UNAUTHORIZED,
                    "The request is unauthenticated. The bearer token is missing or incorrect.",
                ),
                Error::Kernel(error) => {
                    let message = format!("Kernel error: {}", error);
                    error!("delta-kernel error: {}", message);
                    INTERNAL_ERROR
                }
                Error::InvalidTableLocation(location) => {
                    let message = format!("Invalid table location: {}", location);
                    error!("{}", message);
                    INTERNAL_ERROR
                }
                Error::InvalidArgument(message) => {
                    error!("Invalid argument: {}", message);
                    INVALID_ARGUMENT
                }
                Error::Client(error) => {
                    let message = format!("Client error: {}", error);
                    error!("{}", message);
                    INTERNAL_ERROR
                }
                Error::InvalidUrl(_) => {
                    error!("Invalid url");
                    INTERNAL_ERROR
                }
                Error::ObjectStore(_) => {
                    error!("ObjectStore error");
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
                Error::DataFusion(error) => {
                    let message = format!("DataFusion error: {}", error);
                    error!("{}", message);
                    INTERNAL_ERROR
                }
                Error::Arrow(error) => {
                    let message = format!("Arrow error: {}", error);
                    error!("{}", message);
                    INTERNAL_ERROR
                }
                Error::MissingRecipient => {
                    error!("Failed to extract recipient from request");
                    (
                        StatusCode::BAD_REQUEST,
                        "Failed to extract recipient from request",
                    )
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
