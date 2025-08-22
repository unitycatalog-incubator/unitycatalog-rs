//! Simplified Journey Framework
//!
//! This module provides a simplified approach to journey testing that:
//! - Uses manual Rust code instead of JSON configurations
//! - Leverages the actual UnityCatalogClient for type safety
//! - Records responses to numbered files for later comparison
//! - Replays recorded interactions using mock servers
//! - Focuses on user journeys rather than low-level HTTP details

use async_trait::async_trait;
use cloud_client::{CloudClient, RequestResponseInfo};
use mockito::ServerGuard;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use unitycatalog_client::UnityCatalogClient;
use url::Url;

use crate::{AcceptanceError, AcceptanceResult};

/// A user journey that can be executed against Unity Catalog
#[async_trait]
pub trait UserJourney: Send + Sync {
    /// Unique identifier for this journey
    fn name(&self) -> &str;

    /// Human-readable description of what this journey tests
    fn description(&self) -> &str;

    /// Execute the journey steps using the provided client
    async fn execute(&self, client: &UnityCatalogClient) -> AcceptanceResult<()>;

    /// Optional setup that runs before the journey
    #[allow(unused_variables)]
    async fn setup(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        Ok(())
    }

    /// Optional cleanup that runs after the journey (even on failure)
    #[allow(unused_variables)]
    async fn cleanup(&self, client: &UnityCatalogClient) -> AcceptanceResult<()> {
        Ok(())
    }

    /// Tags for organizing journeys
    fn tags(&self) -> Vec<&str> {
        vec![]
    }

    /// Save journey state for replay
    fn save_state(&self) -> AcceptanceResult<JourneyState> {
        Ok(JourneyState::empty())
    }

    /// Restore journey state from replay data
    fn load_state(&mut self, _state: &JourneyState) -> AcceptanceResult<()> {
        Ok(())
    }
}

/// Journey state that can be persisted and restored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyState {
    /// Generic key-value store for journey-specific data
    pub data: HashMap<String, serde_json::Value>,
}

impl JourneyState {
    /// Create an empty state
    pub fn empty() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Set a string value
    pub fn set_string(&mut self, key: &str, value: String) {
        self.data
            .insert(key.to_string(), serde_json::Value::String(value));
    }

    /// Get a string value
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.data.get(key)?.as_str().map(|s| s.to_string())
    }

    /// Set an integer value
    pub fn set_i64(&mut self, key: &str, value: i64) {
        self.data.insert(
            key.to_string(),
            serde_json::Value::Number(serde_json::Number::from(value)),
        );
    }

    /// Get an integer value
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.data.get(key)?.as_i64()
    }

    /// Set a boolean value
    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.data
            .insert(key.to_string(), serde_json::Value::Bool(value));
    }

    /// Get a boolean value
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key)?.as_bool()
    }

    /// Check if state is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Executes simple journeys and manages recording/replay
pub struct JourneyExecutor {
    client: UnityCatalogClient,
    _mock_server: Option<ServerGuard>,
}

impl JourneyExecutor {
    /// Create a new executor for live mode
    pub fn new(client: UnityCatalogClient) -> Self {
        Self {
            client,
            _mock_server: None,
        }
    }

    /// Create a new executor with mock server for replay mode
    pub fn new_with_mock(client: UnityCatalogClient, mock_server: ServerGuard) -> Self {
        Self {
            client,
            _mock_server: Some(mock_server),
        }
    }

