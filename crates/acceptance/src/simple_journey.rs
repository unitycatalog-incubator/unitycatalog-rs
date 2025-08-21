//! Simplified Journey Framework
//!
//! This module provides a simplified approach to journey testing that:
//! - Uses manual Rust code instead of JSON configurations
//! - Leverages the actual UnityCatalogClient for type safety
//! - Records responses to numbered files for later comparison
//! - Focuses on user journeys rather than low-level HTTP details

use async_trait::async_trait;
use serde_json::Value;
use url::Url;

use std::path::{Path, PathBuf};
use tokio::fs;
use unitycatalog_client::UnityCatalogClient;

use crate::{AcceptanceError, AcceptanceResult};

/// A user journey that can be executed against Unity Catalog
#[async_trait]
pub trait UserJourney: Send + Sync {
    /// Unique identifier for this journey
    fn name(&self) -> &str;

    /// Human-readable description of what this journey tests
    fn description(&self) -> &str;

    /// Execute the journey steps using the provided client
    async fn execute(
        &self,
        client: &UnityCatalogClient,
        recorder: &mut JourneyRecorder,
    ) -> AcceptanceResult<()>;

    /// Optional setup that runs before the journey
    #[allow(unused_variables)]
    async fn setup(
        &self,
        client: &UnityCatalogClient,
        recorder: &mut JourneyRecorder,
    ) -> AcceptanceResult<()> {
        Ok(())
    }

    /// Optional cleanup that runs after the journey (even on failure)
    #[allow(unused_variables)]
    async fn cleanup(
        &self,
        client: &UnityCatalogClient,
        recorder: &mut JourneyRecorder,
    ) -> AcceptanceResult<()> {
        Ok(())
    }

    /// Tags for organizing journeys
    fn tags(&self) -> Vec<&str> {
        vec![]
    }
}

/// Records responses during journey execution
pub struct JourneyRecorder {
    journey_name: String,
    output_dir: PathBuf,
    step_counter: usize,
    recorded_responses: Vec<RecordedStep>,
}

/// A recorded step with its response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecordedStep {
    pub step_number: usize,
    pub step_name: String,
    pub description: String,
    pub response: Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl JourneyRecorder {
    /// Create a new recorder for a journey
    pub fn new(journey_name: impl Into<String>, output_dir: impl AsRef<Path>) -> Self {
        Self {
            journey_name: journey_name.into(),
            output_dir: output_dir.as_ref().to_path_buf(),
            step_counter: 0,
            recorded_responses: Vec::new(),
        }
    }

    /// Record a response from a journey step
    pub async fn record_step<T: serde::Serialize>(
        &mut self,
        step_name: impl Into<String>,
        description: impl Into<String>,
        response: &T,
    ) -> AcceptanceResult<()> {
        self.step_counter += 1;

        let response_value = serde_json::to_value(response).map_err(|e| {
            AcceptanceError::Recording(format!("Failed to serialize response: {}", e))
        })?;

        let recorded_step = RecordedStep {
            step_number: self.step_counter,
            step_name: step_name.into(),
            description: description.into(),
            response: response_value.clone(),
            timestamp: chrono::Utc::now(),
        };

        self.recorded_responses.push(recorded_step.clone());

        // Write individual step file
        let step_file = self.output_dir.join(&self.journey_name).join(format!(
            "{:03}_{}.json",
            self.step_counter, recorded_step.step_name
        ));

        if let Some(parent) = step_file.parent() {
            fs::create_dir_all(parent).await?;
        }

        let pretty_json = serde_json::to_string_pretty(&response_value)?;
        fs::write(&step_file, pretty_json).await?;

        println!(
            "📝 Recorded step {}: {} -> {}",
            self.step_counter,
            recorded_step.step_name,
            step_file.display()
        );

        Ok(())
    }

    /// Record an error that occurred during a step
    pub async fn record_error(
        &mut self,
        step_name: impl Into<String>,
        description: impl Into<String>,
        error: &(dyn std::error::Error + Send + Sync),
    ) -> AcceptanceResult<()> {
        let error_response = serde_json::json!({
            "error": true,
            "message": error.to_string(),
            "type": std::any::type_name_of_val(error)
        });

        self.record_step(step_name, description, &error_response)
            .await
    }

    /// Finalize recording and write summary
    pub async fn finalize(&self) -> AcceptanceResult<()> {
        let summary_file = self
            .output_dir
            .join(&self.journey_name)
            .join("journey_summary.json");

        let summary = serde_json::json!({
            "journey_name": self.journey_name,
            "total_steps": self.recorded_responses.len(),
            "recorded_at": chrono::Utc::now(),
            "steps": self.recorded_responses
        });

        if let Some(parent) = summary_file.parent() {
            fs::create_dir_all(parent).await?;
        }

        let pretty_json = serde_json::to_string_pretty(&summary)?;
        fs::write(&summary_file, pretty_json).await?;

        println!("📋 Journey summary written to {}", summary_file.display());

        Ok(())
    }
}

