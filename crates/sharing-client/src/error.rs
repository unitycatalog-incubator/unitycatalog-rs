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
    AxumQuery(#[from] axum_extra::extract::QueryRejection),
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
            let error_code = self.error_code().to_string();
            let (status, message) = match self {
                Error::Common { source } => {
                    let (status, message) = match &source {
                        unitycatalog_common::Error::NotFound => (
                            StatusCode::NOT_FOUND,
                            "The requested resource does not exist.",
                        ),
                        unitycatalog_common::Error::InvalidArgument(msg) => {
                            error!("Invalid argument: {}", msg);
                            INVALID_ARGUMENT
                        }
                        unitycatalog_common::Error::InvalidIdentifier(e) => {
                            error!("Invalid identifier: {}", e);
                            INVALID_ARGUMENT
                        }
                        unitycatalog_common::Error::InvalidTableLocation(loc) => {
                            error!("Invalid table location: {}", loc);
                            INVALID_ARGUMENT
                        }
                        unitycatalog_common::Error::InvalidUrl(e) => {
                            error!("Invalid URL: {}", e);
                            INVALID_ARGUMENT
                        }
                        _ => {
                            error!("Common error: {}", source);
                            INTERNAL_ERROR
                        }
                    };
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
                Error::AxumPath(rejection) => {
                    error!("Path extraction error: {}", rejection);
                    (StatusCode::BAD_REQUEST, "Invalid path parameter.")
                }
                Error::AxumQuery(rejection) => {
                    error!("Query extraction error: {}", rejection);
                    (StatusCode::BAD_REQUEST, "Invalid query parameter.")
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
