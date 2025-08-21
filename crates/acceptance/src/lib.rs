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
//! ```rust
//! use unitycatalog_acceptance::{
//!     AcceptanceResult,
//!     simple_journey::{JourneyConfig, SimpleJourney, SimpleJourneyExecutor},
//! };
//! use async_trait::async_trait;
//!
//! struct MyJourney;
//!
//! #[async_trait]
//! impl SimpleJourney for MyJourney {
//!     fn name(&self) -> &str { "my_test_journey" }
//!     fn description(&self) -> &str { "Tests catalog operations" }
//!
//!     async fn execute(
//!         &self,
//!         client: &unitycatalog_client::UnityCatalogClient,
//!         recorder: &mut unitycatalog_acceptance::simple_journey::JourneyRecorder,
//!     ) -> AcceptanceResult<()> {
//!         // Use the actual client to perform operations
//!         let catalogs = client.list_catalogs(None, None).await?;
//!         recorder.record_step("list_catalogs", "List all catalogs", &catalogs).await?;
//!         Ok(())
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> AcceptanceResult<()> {
//!     let config = JourneyConfig::default();
//!     let executor = config.create_executor()?;
//!     let journey = MyJourney;
//!
//!     let result = executor.execute_journey(&journey).await?;
//!     assert!(result.is_success());
//!     Ok(())
//! }
//! ```
//!
//! ## Module Organization
//!
//! - [`simple_journey`] - Core journey framework with traits and execution logic
//! - [`journeys`] - Example journey implementations

pub mod journeys;
pub mod simple_journey;

// Re-export commonly used types for convenience
pub use simple_journey::{
    JourneyConfig, JourneyExecutionResult, JourneyExecutor, JourneyRecorder, UserJourney,
};

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

/// Default timeout for HTTP requests in seconds
pub const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;

/// Default directory for recorded responses
pub const DEFAULT_RECORDING_DIR: &str = "test_data/recordings";
