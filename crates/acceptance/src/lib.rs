//! Unity Catalog Acceptance Testing Framework
//!
//! This crate provides a comprehensive framework for testing Unity Catalog workflows
//! through user journeys. It supports both mock testing for fast feedback and
//! integration testing against real Unity Catalog deployments.
//!
//! ## Features
//!
//! - **Journey-based Testing**: Define multi-step workflows in JSON format
//! - **Variable Substitution**: Pass data between steps using extracted variables
//! - **Dependency Management**: Automatic step ordering based on dependencies
//! - **Mock Server Support**: Fast testing with configurable mock responses
//! - **Response Recording**: Capture real server responses for test data
//! - **Integration Testing**: Execute against live Unity Catalog instances
//!
//! ## Quick Start
//!
//! ```rust
//! use unitycatalog_acceptance::{
//!     journey::{JourneyExecutor, JourneyLoader},
//!     mock::TestServer,
//! };
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load a journey definition
//!     let journey = JourneyLoader::load_journey("catalog_lifecycle.json")?;
//!
//!     // Set up test environment
//!     let server = TestServer::new().await;
//!     let client = server.create_client();
//!
//!     // Execute the journey
//!     let mut executor = JourneyExecutor::new(client, Some(server))
//!         .with_variables(HashMap::new());
//!
//!     let result = executor.execute_journey(journey).await;
//!     assert!(result.success);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Module Organization
//!
//! - [`journey`] - Core journey execution engine and data structures
//! - [`recorder`] - Response recording for integration testing
//! - [`mock`] - Mock server utilities and test fixtures
//! - [`models`] - Shared data models and utilities
//! - [`assertions`] - Common assertion helpers for testing

pub mod assertions;
pub mod journey;
pub mod mock;
pub mod models;
pub mod recorder;

// Re-export commonly used types for convenience
pub use assertions::TestAssertions;
pub use journey::{
    JourneyContext, JourneyExecutor, JourneyLoader, JourneyResult, JourneyStep, StepResult,
    UserJourney,
};
pub use mock::{TestDataLoader, TestServer};
pub use models::*;
pub use recorder::{JourneyRecorder, RecordedResponse, RecordingConfig};

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

    #[error("Mock server error: {0}")]
    MockServer(String),

    #[error("Recording error: {0}")]
    Recording(String),
}

/// Framework version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default timeout for HTTP requests in seconds
pub const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;

/// Default directory for journey definitions
pub const DEFAULT_JOURNEY_DIR: &str = "journeys";

/// Default directory for recorded responses
pub const DEFAULT_RECORDING_DIR: &str = "recorded";