    /// Read recorded interactions from a directory
    async fn read_recorded_interactions(
        recordings_dir: &PathBuf,
    ) -> AcceptanceResult<Vec<RequestResponseInfo>> {
        if !recordings_dir.exists() {
            return Err(AcceptanceError::JourneyValidation(format!(
                "Recordings directory does not exist: {}",
                recordings_dir.display()
            )));
        }

        let mut recordings = Vec::new();
        let mut entries: Vec<_> = fs::read_dir(recordings_dir)
            .map_err(|e| {
                AcceptanceError::JourneyValidation(format!(
                    "Failed to read recordings directory: {}",
                    e
                ))
            })?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "json" {
                    // Exclude journey_state.json from recordings
                    if path.file_name()? == "journey_state.json" {
                        return None;
                    }
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        // Sort by filename to ensure deterministic order
        entries.sort();

        for path in entries {
            let content = fs::read_to_string(&path).map_err(|e| {
                AcceptanceError::JourneyValidation(format!(
                    "Failed to read recording file {}: {}",
                    path.display(),
                    e
                ))
            })?;

            let recording: RequestResponseInfo = serde_json::from_str(&content).map_err(|e| {
                AcceptanceError::JourneyValidation(format!(
                    "Failed to parse recording file {}: {}",
                    path.display(),
                    e
                ))
            })?;

            recordings.push(recording);
        }

        Ok(recordings)
    }

    /// Save journey state to the recordings directory
    fn save_journey_state(recordings_dir: &PathBuf, state: &JourneyState) -> AcceptanceResult<()> {
        if state.is_empty() {
            return Ok(());
        }

        let state_file = recordings_dir.join("journey_state.json");
        let content = serde_json::to_string_pretty(state).map_err(|e| {
            AcceptanceError::Recording(format!("Failed to serialize journey state: {}", e))
        })?;

        fs::write(&state_file, content).map_err(|e| {
            AcceptanceError::Recording(format!(
                "Failed to write journey state to {}: {}",
                state_file.display(),
                e
            ))
        })?;

        Ok(())
    }

    /// Load journey state from the recordings directory
    async fn load_journey_state(recordings_dir: &PathBuf) -> AcceptanceResult<JourneyState> {
        let state_file = recordings_dir.join("journey_state.json");

        if !state_file.exists() {
            return Ok(JourneyState::empty());
        }

        let content = fs::read_to_string(&state_file).map_err(|e| {
            AcceptanceError::JourneyValidation(format!(
                "Failed to read journey state from {}: {}",
                state_file.display(),
                e
            ))
        })?;

        let state: JourneyState = serde_json::from_str(&content).map_err(|e| {
            AcceptanceError::JourneyValidation(format!(
                "Failed to parse journey state from {}: {}",
                state_file.display(),
                e
            ))
        })?;

        Ok(state)
    }

    /// Set up mock server with recorded interactions
    pub async fn setup_mock_server(
        recordings_dir: &PathBuf,
    ) -> AcceptanceResult<(ServerGuard, UnityCatalogClient)> {
        let recordings = Self::read_recorded_interactions(recordings_dir).await?;

        if recordings.is_empty() {
            return Err(AcceptanceError::JourneyValidation(
                "No recordings found for replay".to_string(),
            ));
        }

        println!(
            "🎬 Setting up mock server with {} recorded interactions from {}",
            recordings.len(),
            recordings_dir.display()
        );

        let mut server = mockito::Server::new_async().await;
        let mut mocks = Vec::new();

        // Group recordings by method and path for better mock setup
        let mut path_method_counts: HashMap<(String, String), usize> = HashMap::new();

        for recording in &recordings {
            let key = (
                recording.request.method.clone(),
                recording.request.url_path.clone(),
            );
            let count = path_method_counts.entry(key).or_insert(0);
            *count += 1;
        }

        // Set up mocks for each recording
        for recording in recordings {
            let method_str = recording.request.method.as_str();
            let path = recording.request.url_path.as_str();

            let mut mock = server
                .mock(method_str, path)
                .with_status(recording.response.status as usize);

            // Add response headers
            for (header_name, header_value) in &recording.response.headers {
                mock = mock.with_header(header_name, header_value);
            }

            // Add response body if present
            if let Some(ref body) = recording.response.body {
                mock = mock.with_body(body);
            }

            // For endpoints that might be called multiple times, allow multiple calls
            let key = (
                recording.request.method.clone(),
                recording.request.url_path.clone(),
            );
            if let Some(&count) = path_method_counts.get(&key) {
                if count > 1 {
                    mock = mock.expect_at_least(1);
                }
            }

            let created_mock = mock.create_async().await;
            mocks.push(created_mock);

            println!(
                "  📝 Mock: {} {} -> {}",
                recording.request.method, recording.request.url_path, recording.response.status
            );
        }

        // Create client pointing to mock server
        let mock_url = format!("{}/api/2.1/unity-catalog", server.url());
        let base_url: Url = mock_url.parse().map_err(|e| {
            AcceptanceError::JourneyValidation(format!("Invalid mock server URL: {}", e))
        })?;

        let cloud_client = CloudClient::new_unauthenticated();
        let client = UnityCatalogClient::new(cloud_client, base_url);

        println!("🚀 Mock server ready at: {}", server.url());

        Ok((server, client))
    }

    /// Execute a journey
    pub async fn execute_journey(
        &self,
        journey: &mut dyn UserJourney,
    ) -> AcceptanceResult<JourneyExecutionResult> {
        let start_time = std::time::Instant::now();
        let mut result = JourneyExecutionResult {
            journey_name: journey.name().to_string(),
            success: false,
            duration: std::time::Duration::default(),
            error_message: None,
            steps_completed: 0,
        };

        // Execute setup
        if let Err(e) = journey.setup(&self.client).await {
            result.error_message = Some(format!("Setup failed: {}", e));
            return Ok(result);
        }

        // Execute main journey
        let journey_result = journey.execute(&self.client).await;

        // Always run cleanup
        if let Err(cleanup_err) = journey.cleanup(&self.client).await {
            eprintln!(
                "⚠️ Cleanup failed for journey '{}': {}",
                journey.name(),
                cleanup_err
            );
        }

        result.duration = start_time.elapsed();

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
            }
        }

