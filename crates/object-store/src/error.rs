#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Unity Catalog error: {source}")]
    UnityCatalogError {
        #[from]
        source: unitycatalog_common::Error,
    },

    #[error("The unity API response did not contain a credential")]
    NoCredential,

    #[error("Credential mismatch: {0}")]
    CredentialMismatch(String),

    #[error("Invalid credential: {0}")]
    InvalidCredential(String),
}

impl Error {
    pub fn invalid_config(msg: impl ToString) -> Self {
        Error::InvalidConfig(msg.to_string())
    }

    pub fn credential_mismatch(msg: impl ToString) -> Self {
        Error::CredentialMismatch(msg.to_string())
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Error::InvalidUrl(err.to_string())
    }
}

impl From<Error> for object_store::Error {
    fn from(err: Error) -> Self {
        object_store::Error::Generic {
            store: "unity",
            source: Box::new(err),
        }
    }
}
