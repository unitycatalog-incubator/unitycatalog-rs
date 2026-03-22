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

    #[error("Generic error: {0}")]
    Generic(String),

    #[error(transparent)]
    SerDe(#[from] serde_json::Error),

    #[error("invalid url: {0}")]
    InvalidUrl(#[from] url::ParseError),
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
            Error::InvalidArgument(_) => "INVALID_PARAMETER_VALUE",
            Error::InvalidIdentifier(_) => "INVALID_PARAMETER_VALUE",
            Error::InvalidTableLocation(_) => "INVALID_PARAMETER_VALUE",
            Error::InvalidUrl(_) => "INVALID_PARAMETER_VALUE",
            Error::SerDe(_) => "INTERNAL_ERROR",
            Error::Generic(_) => "INTERNAL_ERROR",
        }
    }
}
