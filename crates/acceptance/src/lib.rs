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
//! ### Basic Journey with Auto-Registration
//!
//! ```rust,no_run
//! use unitycatalog_acceptance::{
//!     AcceptanceResult,
//!     journey_helpers::JourneyLogger,
//! };
//!
//! #[tokio::main]
//! async fn main() -> AcceptanceResult<()> {
//!     let logger = JourneyLogger::new("my_journey");
//!     logger.start("Testing catalog operations")?;
//!
//!     // Steps are auto-registered when executed
//!     logger.step("create_catalog", async {
//!         // Your operation here
//!         Ok::<_, String>("Catalog created")
//!     }).await?;
//!
//!     logger.step_with_description("list_catalogs", "📋 List all catalogs", async {
//!         // Your operation here
//!         Ok::<_, String>("Listed catalogs")
//!     }).await?;
//!
//!     logger.finish(true)?;
//!     Ok(())
//! }
//! ```
//!
//! ### Structured Journey with Pre-Registration
//!
//! ```rust,no_run
//! use unitycatalog_acceptance::{
//!     AcceptanceResult,
//!     journey_helpers::JourneyLogger,
//!     setup_journey_steps,
//! };
//!
//! #[tokio::main]
//! async fn main() -> AcceptanceResult<()> {
//!     let logger = JourneyLogger::new("structured_journey");
//!     logger.start("Structured catalog testing")?;
//!
//!     // Optional: Pre-register steps for planning/documentation
//!     setup_journey_steps!(
//!         logger,
//!         "create" => "Create test catalog",
//!         "verify" => "Verify catalog exists",
//!         "cleanup" => "Delete test catalog"
//!     );
//!
//!     // Execute the pre-registered steps
//!     logger.step("create", async {
//!         Ok::<_, String>("Created")
//!     }).await?;
//!
//!     logger.step("verify", async {
//!         Ok::<_, String>("Verified")
//!     }).await?;
//!
//!     logger.step("cleanup", async {
//!         Ok::<_, String>("Cleaned up")
//!     }).await?;
//!
//!     logger.finish(true)?;
//!     Ok(())
//! }
//! ```
//!
//! ## Module Organization
//!
//! - [`simple_journey`] - Core journey framework with traits and execution logic
//! - [`journeys`] - Example journey implementations

pub mod execution;
pub mod journeys;
pub mod reporting;

// Re-export commonly used types for convenience
pub use execution::{JourneyConfig, JourneyExecutionResult, JourneyExecutor, UserJourney};
pub use execution::{JourneyLogger, PerformanceMetrics, ProgressTracker, cleanup_step};
pub use reporting::{JourneyReporter, ReportingConfig, generate_journeys_summary_table};

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
