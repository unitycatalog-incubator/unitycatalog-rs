pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
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

    #[error("API error: {0}")]
    Api(#[from] UcApiError),

    #[error("Generic error: {0}")]
    Generic(String),
}

impl Error {
    pub fn generic(message: impl ToString) -> Self {
        Error::Generic(message.to_string())
    }

    pub fn is_not_found(&self) -> bool {
        matches!(self, Error::Api(UcApiError::NotFound { .. }))
    }

    pub fn is_already_exists(&self) -> bool {
        matches!(self, Error::Api(UcApiError::AlreadyExists { .. }))
    }

    pub fn is_permission_denied(&self) -> bool {
        matches!(self, Error::Api(UcApiError::PermissionDenied { .. }))
    }

    pub fn is_unauthenticated(&self) -> bool {
        matches!(self, Error::Api(UcApiError::Unauthenticated { .. }))
    }
}

/// Typed error variants mapped to the Databricks Unity Catalog API error code spec.
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum UcApiError {
    #[error("Invalid parameter: {message}")]
    InvalidParameter { message: String },

    #[error("Unauthenticated: {message}")]
    Unauthenticated { message: String },

    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },

    #[error("Resource not found: {message}")]
    NotFound { message: String },

    #[error("Resource already exists: {message}")]
    AlreadyExists { message: String },

    #[error("Request limit exceeded: {message}")]
    RequestLimitExceeded { message: String },

    #[error("Internal server error: {message}")]
    InternalError { message: String },

    #[error("Temporarily unavailable: {message}")]
    TemporarilyUnavailable { message: String },

    #[error("API error {status}: [{error_code}] {message}")]
    Other {
        status: u16,
        error_code: String,
        message: String,
    },
}

impl UcApiError {
    /// Returns the UC API error code string.
    pub fn error_code(&self) -> &str {
        match self {
            UcApiError::InvalidParameter { .. } => "INVALID_PARAMETER_VALUE",
            UcApiError::Unauthenticated { .. } => "UNAUTHENTICATED",
            UcApiError::PermissionDenied { .. } => "PERMISSION_DENIED",
            UcApiError::NotFound { .. } => "RESOURCE_NOT_FOUND",
            UcApiError::AlreadyExists { .. } => "RESOURCE_ALREADY_EXISTS",
            UcApiError::RequestLimitExceeded { .. } => "REQUEST_LIMIT_EXCEEDED",
            UcApiError::InternalError { .. } => "INTERNAL_ERROR",
            UcApiError::TemporarilyUnavailable { .. } => "TEMPORARILY_UNAVAILABLE",
            UcApiError::Other { error_code, .. } => error_code,
        }
    }

    /// Returns the HTTP status code associated with this error.
    pub fn http_status(&self) -> u16 {
        match self {
            UcApiError::InvalidParameter { .. } => 400,
            UcApiError::Unauthenticated { .. } => 401,
            UcApiError::PermissionDenied { .. } => 403,
            UcApiError::NotFound { .. } => 404,
            UcApiError::AlreadyExists { .. } => 409,
            UcApiError::RequestLimitExceeded { .. } => 429,
            UcApiError::InternalError { .. } => 500,
            UcApiError::TemporarilyUnavailable { .. } => 503,
            UcApiError::Other { status, .. } => *status,
        }
    }

    /// Construct from an API response with status code, error code string, and message.
    pub fn from_api_response(status: u16, error_code: &str, message: String) -> Self {
        match error_code {
            "INVALID_PARAMETER_VALUE" => UcApiError::InvalidParameter { message },
            "UNAUTHENTICATED" => UcApiError::Unauthenticated { message },
            "PERMISSION_DENIED" => UcApiError::PermissionDenied { message },
            "RESOURCE_NOT_FOUND" => UcApiError::NotFound { message },
            "RESOURCE_ALREADY_EXISTS" => UcApiError::AlreadyExists { message },
            "REQUEST_LIMIT_EXCEEDED" => UcApiError::RequestLimitExceeded { message },
            "INTERNAL_ERROR" => UcApiError::InternalError { message },
            "TEMPORARILY_UNAVAILABLE" => UcApiError::TemporarilyUnavailable { message },
            other => UcApiError::Other {
                status,
                error_code: other.to_string(),
                message,
            },
        }
    }
}

/// Serde helper for parsing the UC API error body.
#[derive(serde::Deserialize)]
struct ApiErrorBody {
    #[serde(alias = "errorCode")]
    error_code: String,
    message: String,
}

/// Read an error HTTP response, parse the UC API JSON body, and return a typed [`Error`].
pub(crate) async fn parse_error_response(response: reqwest::Response) -> Error {
    let status = response.status().as_u16();
    match response.bytes().await {
        Ok(body) => match serde_json::from_slice::<ApiErrorBody>(&body) {
            Ok(api_err) => {
                UcApiError::from_api_response(status, &api_err.error_code, api_err.message).into()
            }
            Err(_) => UcApiError::Other {
                status,
                error_code: String::new(),
                message: String::from_utf8_lossy(&body).into_owned(),
            }
            .into(),
        },
        Err(e) => Error::RequestError(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_response(status: u16, body: &'static str) -> reqwest::Response {
        http::Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .body(bytes::Bytes::from_static(body.as_bytes()))
            .map(reqwest::Response::from)
            .unwrap()
    }

    #[tokio::test]
    async fn test_parse_error_resource_not_found() {
        let resp = make_response(
            404,
            r#"{"error_code":"RESOURCE_NOT_FOUND","message":"catalog 'foo' not found"}"#,
        );
        let err = parse_error_response(resp).await;
        assert!(err.is_not_found());
        assert!(matches!(
            err,
            Error::Api(UcApiError::NotFound { ref message }) if message == "catalog 'foo' not found"
        ));
    }

    #[tokio::test]
    async fn test_parse_error_camel_case_alias() {
        let resp = make_response(
            404,
            r#"{"errorCode":"RESOURCE_NOT_FOUND","message":"not found"}"#,
        );
        let err = parse_error_response(resp).await;
        assert!(err.is_not_found());
    }

    #[tokio::test]
    async fn test_parse_error_non_json_body() {
        let resp = make_response(500, "Internal Server Error");
        let err = parse_error_response(resp).await;
        assert!(matches!(
            err,
            Error::Api(UcApiError::Other {
                status: 500,
                ref message,
                ..
            }) if message == "Internal Server Error"
        ));
    }

    #[tokio::test]
    async fn test_parse_error_already_exists() {
        let resp = make_response(
            409,
            r#"{"error_code":"RESOURCE_ALREADY_EXISTS","message":"already exists"}"#,
        );
        let err = parse_error_response(resp).await;
        assert!(err.is_already_exists());
    }
}
