//! Unity Catalog Acceptance Testing Framework
//!
//! This crate provides a simplified framework for testing Unity Catalog workflows
//! through user journeys written in Rust. The framework emphasizes type safety,
//! ease of use, and automatic response recording.
//!
//! ## Features
//!
//! - **Type-safe Journeys**: Write journeys in Rust using the actual UnityCatalogClient
//! - **Automatic Recording**: Capture server responses for later comparison and mocking
//! - **Simple API**: Clean trait-based interface for defining test workflows
//! - **Integration Testing**: Execute against live Unity Catalog instances
//! - **Organized Output**: Responses saved as numbered files for easy review
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use unitycatalog_acceptance::{
//!     AcceptanceResult,
//!     simple_journey::{JourneyConfig, UserJourney},
//! };
//! use async_trait::async_trait;
//! use futures::StreamExt;
//!
//! struct MyJourney;
//!
//! #[async_trait]
//! impl UserJourney for MyJourney {
//!     fn name(&self) -> &str { "my_test_journey" }
//!     fn description(&self) -> &str { "Tests catalog operations" }
//!
//!     async fn execute(
//!         &self,
//!         client: &unitycatalog_client::UnityCatalogClient,
//!     ) -> AcceptanceResult<()> {
//!         // Use the actual client to perform operations
//!         let mut catalogs = client.list_catalogs(None);
//!         while let Some(catalog) = catalogs.next().await {
//!             let _catalog = catalog?;
//!             // Process catalog...
//!         }
//!         Ok(())
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> AcceptanceResult<()> {
//!     let config = JourneyConfig::default();
//!     let mut journey = MyJourney;
//!
//!     let result = config.execute_journey_with_state(&mut journey).await?;
//!     assert!(result.is_success());
//!     Ok(())
//! }
//! ```
//!
//! ## Module Organization
//!
//! - [`simple_journey`] - Core journey framework with traits and execution logic
//! - [`journeys`] - Example journey implementations

pub mod journey;
pub mod journeys;

// Re-export commonly used types for convenience
pub use journey::{JourneyConfig, JourneyExecutionResult, JourneyExecutor, UserJourney};

/// Result type commonly used throughout the framework
pub type AcceptanceResult<T> = Result<T, AcceptanceError>;

/// Common error types for the acceptance framework
#[derive(Debug, thiserror::Error)]
pub enum AcceptanceError {
    #[error("Journey execution failed: {0}")]
    JourneyExecution(String),

    #[error("Journey validation failed: {0}")]
    JourneyValidation(String),

    #[error("Step execution failed: {step_id}: {message}")]
    StepExecution { step_id: String, message: String },

    #[error("Variable substitution failed: {0}")]
    VariableSubstitution(String),

    #[error("JSON parsing error: {0}")]
    JsonParsing(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP client error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Unity Catalog client error: {0}")]
    UnityCatalog(String),

    #[error("Recording error: {0}")]
    Recording(String),
}

impl From<unitycatalog_client::Error> for AcceptanceError {
    fn from(err: unitycatalog_client::Error) -> Self {
        AcceptanceError::UnityCatalog(err.to_string())
    }
}

/// Framework version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