        Ok(result)
    }

    /// Execute multiple journeys
    pub async fn execute_journeys(
        &self,
        journeys: Vec<&mut dyn UserJourney>,
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
    /// Create client from configuration for live mode
    pub fn create_client(&self, out_dir: PathBuf) -> AcceptanceResult<UnityCatalogClient> {
        let base_url: Url = self.server_url.parse().map_err(|e| {
            AcceptanceError::JourneyValidation(format!("Invalid server URL: {}", e))
        })?;
        let base_url = base_url.join("/api/2.1/unity-catalog").unwrap();

        let mut client = if let Some(ref token) = self.auth_token {
            CloudClient::new_with_token(token)
        } else {
            CloudClient::new_unauthenticated()
        };
        if self.recording_enabled {
            std::fs::create_dir_all(&out_dir)?;
            client.set_recording_dir(out_dir)?;
        }

        Ok(UnityCatalogClient::new(client, base_url))
    }

    /// Enable/disable recording (true = live mode with recording, false = replay mode)
    pub fn with_recording(mut self, enabled: bool) -> Self {
        self.recording_enabled = enabled;
        self
    }

    /// Set output directory for recordings
    pub fn with_output_dir(mut self, dir: PathBuf) -> Self {
        self.output_dir = dir;
        self
    }

    /// Execute a journey with state management
    pub async fn execute_journey(
        &self,
        journey: &mut dyn UserJourney,
    ) -> AcceptanceResult<JourneyExecutionResult> {
        if self.recording_enabled {
            // Live mode with recording - save state after execution
            let out_dir = std::fs::canonicalize(self.output_dir.clone())?;
            let out_dir = out_dir.join(journey.name());
            let client = self.create_client(out_dir.clone())?;
            let executor = JourneyExecutor::new(client);
            let result = executor.execute_journey(journey).await?;

            // Save journey state if journey was successful
            if result.is_success() {
                let state = journey.save_state()?;
                JourneyExecutor::save_journey_state(&out_dir, &state)?;
            }

            Ok(result)
        } else {
            // Replay mode - load state and use recorded interactions
            let recordings_dir =
                std::fs::canonicalize(self.output_dir.clone())?.join(journey.name());

            println!(
                "🎬 Starting replay mode for journey '{}' from {}",
                journey.name(),
                recordings_dir.display()
            );

            // Load and apply journey state
            let state = JourneyExecutor::load_journey_state(&recordings_dir).await?;
            journey.load_state(&state)?;

            let (mock_server, client) = JourneyExecutor::setup_mock_server(&recordings_dir).await?;
            let executor = JourneyExecutor::new_with_mock(client, mock_server);

            executor.execute_journey(journey).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_mock_server_replay() {
        // Create a temporary directory with mock recordings
        let temp_dir = TempDir::new().unwrap();
        let recordings_dir = temp_dir.path().join("test_journey");
        fs::create_dir_all(&recordings_dir).unwrap();

        // Create a mock recording file
        let recording = RequestResponseInfo {
            request: cloud_client::RequestInfo {
                method: "GET".to_string(),
                url_path: "/api/2.1/unity-catalog/catalogs".to_string(),
                body: None,
            },
            response: cloud_client::ResponseInfo {
                status: 200,
                headers: {
                    let mut headers = std::collections::HashMap::new();
                    headers.insert("content-type".to_string(), "application/json".to_string());
                    headers
                },
                body: Some(r#"{"catalogs":[]}"#.to_string()),
            },
        };

        let recording_file = recordings_dir.join("000000.json");
        let file = fs::File::create(recording_file).unwrap();
        serde_json::to_writer_pretty(file, &recording).unwrap();

        // Test mock server setup
        let (_mock_server, client) = JourneyExecutor::setup_mock_server(&recordings_dir)
            .await
            .unwrap();

        // Verify that the client can make requests to the mock server
        // We'll test this by trying to list catalogs - if the mock server is working,
        // it should return our mocked empty catalogs response
        let mut catalogs_stream = client.list_catalogs(None);
        use futures::StreamExt;

        // Since we mocked an empty catalogs response, the stream should be empty
        let first_item = catalogs_stream.next().await;
        assert!(first_item.is_none());
    }

    #[tokio::test]
    async fn test_execution_mode_configuration() {
        let config = JourneyConfig::default();

        // Test default mode (replay if UC_INTEGRATION_RECORD is not set)
        assert_eq!(config.recording_enabled, false);

        // Test recording flag (live mode with recording)
        let recording_config = config.clone().with_recording(true);
        assert!(recording_config.recording_enabled);

        // Test replay mode (recording disabled)
        let replay_config = config.clone().with_recording(false);
        assert!(!replay_config.recording_enabled);
    }
}
