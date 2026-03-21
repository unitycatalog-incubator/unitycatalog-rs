//! Telemetry initialization for the Unity Catalog server.

use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Initialize structured tracing for the server.
///
/// Reads the `RUST_LOG` environment variable for filter configuration.
/// Falls back to `info` level if unset.
///
/// Should be called once at process startup before any async tasks are spawned.
pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
}
