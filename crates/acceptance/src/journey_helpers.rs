//! Helper utilities for simplifying logging and reporting in user journeys
//!
//! This module provides simple, ergonomic functions that journey authors can use
//! to add rich logging and step tracking to their journeys without dealing
//! with the complexity of the full reporting system.
//!
//! ## Step Registration
//!
//! Steps can be handled in two ways:
//! 1. **Pre-registration** (optional): Use `setup_journey_steps!` or `add_step()` to define steps upfront
//! 2. **Auto-registration**: Steps are automatically registered when first executed
//!
//! Pre-registration is useful for planning and documentation, but not required for functionality.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::AcceptanceResult;
use crate::reporting::{JourneyReporter, ReportingConfig};

/// A thread-safe wrapper around JourneyReporter for easy use in journeys
#[derive(Clone)]
pub struct JourneyLogger {
    inner: Arc<Mutex<JourneyReporter>>,
}

impl JourneyLogger {
    /// Create a new journey logger
    pub fn new(journey_name: impl Into<String>) -> Self {
        let config = ReportingConfig::default();
        let reporter = JourneyReporter::new(journey_name, config);

        Self {
            inner: Arc::new(Mutex::new(reporter)),
        }
    }

    /// Create a journey logger with custom configuration
    pub fn with_config(journey_name: impl Into<String>, config: ReportingConfig) -> Self {
        let reporter = JourneyReporter::new(journey_name, config);

        Self {
            inner: Arc::new(Mutex::new(reporter)),
        }
    }

    /// Start the journey with a description
    pub fn start(&self, description: &str) -> AcceptanceResult<()> {
        self.inner.lock().unwrap().start_journey(description)
    }

    /// Add a step to track
    pub fn add_step(&self, id: impl Into<String>, description: impl Into<String>) {
        self.inner.lock().unwrap().add_step(id, description);
    }

    /// Log an info message
    pub fn info(&self, message: &str) -> AcceptanceResult<()> {
        self.inner.lock().unwrap().info(message)
    }

    /// Log a warning message
    pub fn warn(&self, message: &str) -> AcceptanceResult<()> {
        self.inner.lock().unwrap().warn(message)
    }

    /// Execute a step with automatic timing and status tracking
    pub async fn step<F, T, E>(&self, step_id: &str, operation: F) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let _ = self.inner.lock().unwrap().start_step(step_id);

        let result = operation.await;

        match &result {
            Ok(_) => {
                let _ = self.inner.lock().unwrap().complete_step(step_id, None);
            }
            Err(e) => {
                let _ = self
                    .inner
                    .lock()
                    .unwrap()
                    .fail_step(step_id, &e.to_string());
            }
        }

        result
    }

    /// Execute a step with custom description without pre-registration
    pub async fn step_with_description<F, T, E>(
        &self,
        step_id: &str,
        description: &str,
        operation: F,
    ) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        // Auto-register step with custom description
        self.inner.lock().unwrap().add_step(step_id, description);

        let _ = self.inner.lock().unwrap().start_step(step_id);

        let result = operation.await;

        match &result {
            Ok(_) => {
                let _ = self.inner.lock().unwrap().complete_step(step_id, None);
            }
            Err(e) => {
                let _ = self
                    .inner
                    .lock()
                    .unwrap()
                    .fail_step(step_id, &e.to_string());
            }
        }

        result
    }

    /// Execute a step with custom success details
    pub async fn step_with_details<F, T, E>(
        &self,
        step_id: &str,
        operation: F,
        success_details: impl Fn(&T) -> String,
    ) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let _ = self.inner.lock().unwrap().start_step(step_id);

        let result = operation.await;

        match &result {
            Ok(value) => {
                let details = success_details(value);
                let _ = self
                    .inner
                    .lock()
                    .unwrap()
                    .complete_step(step_id, Some(details));
            }
            Err(e) => {
                let _ = self
                    .inner
                    .lock()
                    .unwrap()
                    .fail_step(step_id, &e.to_string());
            }
        }

        result
    }

    /// Finish the journey
    pub fn finish(&self, success: bool) -> AcceptanceResult<()> {
        self.inner.lock().unwrap().finish_journey(success)
    }

    /// Generate a summary table
    pub fn summary_table(&self) -> AcceptanceResult<String> {
        self.inner.lock().unwrap().generate_summary_table()
    }
}

/// Simple progress tracker for operations that take time
pub struct ProgressTracker {
    start_time: Instant,
    name: String,
}

impl ProgressTracker {
    /// Start tracking progress for an operation
    pub fn start(name: impl Into<String>) -> Self {
        Self {
            start_time: Instant::now(),
            name: name.into(),
        }
    }

    /// Get the elapsed time
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Finish tracking and return a summary message
    pub fn finish(&self) -> String {
        format!(
            "{} completed in {}ms",
            self.name,
            self.elapsed().as_millis()
        )
    }
}

/// Convenience macros for common journey operations

