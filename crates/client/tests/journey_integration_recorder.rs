//! Journey Integration Test Recorder
//!
//! This module provides utilities for recording real Unity Catalog server responses
//! during journey execution. It can be used to capture actual API responses and
//! update journey files with real data for more accurate testing.

use reqwest::{Client, Method};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use url::Url;

mod test_utils;
use test_utils::journeys::{JourneyLoader, JourneyStep, UserJourney};

/// Configuration for integration test recording
#[derive(Debug, Clone)]
pub struct RecorderConfig {
    /// Unity Catalog server URL
    pub server_url: String,

    /// Authentication token (if required)
    pub auth_token: Option<String>,

    /// Directory to save recorded responses
    pub output_dir: String,

    /// Whether to overwrite existing journey files
    pub overwrite_existing: bool,

    /// HTTP client timeout in seconds
    pub timeout_seconds: u64,

    /// Whether to record only successful responses
    pub record_success_only: bool,
}

impl Default for RecorderConfig {
    fn default() -> Self {
        Self {
            server_url: env::var("UC_SERVER_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            auth_token: env::var("UC_AUTH_TOKEN").ok(),
            output_dir: "tests/test_data/journeys/recorded".to_string(),
            overwrite_existing: env::var("OVERWRITE_JOURNEY_RESPONSES")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            timeout_seconds: 30,
            record_success_only: env::var("RECORD_SUCCESS_ONLY")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
        }
    }
}

/// Journey recorder that captures real server responses
pub struct JourneyRecorder {
    config: RecorderConfig,
    http_client: Client,
    base_url: Url,
}

impl JourneyRecorder {
    /// Create a new journey recorder with the given configuration
    pub fn new(config: RecorderConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()?;

        let base_url = Url::parse(&config.server_url)?;

        // Ensure output directory exists
        fs::create_dir_all(&config.output_dir)?;

        Ok(Self {
            config,
            http_client,
            base_url,
        })
    }

    /// Create a recorder with default configuration from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Self::new(RecorderConfig::default())
    }

    /// Record responses for a journey and save updated journey file
    pub async fn record_journey(
        &self,
        journey_name: &str,
    ) -> Result<UserJourney, Box<dyn std::error::Error>> {
        // Load the original journey
        let mut journey = JourneyLoader::load_journey(journey_name)?;

        println!("Recording journey: {}", journey.name);
        println!("Server URL: {}", self.config.server_url);

        // Execute journey and record responses
        let recorded_journey = self.execute_and_record(&mut journey).await?;

        // Save the updated journey
        self.save_recorded_journey(&recorded_journey, journey_name)?;

        Ok(recorded_journey)
    }

    /// Execute all journeys in the journeys directory and record responses
    pub async fn record_all_journeys(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let journeys = JourneyLoader::load_all_journeys()?;
        let mut recorded_files = Vec::new();

        for journey in journeys {
            let journey_file = format!("{}.json", journey.name);

            match self.record_journey(&journey_file).await {
                Ok(_) => {
                    recorded_files.push(journey_file.clone());
                    println!("Successfully recorded: {}", journey_file);
                }
                Err(e) => {
                    eprintln!("Failed to record {}: {}", journey_file, e);
                }
            }
        }

        Ok(recorded_files)
    }

    /// Execute journey steps and capture actual responses
    async fn execute_and_record(
        &self,
        journey: &mut UserJourney,
    ) -> Result<UserJourney, Box<dyn std::error::Error>> {
        let mut context = self.create_execution_context(journey);
        let mut updated_steps = Vec::new();

        for step in &journey.steps {
            // Check dependencies (simplified - assume previous steps succeeded)
            if let Some(_deps) = &step.depends_on {
                // In a full implementation, we would check if dependencies were satisfied
                // For now, we continue with execution
            }

            // Execute the step and record response
            match self.execute_step_and_record(step, &mut context).await {
                Ok(updated_step) => {
                    updated_steps.push(updated_step);
                }
                Err(e) => {
                    eprintln!("Failed to execute step '{}': {}", step.id, e);

                    if !step.continue_on_failure.unwrap_or(false) {
                        return Err(e);
                    }

                    // Use original step if recording failed
                    updated_steps.push(step.clone());
                }
            }
        }

        // Create updated journey with recorded responses
        let mut recorded_journey = journey.clone();
        recorded_journey.steps = updated_steps;

        // Add recording metadata
        if recorded_journey.metadata.is_none() {
            recorded_journey.metadata = Some(Map::new());
        }

        if let Some(ref mut metadata) = recorded_journey.metadata {
            metadata.insert(
                "recorded_at".to_string(),
                Value::String(chrono::Utc::now().to_rfc3339()),
            );
            metadata.insert(
                "recorded_from".to_string(),
                Value::String(self.config.server_url.clone()),
            );
            metadata.insert(
                "recording_tool".to_string(),
                Value::String("unity_catalog_journey_recorder".to_string()),
            );
        }

        Ok(recorded_journey)
    }

