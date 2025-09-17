use pyo3::create_exception;
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use thiserror::Error;
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
            PyUnityCatalogError::UnityCatalogClientError(ref err) => {
                GenericError::new_err(print_with_debug_client(err))
            }
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