/// Log a step with automatic emoji selection
#[macro_export]
macro_rules! journey_step {
    ($logger:expr, $id:expr, $description:expr, create => $operation:block) => {
        $logger
            .step($id, async {
                $logger.info(&format!("📁 Creating {}", $description))?;
                let result = $operation;
                $logger.info(&format!("✅ Created {}", $description))?;
                result
            })
            .await
    };

    ($logger:expr, $id:expr, $description:expr, list => $operation:block) => {
        $logger
            .step($id, async {
                $logger.info(&format!("📋 Listing {}", $description))?;
                let result = $operation;
                $logger.info(&format!("✅ Listed {}", $description))?;
                result
            })
            .await
    };

    ($logger:expr, $id:expr, $description:expr, get => $operation:block) => {
        $logger
            .step($id, async {
                $logger.info(&format!("🔍 Getting {}", $description))?;
                let result = $operation;
                $logger.info(&format!("✅ Retrieved {}", $description))?;
                result
            })
            .await
    };

    ($logger:expr, $id:expr, $description:expr, delete => $operation:block) => {
        $logger
            .step($id, async {
                $logger.info(&format!("🗑️ Deleting {}", $description))?;
                let result = $operation;
                $logger.info(&format!("✅ Deleted {}", $description))?;
                result
            })
            .await
    };

    ($logger:expr, $id:expr, $description:expr, verify => $operation:block) => {
        $logger
            .step($id, async {
                $logger.info(&format!("🔍 Verifying {}", $description))?;
                $operation;
                $logger.info(&format!("✅ Verified {}", $description))?;
                Ok(())
            })
            .await
    };

    ($logger:expr, $id:expr, $description:expr => $operation:block) => {
        $logger
            .step($id, async {
                $logger.info(&format!("⏳ {}", $description))?;
                let result = $operation;
                $logger.info(&format!("✅ {}", $description))?;
                result
            })
            .await
    };
}

/// Create a journey logger and initialize it
#[macro_export]
macro_rules! init_journey {
    ($name:expr, $description:expr) => {{
        let logger = $crate::journey_helpers::JourneyLogger::new($name);
        logger.start($description)?;
        logger
    }};
}

/// Execute cleanup operations with error suppression
pub async fn cleanup_step<F, T, E>(
    logger: &JourneyLogger,
    step_id: &str,
    operation: F,
) -> AcceptanceResult<()>
where
    F: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let _ = logger.inner.lock().unwrap().start_step(step_id);

    match operation.await {
        Ok(_) => {
            let _ = logger.inner.lock().unwrap().complete_step(step_id, None);
        }
        Err(e) => {
            // In cleanup, we often want to continue even if operations fail
            logger.warn(&format!("Cleanup warning: {}", e))?;
            let _ = logger
                .inner
                .lock()
                .unwrap()
                .complete_step(step_id, Some(format!("Warning: {}", e)));
        }
    }

    Ok(())
}

/// Utility for measuring operation performance
pub struct PerformanceMetrics {
    operations: Vec<(String, std::time::Duration)>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub async fn measure<F, T>(&mut self, name: &str, operation: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let start = Instant::now();
        let result = operation.await;
        let duration = start.elapsed();

        self.operations.push((name.to_string(), duration));
        result
    }

    pub fn summary(&self) -> String {
        if self.operations.is_empty() {
            return "No operations measured".to_string();
        }

        let total: std::time::Duration = self.operations.iter().map(|(_, d)| *d).sum();
        let mut summary = format!("Performance Summary (Total: {}ms):\n", total.as_millis());

        for (name, duration) in &self.operations {
            let percentage = (duration.as_nanos() as f64 / total.as_nanos() as f64) * 100.0;
            summary.push_str(&format!(
                "  • {}: {}ms ({:.1}%)\n",
                name,
                duration.as_millis(),
                percentage
            ));
        }

        summary
    }

    pub fn operations(&self) -> &[(String, std::time::Duration)] {
        &self.operations
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_journey_logger_basic_flow() {
        let logger = JourneyLogger::new("test_journey");

        // This should not panic
        let _ = logger.start("Test journey description");
        logger.add_step("step1", "Test step");

        let result = logger
            .step("step1", async { Ok::<_, String>("success") })
            .await;

        assert!(result.is_ok());
        let _ = logger.finish(true);
    }

    #[tokio::test]
    async fn test_progress_tracker() {
        let tracker = ProgressTracker::start("test_operation");

        // Simulate some work
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let elapsed = tracker.elapsed();
        assert!(elapsed.as_millis() >= 10);

        let summary = tracker.finish();
        assert!(summary.contains("test_operation"));
        assert!(summary.contains("completed"));
    }

    #[tokio::test]
    async fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::new();

        let result = metrics
            .measure("test_op", async {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                42
            })
            .await;

        assert_eq!(result, 42);
        assert_eq!(metrics.operations().len(), 1);
        assert_eq!(metrics.operations()[0].0, "test_op");

        let summary = metrics.summary();
        assert!(summary.contains("Performance Summary"));
        assert!(summary.contains("test_op"));
    }

    #[test]
    fn test_performance_metrics_empty() {
        let metrics = PerformanceMetrics::new();
        assert_eq!(metrics.summary(), "No operations measured");
    }

    #[tokio::test]
    async fn test_auto_registration_of_steps() {
        let logger = JourneyLogger::new("auto_registration_test");
        let _ = logger.start("Testing auto-registration of steps");

        // Execute a step without pre-registration - should auto-register
        let result = logger
            .step("auto_step", async { Ok::<_, String>("success") })
            .await;

        assert!(result.is_ok());

        // Execute step with custom description
        let result2 = logger
            .step_with_description("custom_step", "Custom description", async {
                Ok::<_, String>("custom success")
            })
            .await;

        assert!(result2.is_ok());
        let _ = logger.finish(true);
    }
}
