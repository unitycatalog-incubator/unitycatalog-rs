use pyo3::create_exception;
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use thiserror::Error;
use unitycatalog_client::error::UcApiError;
use unitycatalog_common::error::Error as UCError;

/// A type wrapper around `Result<T, PyUnityCatalogError>`.
pub type PyUnityCatalogResult<T> = Result<T, PyUnityCatalogError>;

// Base exception
create_exception!(
    unitycatalog_client,
    UnityCatalogError,
    pyo3::exceptions::PyException,
    "The base Python-facing exception from which all other errors subclass."
);

// Subclasses from base exception
create_exception!(
    unitycatalog_client,
    GenericError,
    UnityCatalogError,
    "A Python-facing exception wrapping [unitycatalog_common::Error::Generic]."
);

create_exception!(
    unitycatalog_client,
    NotFoundError,
    UnityCatalogError,
    "Raised when a requested resource does not exist (RESOURCE_NOT_FOUND)."
);

create_exception!(
    unitycatalog_client,
    AlreadyExistsError,
    UnityCatalogError,
    "Raised when a resource already exists (RESOURCE_ALREADY_EXISTS)."
);

create_exception!(
    unitycatalog_client,
    PermissionDeniedError,
    UnityCatalogError,
    "Raised when the caller lacks permission (PERMISSION_DENIED)."
);

create_exception!(
    unitycatalog_client,
    UnauthenticatedError,
    UnityCatalogError,
    "Raised when the request is not authenticated (UNAUTHENTICATED)."
);

create_exception!(
    unitycatalog_client,
    InvalidParameterError,
    UnityCatalogError,
    "Raised when a request parameter is invalid (INVALID_PARAMETER_VALUE)."
);

create_exception!(
    unitycatalog_client,
    RequestLimitError,
    UnityCatalogError,
    "Raised when the request rate limit is exceeded (REQUEST_LIMIT_EXCEEDED)."
);

create_exception!(
    unitycatalog_client,
    InternalServerError,
    UnityCatalogError,
    "Raised when the server encounters an internal error (INTERNAL_ERROR)."
);

create_exception!(
    unitycatalog_client,
    ServiceUnavailableError,
    UnityCatalogError,
    "Raised when the service is temporarily unavailable (TEMPORARILY_UNAVAILABLE)."
);

/// The Error variants returned by this crate.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum PyUnityCatalogError {
    /// A wrapped [unitycatalog_common::error::Error]
    #[error(transparent)]
    UnityCatalogError(#[from] unitycatalog_common::error::Error),

    /// A wrapped [unitycatalog_client::error::Error]
    #[error(transparent)]
    UnityCatalogClientError(#[from] unitycatalog_client::error::Error),

    /// A wrapped [PyErr]
    #[error(transparent)]
    PyErr(#[from] PyErr),

    /// A wrapped [std::io::Error]
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    /// A wrapped [url::ParseError]
    #[error(transparent)]
    InvalidUrl(#[from] url::ParseError),
}

impl From<PyUnityCatalogError> for PyErr {
    fn from(error: PyUnityCatalogError) -> Self {
        match error {
            PyUnityCatalogError::PyErr(err) => err,
            PyUnityCatalogError::UnityCatalogError(ref err) => match err {
                UCError::Generic(_) => GenericError::new_err(print_with_debug(err)),
                _ => GenericError::new_err(print_with_debug(err)),
            },
            PyUnityCatalogError::UnityCatalogClientError(ref err) => match err {
                unitycatalog_client::Error::Api(api_err) => match api_err {
                    UcApiError::NotFound { .. } => NotFoundError::new_err(api_err.to_string()),
                    UcApiError::AlreadyExists { .. } => {
                        AlreadyExistsError::new_err(api_err.to_string())
                    }
                    UcApiError::PermissionDenied { .. } => {
                        PermissionDeniedError::new_err(api_err.to_string())
                    }
                    UcApiError::Unauthenticated { .. } => {
                        UnauthenticatedError::new_err(api_err.to_string())
                    }
                    UcApiError::InvalidParameter { .. } => {
                        InvalidParameterError::new_err(api_err.to_string())
                    }
                    UcApiError::RequestLimitExceeded { .. } => {
                        RequestLimitError::new_err(api_err.to_string())
                    }
                    UcApiError::InternalError { .. } => {
                        InternalServerError::new_err(api_err.to_string())
                    }
                    UcApiError::TemporarilyUnavailable { .. } => {
                        ServiceUnavailableError::new_err(api_err.to_string())
                    }
                    UcApiError::Other { .. } => GenericError::new_err(api_err.to_string()),
                    _ => GenericError::new_err(api_err.to_string()),
                },
                _ => GenericError::new_err(print_with_debug_client(err)),
            },
            PyUnityCatalogError::IOError(err) => PyIOError::new_err(err),
            PyUnityCatalogError::InvalidUrl(err) => PyValueError::new_err(err.to_string()),
        }
    }
}

fn print_with_debug(err: &UCError) -> String {
    // #? gives "pretty-printing" for debug
    // https://doc.rust-lang.org/std/fmt/trait.Debug.html
    format!("{err}\n\nDebug source:\n{err:#?}")
}

fn print_with_debug_client(err: &unitycatalog_client::Error) -> String {
    // #? gives "pretty-printing" for debug
    // https://doc.rust-lang.org/std/fmt/trait.Debug.html
    format!("{err}\n\nDebug source:\n{err:#?}")
}

/// Register all exception types with the given PyO3 module.
pub(crate) fn register_exceptions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("UnityCatalogError", m.py().get_type::<UnityCatalogError>())?;
    m.add("GenericError", m.py().get_type::<GenericError>())?;
    m.add("NotFoundError", m.py().get_type::<NotFoundError>())?;
    m.add(
        "AlreadyExistsError",
        m.py().get_type::<AlreadyExistsError>(),
    )?;
    m.add(
        "PermissionDeniedError",
        m.py().get_type::<PermissionDeniedError>(),
    )?;
    m.add(
        "UnauthenticatedError",
        m.py().get_type::<UnauthenticatedError>(),
    )?;
    m.add(
        "InvalidParameterError",
        m.py().get_type::<InvalidParameterError>(),
    )?;
    m.add("RequestLimitError", m.py().get_type::<RequestLimitError>())?;
    m.add(
        "InternalServerError",
        m.py().get_type::<InternalServerError>(),
    )?;
    m.add(
        "ServiceUnavailableError",
        m.py().get_type::<ServiceUnavailableError>(),
    )?;
    Ok(())
}