    /// Execute a single step and record the response
    async fn execute_step_and_record(
        &self,
        step: &JourneyStep,
        context: &mut HashMap<String, Value>,
    ) -> Result<JourneyStep, Box<dyn std::error::Error>> {
        println!("Recording step: {} - {}", step.id, step.description);

        // Substitute variables in path and request body
        let path = self.substitute_variables(&step.path, context);
        let request_body = step
            .request_body
            .as_ref()
            .map(|body| self.substitute_variables_in_json(body, context));

        // Build the full URL
        let full_url = self.base_url.join(&path.trim_start_matches('/'))?;

        // Create HTTP request
        let method = Method::from_bytes(step.method.as_bytes())?;
        let mut request_builder = self.http_client.request(method, full_url);

        // Add authentication if configured
        if let Some(ref token) = self.config.auth_token {
            request_builder = request_builder.bearer_auth(token);
        }

        // Add request body if present
        if let Some(ref body) = request_body {
            request_builder = request_builder
                .header("Content-Type", "application/json")
                .json(body);
        }

        // Execute the request
        let response = request_builder.send().await?;
        let status_code = response.status().as_u16();

        // Read response body
        let response_text = response.text().await?;
        let response_json: Value = if response_text.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&response_text).unwrap_or_else(|_| Value::String(response_text))
        };

        println!("  Status: {}", status_code);

        // Extract variables from response if configured
        if let Some(ref extract_rules) = step.extract_variables {
            for (var_name, json_path) in extract_rules {
                if let Some(extracted_value) =
                    self.simple_json_path_extract(&response_json, json_path)
                {
                    context.insert(var_name.clone(), extracted_value);
                }
            }
        }

        // Create updated step with recorded response
        let mut updated_step = step.clone();

        // Update expected status and response if we're recording all responses
        // or if it's a successful response and we're only recording success
        let should_record = if self.config.record_success_only {
            status_code >= 200 && status_code < 300
        } else {
            true
        };

        if should_record {
            updated_step.expected_status = status_code;
            updated_step.expected_response = if response_json == Value::Null {
                None
            } else {
                Some(response_json)
            };
        }

        Ok(updated_step)
    }

    /// Create execution context with initial variables
    fn create_execution_context(&self, journey: &UserJourney) -> HashMap<String, Value> {
        let mut context = HashMap::new();

        // Add journey variables
        if let Some(ref variables) = journey.variables {
            for (key, value) in variables {
                context.insert(key.clone(), value.clone());
            }
        }

        // Add some default variables for recording
        context.insert(
            "timestamp".to_string(),
            Value::Number(serde_json::Number::from(chrono::Utc::now().timestamp())),
        );

        context.insert(
            "recording_session".to_string(),
            Value::String(uuid::Uuid::new_v4().to_string()),
        );

        context
    }

    /// Substitute variables in a string template
    fn substitute_variables(&self, template: &str, context: &HashMap<String, Value>) -> String {
        let mut result = template.to_string();

        for (key, value) in context {
            let placeholder = format!("{{{}}}", key);
            let replacement = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => value.to_string().trim_matches('"').to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }

        result
    }

    /// Substitute variables in a JSON value
    fn substitute_variables_in_json(
        &self,
        json: &Value,
        context: &HashMap<String, Value>,
    ) -> Value {
        match json {
            Value::String(s) => Value::String(self.substitute_variables(s, context)),
            Value::Object(obj) => {
                let mut new_obj = Map::new();
                for (key, value) in obj {
                    new_obj.insert(
                        key.clone(),
                        self.substitute_variables_in_json(value, context),
                    );
                }
                Value::Object(new_obj)
            }
            Value::Array(arr) => Value::Array(
                arr.iter()
                    .map(|v| self.substitute_variables_in_json(v, context))
                    .collect(),
            ),
            _ => json.clone(),
        }
    }

    /// Simple JSONPath-like extraction
    fn simple_json_path_extract(&self, json: &Value, path: &str) -> Option<Value> {
        if path.starts_with("$.") {
            let path_parts: Vec<&str> = path[2..].split('.').collect();
            let mut current = json;

            for part in path_parts {
                match current {
                    Value::Object(obj) => {
                        current = obj.get(part)?;
                    }
                    _ => return None,
                }
            }

            Some(current.clone())
        } else {
            None
        }
    }

    /// Save recorded journey to file
    fn save_recorded_journey(
        &self,
        journey: &UserJourney,
        original_filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = Path::new(&self.config.output_dir).join(original_filename);

        // Check if file exists and we shouldn't overwrite
        if output_path.exists() && !self.config.overwrite_existing {
            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            let stem = output_path.file_stem().unwrap().to_str().unwrap();
            let extension = output_path.extension().unwrap().to_str().unwrap();
            let new_filename = format!("{}_{}.{}", stem, timestamp, extension);
            let output_path = Path::new(&self.config.output_dir).join(new_filename);

            println!("Saving recorded journey to: {}", output_path.display());
            let json_content = serde_json::to_string_pretty(journey)?;
            fs::write(output_path, json_content)?;
        } else {
            println!("Saving recorded journey to: {}", output_path.display());
            let json_content = serde_json::to_string_pretty(journey)?;
            fs::write(output_path, json_content)?;
        }

        Ok(())
    }
}

