use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
#[cfg(feature = "grpc")]
use tonic::Status;
use tracing::error;
use unitycatalog_common::ErrorResponse;

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

    #[error("Sharing Error: {source}")]
    Sharing {
        #[from]
        source: unitycatalog_sharing_client::Error,
    },

    #[error("Object Store Error: {source}")]
    ObjectStore {
        #[from]
        source: object_store::Error,
    },

    #[error("Serialization Error: {source}")]
    SerDe {
        #[from]
        source: serde_json::Error,
    },

    #[error("Entity not found.")]
    NotFound,

    #[error("No or invalid token provided.")]
    Unauthenticated,

    #[error("Recipient is not allowed to read the entity.")]
    NotAllowed,

    #[error("Already exists")]
    AlreadyExists,

    #[error("Invalid argument")]
    InvalidArgument(String),

    #[error("Invalid identifier: {0}")]
    InvalidIdentifier(#[from] uuid::Error),

    #[error("Missing recipient")]
    MissingRecipient,

    #[error("Generic error: {0}")]
    Generic(String),
}

impl Error {
    pub fn generic(message: impl ToString) -> Self {
        Error::Generic(message.to_string())
    }

    pub fn invalid_argument(message: impl ToString) -> Self {
        Error::InvalidArgument(message.to_string())
    }
}

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
            // EXTERNAL ERRORS
            Error::DeltaKernel { .. } => {
                error!("Failed to interact with Delta Kernel");
                INTERNAL_ERROR
            }
            Error::Sharing { .. } => {
                error!("DELTA Sharing Error");
                INTERNAL_ERROR
            }
            Error::ObjectStore { .. } => {
                error!("Failed to interact with object store");
                INTERNAL_ERROR
            }
            Error::SerDe { .. } => {
                error!("Failed to serialize/deserialize data");
                INTERNAL_ERROR
            }

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
            Error::InvalidArgument(message) => {
                error!("Invalid argument: {}", message);
                INVALID_ARGUMENT
            }
            Error::InvalidIdentifier(_) => {
                error!("Invalid uuid identifier");
                INTERNAL_ERROR
            }
            Error::Generic(message) => {
                error!("Generic error: {}", message);
                INTERNAL_ERROR
            }
            // Error::DataFusion(error) => {
            //     let message = format!("DataFusion error: {}", error);
            //     error!("{}", message);
            //     INTERNAL_ERROR
            // }
            // Error::Arrow(error) => {
            //     let message = format!("Arrow error: {}", error);
            //     error!("{}", message);
            //     INTERNAL_ERROR
            // }
            Error::MissingRecipient => {
                error!("Failed to extract recipient from request");
                (
                    StatusCode::BAD_REQUEST,
                    "Failed to extract recipient from request",
                )
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
            // Error::DataFusion(error) => Status::internal(error.to_string()),
            // Error::Arrow(error) => Status::internal(error.to_string()),
            Error::InvalidPredicate(msg) => Status::invalid_argument(msg),
            Error::AlreadyExists => Status::already_exists("The resource already exists."),
            Error::InvalidIdentifier(_) => Status::internal("Invalid uuid identifier"),
            Error::InvalidArgument(message) => Status::invalid_argument(message),
            Error::Generic(message) => Status::internal(message),
            // Error::Client(error) => Status::internal(error.to_string()),
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
