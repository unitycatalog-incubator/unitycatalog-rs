pub use handler::ExternalLocationHandler;
mod handler;
#[cfg(feature = "axum")]
pub mod server;