/// Executes simple journeys and manages recording
pub struct JourneyExecutor {
    client: UnityCatalogClient,
    recording_enabled: bool,
    output_dir: PathBuf,
}

impl JourneyExecutor {
    /// Create a new executor
    pub fn new(client: UnityCatalogClient, output_dir: impl AsRef<Path>) -> Self {
        Self {
            client,
            recording_enabled: true,
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }

    /// Enable or disable response recording
    pub fn with_recording(mut self, enabled: bool) -> Self {
        self.recording_enabled = enabled;
        self
    }

    /// Execute a journey
    pub async fn execute_journey(
        &self,
        journey: &dyn UserJourney,
    ) -> AcceptanceResult<JourneyExecutionResult> {
        let mut recorder = if self.recording_enabled {
            JourneyRecorder::new(journey.name(), &self.output_dir)
        } else {
            JourneyRecorder::new(journey.name(), "/dev/null") // Dummy recorder
        };

        let start_time = std::time::Instant::now();
        let mut result = JourneyExecutionResult {
            journey_name: journey.name().to_string(),
            success: false,
            duration: std::time::Duration::default(),
            error_message: None,
            steps_completed: 0,
        };

        // Execute setup
        if let Err(e) = journey.setup(&self.client, &mut recorder).await {
            result.error_message = Some(format!("Setup failed: {}", e));
            if self.recording_enabled {
                let _ = recorder.record_error("setup", "Journey setup", &e).await;
                let _ = recorder.finalize().await;
            }
            return Ok(result);
        }

        // Execute main journey
        let journey_result = journey.execute(&self.client, &mut recorder).await;

        // Always run cleanup
        if let Err(cleanup_err) = journey.cleanup(&self.client, &mut recorder).await {
            eprintln!(
                "⚠️ Cleanup failed for journey '{}': {}",
                journey.name(),
                cleanup_err
            );
            if self.recording_enabled {
                let _ = recorder
                    .record_error("cleanup", "Journey cleanup", &cleanup_err)
                    .await;
            }
        }

        result.duration = start_time.elapsed();
        result.steps_completed = recorder.step_counter;

        match journey_result {
            Ok(()) => {
                result.success = true;
                println!(
                    "✅ Journey '{}' completed successfully in {:?}",
                    journey.name(),
                    result.duration
                );
            }
            Err(e) => {
                result.error_message = Some(e.to_string());
                println!(
                    "❌ Journey '{}' failed after {:?}: {}",
                    journey.name(),
                    result.duration,
                    e
                );
                if self.recording_enabled {
                    let _ = recorder
                        .record_error("journey_failure", "Journey execution failed", &e)
                        .await;
                }
            }
        }

        // Finalize recording
        if self.recording_enabled {
            if let Err(e) = recorder.finalize().await {
                eprintln!("⚠️ Failed to finalize recording: {}", e);
            }
        }

        Ok(result)
    }

    /// Execute multiple journeys
    pub async fn execute_journeys(
        &self,
        journeys: Vec<&dyn UserJourney>,
    ) -> AcceptanceResult<Vec<JourneyExecutionResult>> {
        let mut results = Vec::new();

        for journey in journeys {
            let result = self.execute_journey(journey).await?;
            results.push(result);

            // Small delay between journeys to avoid overwhelming the server
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        Ok(results)
    }
}

/// Result of executing a journey
#[derive(Debug, Clone)]
pub struct JourneyExecutionResult {
    pub journey_name: String,
    pub success: bool,
    pub duration: std::time::Duration,
    pub error_message: Option<String>,
    pub steps_completed: usize,
}

impl JourneyExecutionResult {
    /// Check if the journey was successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get a summary string for reporting
    pub fn summary(&self) -> String {
        if self.success {
            format!(
                "✅ {} ({} steps, {:?})",
                self.journey_name, self.steps_completed, self.duration
            )
        } else {
            format!(
                "❌ {} ({} steps, {:?}) - {}",
                self.journey_name,
                self.steps_completed,
                self.duration,
                self.error_message.as_deref().unwrap_or("Unknown error")
            )
        }
    }
}

/// Configuration for journey execution
#[derive(Debug, Clone)]
pub struct JourneyConfig {
    pub recording_enabled: bool,
    pub output_dir: PathBuf,
    pub server_url: String,
    pub auth_token: Option<String>,
    pub timeout_seconds: u64,
    pub storage_root: String,
}

impl Default for JourneyConfig {
    fn default() -> Self {
        Self {
            recording_enabled: std::env::var("UC_INTEGRATION_RECORD").unwrap_or_default() == "true",
            output_dir: std::env::var("UC_INTEGRATION_DIR")
                .unwrap_or_else(|_| "test_data/recordings".to_string())
                .into(),
            server_url: std::env::var("UC_INTEGRATION_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            auth_token: std::env::var("UC_INTEGRATION_TOKEN").ok(),
            timeout_seconds: std::env::var("REQUEST_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            storage_root: std::env::var("UC_INTEGRATION_STORAGE_ROOT")
                .unwrap_or_else(|_| "s3://open-lakehouse-dev/".to_string()),
        }
    }
}

impl JourneyConfig {
    /// Create client from configuration
    pub fn create_client(&self) -> AcceptanceResult<UnityCatalogClient> {
        let base_url: Url = self.server_url.parse().map_err(|e| {
            AcceptanceError::JourneyValidation(format!("Invalid server URL: {}", e))
        })?;
        let base_url = base_url.join("/api/2.1/unity-catalog").unwrap();

        let client = if let Some(ref token) = self.auth_token {
            UnityCatalogClient::new_with_token(base_url, token)
        } else {
            UnityCatalogClient::new_unauthenticated(base_url)
        };

        Ok(client)
    }

    /// Create executor from configuration
    pub fn create_executor(&self) -> AcceptanceResult<JourneyExecutor> {
        let client = self.create_client()?;
        let executor =
            JourneyExecutor::new(client, &self.output_dir).with_recording(self.recording_enabled);
        Ok(executor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    struct TestJourney;

    #[async_trait]
    impl UserJourney for TestJourney {
        fn name(&self) -> &str {
            "test_journey"
        }

        fn description(&self) -> &str {
            "A test journey for unit testing"
        }

        async fn execute(
            &self,
            _client: &UnityCatalogClient,
            recorder: &mut JourneyRecorder,
        ) -> AcceptanceResult<()> {
            recorder
                .record_step(
                    "test_step",
                    "Test step description",
                    &serde_json::json!({"test": "data"}),
                )
                .await?;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_journey_recorder() {
        let temp_dir = TempDir::new().unwrap();
        let mut recorder = JourneyRecorder::new("test_journey", temp_dir.path());

        let test_data = serde_json::json!({"key": "value"});
        recorder
            .record_step("test_step", "Test description", &test_data)
            .await
            .unwrap();
        recorder.finalize().await.unwrap();

        // Check that files were created
        let step_file = temp_dir
            .path()
            .join("test_journey")
            .join("001_test_step.json");
        let summary_file = temp_dir
            .path()
            .join("test_journey")
            .join("journey_summary.json");

        assert!(step_file.exists());
        assert!(summary_file.exists());
    }
}
