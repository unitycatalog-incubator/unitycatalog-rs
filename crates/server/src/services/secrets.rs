//! Secret management abstraction.
//!
//! The `SecretManager` trait lives in `unitycatalog-common` so storage backends
//! can implement it without depending on this server crate. It is re-exported
//! here to keep the historical `unitycatalog_server::services::secrets::*` paths
//! working.
pub use unitycatalog_common::services::secrets::{ProvidesSecretManager, SecretManager};
