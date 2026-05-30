/// A convenience type for declaring Results in the Delta Sharing libraries.
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Entity not found.")]
    NotFound,

    #[error("Already exists")]
    AlreadyExists,

    #[error("Invalid table location: {0}")]
    InvalidTableLocation(String),

    #[error("Invalid Argument: {0}")]
    InvalidArgument(String),

    #[error("Invalid identifier: {0}")]
    InvalidIdentifier(#[from] uuid::Error),

    #[error("Generic error: {0}")]
    Generic(String),

    #[error(transparent)]
    SerDe(#[from] serde_json::Error),

    #[error("invalid url: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error(transparent)]
    ResourceStore(#[from] olai_store::Error),
}

impl Error {
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }

    pub fn invalid_argument(msg: impl Into<String>) -> Self {
        Self::InvalidArgument(msg.into())
    }

    /// Returns a machine-readable error code matching the UC API spec.
    pub fn error_code(&self) -> &str {
        match self {
            Error::NotFound => "RESOURCE_NOT_FOUND",
            Error::AlreadyExists => "RESOURCE_ALREADY_EXISTS",
            Error::InvalidArgument(_) => "INVALID_PARAMETER_VALUE",
            Error::InvalidIdentifier(_) => "INVALID_PARAMETER_VALUE",
            Error::InvalidTableLocation(_) => "INVALID_PARAMETER_VALUE",
            Error::InvalidUrl(_) => "INVALID_PARAMETER_VALUE",
            Error::SerDe(_) => "INTERNAL_ERROR",
            Error::Generic(_) => "INTERNAL_ERROR",
            Error::ResourceStore(e) => match e {
                olai_store::Error::NotFound => "RESOURCE_NOT_FOUND",
                olai_store::Error::AlreadyExists => "ALREADY_EXISTS",
                olai_store::Error::InvalidArgument(_) => "INVALID_PARAMETER_VALUE",
                olai_store::Error::InvalidIdentifier(_) => "INVALID_PARAMETER_VALUE",
                _ => "INTERNAL_ERROR",
            },
        }
    }
}

#[cfg(feature = "axum")]
impl Error {
    /// Maps this error to an HTTP status and a static client-facing message.
    ///
    /// Shared by downstream `IntoResponse` impls (server, sharing-client) so the
    /// status/message table for common variants lives in one place. Callers
    /// remain responsible for composing the `error_code` and wrapping the result
    /// in their own `ErrorResponse` body.
    pub fn response_parts(&self) -> (http::StatusCode, &'static str) {
        use http::StatusCode;

        const INTERNAL: &str = "The request is not handled correctly due to a server error.";
        const INVALID: &str = "Invalid argument provided in the request.";
        const NOT_FOUND: &str = "The requested resource does not exist.";
        const ALREADY_EXISTS: &str = "The resource already exists.";

        match self {
            Error::NotFound => (StatusCode::NOT_FOUND, NOT_FOUND),
            Error::AlreadyExists => (StatusCode::CONFLICT, ALREADY_EXISTS),
            Error::InvalidArgument(msg) => {
                tracing::error!("Invalid argument: {msg}");
                (StatusCode::BAD_REQUEST, INVALID)
            }
            Error::InvalidIdentifier(e) => {
                tracing::error!("Invalid identifier: {e}");
                (StatusCode::BAD_REQUEST, INVALID)
            }
            Error::InvalidTableLocation(loc) => {
                tracing::error!("Invalid table location: {loc}");
                (StatusCode::BAD_REQUEST, INVALID)
            }
            Error::InvalidUrl(e) => {
                tracing::error!("Invalid URL: {e}");
                (StatusCode::BAD_REQUEST, INVALID)
            }
            Error::SerDe(e) => {
                tracing::error!("Serialization error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, INTERNAL)
            }
            Error::Generic(msg) => {
                tracing::error!("Generic common error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, INTERNAL)
            }
            Error::ResourceStore(e) => match e {
                olai_store::Error::NotFound => (StatusCode::NOT_FOUND, NOT_FOUND),
                olai_store::Error::AlreadyExists => (StatusCode::CONFLICT, ALREADY_EXISTS),
                olai_store::Error::InvalidArgument(msg) => {
                    tracing::error!("Invalid argument: {msg}");
                    (StatusCode::BAD_REQUEST, INVALID)
                }
                olai_store::Error::InvalidIdentifier(e) => {
                    tracing::error!("Invalid identifier: {e}");
                    (StatusCode::BAD_REQUEST, INVALID)
                }
                _ => {
                    tracing::error!("Resource store error: {e}");
                    (StatusCode::INTERNAL_SERVER_ERROR, INTERNAL)
                }
            },
        }
    }
}
