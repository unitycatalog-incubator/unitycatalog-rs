// The generated client/extractor code refers to this crate by its external name
// (`unitycatalog_sharing_client::...`); alias `self` so those paths resolve from
// within the crate.
extern crate self as unitycatalog_sharing_client;

pub use crate::error::{Error, Result};

pub mod client;
mod codegen;
pub mod error;
pub mod models;
mod utils;

// The generated axum request extractors live under `codegen/extractors` (a
// distinct dir from the client code so their per-service `mod.rs` files don't
// collide). They are wired in here because the generated `codegen/mod.rs` only
// lists the client service modules.
#[cfg(feature = "axum")]
#[path = "codegen/extractors/mod.rs"]
pub mod extractors;

// Hand-written extractors for the NDJSON query path (not part of the generated
// service). See [`query_extractors`].
#[cfg(feature = "axum")]
mod query_extractors;
