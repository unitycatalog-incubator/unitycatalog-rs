pub use handler::TableHandler;
mod handler;
#[cfg(feature = "axum")]
pub mod server;
