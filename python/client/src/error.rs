use pyo3::exceptions::{PyFileNotFoundError, PyIOError, PyNotImplementedError, PyValueError};
use pyo3::prelude::*;
use pyo3::{create_exception, DowncastError};
use thiserror::Error;
use unitycatalog_common::error::Error as UCError;

/// A type wrapper around `Result<T, PyObjectStoreError>`.
pub type PyUnityCatalogResult<T> = Result<T, PyUnityCatalogError>;

// Base exception
create_exception!(
    unitycatalog_common,
    UnityCatalogError,
    pyo3::exceptions::PyException,
    "The base Python-facing exception from which all other errors subclass."
);

// Subclasses from base exception
create_exception!(
    pyo3_object_store,
    GenericError,
    UnityCatalogError,
    "A Python-facing exception wrapping [object_store::Error::Generic]."
);

/// The Error variants returned by this crate.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum PyUnityCatalogError {
    /// A wrapped [object_store::Error]
    #[error(transparent)]
    UnityCatalogError(#[from] unitycatalog_common::error::Error),

    /// A wrapped [PyErr]
    #[error(transparent)]
    PyErr(#[from] PyErr),

    /// A wrapped [std::io::Error]
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

impl From<PyUnityCatalogError> for PyErr {
    fn from(error: PyUnityCatalogError) -> Self {
        match error {
            PyUnityCatalogError::PyErr(err) => err,
            PyUnityCatalogError::UnityCatalogError(ref err) => match err {
                UCError::Generic(msg) => GenericError::new_err(print_with_debug(err)),
                _ => GenericError::new_err(print_with_debug(err)),
            },
            PyUnityCatalogError::IOError(err) => PyIOError::new_err(err),
        }
    }
}

fn print_with_debug(err: &UCError) -> String {
    // #? gives "pretty-printing" for debug
    // https://doc.rust-lang.org/std/fmt/trait.Debug.html
    format!("{err}\n\nDebug source:\n{err:#?}")
}