/// Utility functions for recording management
pub struct RecordingUtils;

impl RecordingUtils {
    /// Check if recording is enabled via environment variables
    pub fn is_recording_enabled() -> bool {
        env::var("RECORD_JOURNEY_RESPONSES")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false)
    }

    /// Get recording configuration from environment
    pub fn get_recording_config() -> Option<RecorderConfig> {
        if Self::is_recording_enabled() {
            Some(RecorderConfig::default())
        } else {
            None
        }
    }

    /// Record all journeys if recording is enabled
    pub async fn record_if_enabled() -> Result<(), Box<dyn std::error::Error>> {
        if Self::is_recording_enabled() {
            println!("Journey recording is enabled, recording all journeys...");

            let recorder = JourneyRecorder::from_env()?;
            let recorded_files = recorder.record_all_journeys().await?;

            println!("Recorded {} journey files:", recorded_files.len());
            for file in recorded_files {
                println!("  - {}", file);
            }
        } else {
            println!("Journey recording is disabled. Set RECORD_JOURNEY_RESPONSES=true to enable.");
        }

        Ok(())
    }

    /// Clean up old recorded files
    pub fn cleanup_old_recordings(days_old: u64) -> Result<(), Box<dyn std::error::Error>> {
        let config = RecorderConfig::default();
        let recorded_dir = Path::new(&config.output_dir);

        if !recorded_dir.exists() {
            return Ok(());
        }

        let cutoff_time =
            std::time::SystemTime::now() - std::time::Duration::from_secs(days_old * 24 * 60 * 60);

        for entry in fs::read_dir(recorded_dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if metadata.is_file() && metadata.modified()? < cutoff_time {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.contains("_20") && filename.ends_with(".json") {
                        println!("Cleaning up old recording: {}", filename);
                        fs::remove_file(entry.path())?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_variable_substitution() {
        let recorder = JourneyRecorder::new(RecorderConfig::default()).unwrap();

        let mut context = HashMap::new();
        context.insert("catalog_name".to_string(), json!("test_catalog"));
        context.insert("user_id".to_string(), json!(12345));

        let template = "/catalogs/{catalog_name}/users/{user_id}";
        let result = recorder.substitute_variables(template, &context);

        assert_eq!(result, "/catalogs/test_catalog/users/12345");
    }

    #[test]
    fn test_json_variable_substitution() {
        let recorder = JourneyRecorder::new(RecorderConfig::default()).unwrap();

        let mut context = HashMap::new();
        context.insert("name".to_string(), json!("test_catalog"));
        context.insert("environment".to_string(), json!("production"));

        let json_template = json!({
            "name": "{name}",
            "properties": {
                "env": "{environment}"
            }
        });

        let result = recorder.substitute_variables_in_json(&json_template, &context);

        assert_eq!(
            result,
            json!({
                "name": "test_catalog",
                "properties": {
                    "env": "production"
                }
            })
        );
    }

    #[test]
    fn test_json_path_extraction() {
        let recorder = JourneyRecorder::new(RecorderConfig::default()).unwrap();

        let response = json!({
            "name": "test_catalog",
            "metadata": {
                "id": "12345",
                "created_at": 1699564800000i64
            }
        });

        let name = recorder.simple_json_path_extract(&response, "$.name");
        assert_eq!(name, Some(json!("test_catalog")));

        let id = recorder.simple_json_path_extract(&response, "$.metadata.id");
        assert_eq!(id, Some(json!("12345")));

        let nonexistent = recorder.simple_json_path_extract(&response, "$.nonexistent");
        assert_eq!(nonexistent, None);
    }

    #[test]
    fn test_recording_config_from_env() {
        // Test default configuration
        let config = RecorderConfig::default();
        assert!(!config.server_url.is_empty());
        assert!(!config.output_dir.is_empty());
    }
}
