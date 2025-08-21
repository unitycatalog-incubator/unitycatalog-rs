//! Journey recording functionality for Unity Catalog acceptance testing
//!
//! This module provides the ability to record real API responses from Unity Catalog
//! servers during journey execution, which can then be used as test data for mock
//! servers or for validation purposes.

use crate::{
    AcceptanceError, AcceptanceResult,
    journey::{JourneyStep, UserJourney},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for journey recording
#[derive(Debug, Clone)]
pub struct RecordingConfig {
    /// Base URL of the Unity Catalog server
    pub server_url: String,
    /// Authentication token for the server
    pub auth_token: Option<String>,
    /// Directory to save recorded responses
    pub output_dir: PathBuf,
    /// Whether to record only successful responses (2xx status codes)
    pub record_success_only: bool,
    /// Whether to overwrite existing recorded files
    pub overwrite_existing: bool,
    /// Timeout for HTTP requests in seconds
    pub request_timeout_secs: u64,
}

impl RecordingConfig {
    /// Create recording configuration from environment variables
    pub fn from_env() -> AcceptanceResult<Self> {
        let server_url = std::env::var("UC_SERVER_URL").map_err(|_| {
            AcceptanceError::Recording("UC_SERVER_URL environment variable not set".to_string())
        })?;

        let auth_token = std::env::var("UC_AUTH_TOKEN").ok();

        let output_dir = std::env::var("JOURNEY_RECORDING_DIR")
            .unwrap_or_else(|_| "test_data/journeys/recorded".to_string());

        let record_success_only = std::env::var("RECORD_SUCCESS_ONLY")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let overwrite_existing = std::env::var("OVERWRITE_JOURNEY_RESPONSES")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let request_timeout_secs = std::env::var("JOURNEY_REQUEST_TIMEOUT")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        Ok(Self {
            server_url,
            auth_token,
            output_dir: PathBuf::from(output_dir),
            record_success_only,
            overwrite_existing,
            request_timeout_secs,
        })
    }

    /// Check if recording is enabled via environment variables
    pub fn is_recording_enabled() -> bool {
        std::env::var("RECORD_JOURNEY_RESPONSES")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false)
    }

    /// Get the server URL
    pub fn server_url(&self) -> &str {
        &self.server_url
    }

    /// Get the auth token
    pub fn auth_token(&self) -> Option<&str> {
        self.auth_token.as_deref()
    }
}

/// Recorded response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedResponse {
    /// HTTP status code
    pub status_code: u16,
    /// Response body as JSON
    pub body: Value,
    /// Response headers (selected important ones)
    pub headers: HashMap<String, String>,
    /// Timestamp when recorded
    pub recorded_at: DateTime<Utc>,
    /// Request method
    pub method: String,
    /// Request path
    pub path: String,
    /// Request body (if any)
    pub request_body: Option<Value>,
}

/// Recorded step result containing both the step and its response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedStep {
    /// The journey step that was executed
    pub step: JourneyStep,
    /// The recorded response from the server
    pub response: RecordedResponse,
    /// Variables extracted from this step
    pub extracted_variables: HashMap<String, Value>,
}

/// Complete recorded journey with all steps and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedJourney {
    /// The journey that was executed
    pub journey: UserJourney,
    /// All recorded steps in execution order
    pub recorded_steps: Vec<RecordedStep>,
    /// Final variable state after journey execution
    pub final_variables: HashMap<String, Value>,
    /// Recording metadata
    pub metadata: RecordingMetadata,
}

/// Metadata about the recording session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingMetadata {
    /// When the recording was made
    pub recorded_at: DateTime<Utc>,
    /// Server URL that was recorded from
    pub server_url: String,
    /// Server version (if available)
    pub server_version: Option<String>,
    /// Total number of steps recorded
    pub total_steps: usize,
    /// Number of successful steps
    pub successful_steps: usize,
    /// Configuration summary
    pub config_summary: HashMap<String, Value>,
}

/// Journey recorder for capturing real API responses
pub struct JourneyRecorder {
    /// Recording configuration
    config: RecordingConfig,
    /// HTTP client for making requests
    client: reqwest::Client,
    /// Variables for the current recording session
    variables: HashMap<String, Value>,
}

impl JourneyRecorder {
    /// Create a new journey recorder with configuration
    pub fn new(config: RecordingConfig) -> AcceptanceResult<Self> {
        let mut client_builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.request_timeout_secs));

        // Add authentication if provided
        if let Some(token) = &config.auth_token {
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).map_err(
                    |e| AcceptanceError::Recording(format!("Invalid auth token: {}", e)),
                )?,
            );
            client_builder = client_builder.default_headers(headers);
        }

        let client = client_builder.build()?;

        Ok(Self {
            config,
            client,
            variables: HashMap::new(),
        })
    }

    /// Create a recorder from environment variables
    pub fn from_env() -> AcceptanceResult<Self> {
        let config = RecordingConfig::from_env()?;
        Self::new(config)
    }

    /// Set variables for the recording session
    pub fn with_variables(mut self, variables: HashMap<String, Value>) -> Self {
        self.variables = variables;
        self
    }

    /// Get the current variables
    pub fn variables(&self) -> &HashMap<String, Value> {
        &self.variables
    }

    /// Record a complete journey by executing it against a real server
    pub async fn record_journey(
        &mut self,
        journey: UserJourney,
    ) -> AcceptanceResult<RecordedJourney> {
        let start_time = Utc::now();
        let mut recorded_steps = Vec::new();
        let mut successful_steps = 0;

        // Initialize variables with journey variables
        if let Some(journey_vars) = &journey.variables {
            for (key, value) in journey_vars {
                self.variables.insert(key.clone(), value.clone());
            }
        }

        // Resolve step execution order based on dependencies
        let execution_order = self.resolve_step_dependencies(&journey.steps);

        // Execute and record each step
        for step in execution_order {
            // Check if dependencies are satisfied
            if let Some(deps) = &step.depends_on {
                let deps_satisfied = deps.iter().all(|dep| {
                    recorded_steps.iter().any(|recorded: &RecordedStep| {
                        recorded.step.id == *dep && recorded.response.status_code < 400
                    })
                });

                if !deps_satisfied {
                    tracing::warn!(
                        "Skipping step '{}' due to unsatisfied dependencies",
                        step.id
                    );
                    continue;
                }
            }

            match self.record_step(&step).await {
                Ok(recorded_step) => {
                    let is_success = recorded_step.response.status_code < 400;
                    if is_success {
                        successful_steps += 1;
                        // Extract variables for use in subsequent steps
                        for (key, value) in &recorded_step.extracted_variables {
                            self.variables.insert(key.clone(), value.clone());
                        }
                    }
                    recorded_steps.push(recorded_step);
                }
                Err(e) => {
                    tracing::error!("Failed to record step '{}': {}", step.id, e);

                    // Create a failed step record
                    let failed_step = RecordedStep {
                        step: step.clone(),
                        response: RecordedResponse {
                            status_code: 0,
                            body: Value::String(format!("Recording failed: {}", e)),
                            headers: HashMap::new(),
                            recorded_at: Utc::now(),
                            method: step.method.clone(),
                            path: step.path.clone(),
                            request_body: step.request_body.clone(),
                        },
                        extracted_variables: HashMap::new(),
                    };
                    recorded_steps.push(failed_step);

                    // Continue recording other steps unless it's a critical failure
                    let should_continue = step.continue_on_failure.unwrap_or(false);
                    if !should_continue {
                        break;
                    }
                }
            }
        }

        // Create recording metadata
        let metadata = RecordingMetadata {
            recorded_at: start_time,
            server_url: self.config.server_url.clone(),
            server_version: None, // Could be populated by making a version endpoint call
            total_steps: recorded_steps.len(),
            successful_steps,
            config_summary: {
                let mut map = HashMap::new();
                map.insert(
                    "record_success_only".to_string(),
                    Value::Bool(self.config.record_success_only),
                );
                map.insert(
                    "overwrite_existing".to_string(),
                    Value::Bool(self.config.overwrite_existing),
                );
                map.insert(
                    "request_timeout_secs".to_string(),
                    Value::Number(serde_json::Number::from(self.config.request_timeout_secs)),
                );
                map
            },
        };

        let recorded_journey = RecordedJourney {
            journey,
            recorded_steps,
            final_variables: self.variables.clone(),
            metadata,
        };

        // Save the recorded journey
        self.save_recorded_journey(&recorded_journey).await?;

        Ok(recorded_journey)
    }

    /// Record a single step execution
    async fn record_step(&mut self, step: &JourneyStep) -> AcceptanceResult<RecordedStep> {
        // Substitute variables in the step
        let resolved_step = self.substitute_variables_in_step(step);

        // Build the request URL
        let url = format!("{}{}", self.config.server_url, resolved_step.path);

        // Execute the HTTP request
        let (status_code, response_body, headers) =
            self.execute_http_request(&resolved_step, &url).await?;

        // Check if we should record this response
        let should_record = if self.config.record_success_only {
            status_code < 400
        } else {
            true
        };

        if !should_record {
            return Err(AcceptanceError::Recording(format!(
                "Response not recorded due to status {} (record_success_only={})",
                status_code, self.config.record_success_only
            )));
        }

        // Extract variables from the response
        let extracted_variables = if let Some(extract_vars) = &resolved_step.extract_variables {
            self.extract_variables_from_response(extract_vars, &response_body)
        } else {
            HashMap::new()
        };

        // Create the recorded response
        let recorded_response = RecordedResponse {
            status_code,
            body: response_body,
            headers,
            recorded_at: Utc::now(),
            method: resolved_step.method.clone(),
            path: resolved_step.path.clone(),
            request_body: resolved_step.request_body.clone(),
        };

        Ok(RecordedStep {
            step: resolved_step,
            response: recorded_response,
            extracted_variables,
        })
    }

    /// Execute an HTTP request and return response details
    async fn execute_http_request(
        &self,
        step: &JourneyStep,
        url: &str,
    ) -> AcceptanceResult<(u16, Value, HashMap<String, String>)> {
        // Build the request
        let mut request_builder = match step.method.to_uppercase().as_str() {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "PUT" => self.client.put(url),
            "DELETE" => self.client.delete(url),
            "PATCH" => self.client.patch(url),
            _ => {
                return Err(AcceptanceError::StepExecution {
                    step_id: step.id.clone(),
                    message: format!("Unsupported HTTP method: {}", step.method),
                });
            }
        };

        // Add request body if present
        if let Some(body) = &step.request_body {
            request_builder = request_builder.json(body);
        }

        // Execute the request
        let response = request_builder.send().await?;
        let status_code = response.status().as_u16();

        // Extract important headers
        let mut headers = HashMap::new();
        if let Some(content_type) = response.headers().get("content-type") {
            if let Ok(content_type_str) = content_type.to_str() {
                headers.insert("content-type".to_string(), content_type_str.to_string());
            }
        }
        if let Some(content_length) = response.headers().get("content-length") {
            if let Ok(content_length_str) = content_length.to_str() {
                headers.insert("content-length".to_string(), content_length_str.to_string());
            }
        }

        // Parse response body
        let response_text = response.text().await?;
        let response_body: Value = if response_text.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&response_text).unwrap_or_else(|_| Value::String(response_text))
        };

        Ok((status_code, response_body, headers))
    }

    /// Save a recorded journey to disk
    async fn save_recorded_journey(
        &self,
        recorded_journey: &RecordedJourney,
    ) -> AcceptanceResult<()> {
        // Ensure output directory exists
        tokio::fs::create_dir_all(&self.config.output_dir).await?;

        // Generate filename
        let filename = format!("{}.json", recorded_journey.journey.name);
        let filepath = self.config.output_dir.join(filename);

        // Check if file exists and if we should overwrite
        if filepath.exists() && !self.config.overwrite_existing {
            return Err(AcceptanceError::Recording(format!(
                "Recorded journey file already exists: {:?} (set OVERWRITE_JOURNEY_RESPONSES=true to overwrite)",
                filepath
            )));
        }

        // Serialize and write the recorded journey
        let json_content = serde_json::to_string_pretty(recorded_journey)?;
        tokio::fs::write(&filepath, json_content).await?;

        tracing::info!("Recorded journey saved to: {:?}", filepath);
        Ok(())
    }

    /// Resolve step dependencies to determine execution order
    fn resolve_step_dependencies(&self, steps: &[JourneyStep]) -> Vec<JourneyStep> {
        let mut ordered_steps = Vec::new();
        let mut remaining_steps: Vec<_> = steps.to_vec();
        let mut satisfied_steps = std::collections::HashSet::new();

        while !remaining_steps.is_empty() {
            let mut made_progress = false;

            remaining_steps.retain(|step| {
                let deps_satisfied = step
                    .depends_on
                    .as_ref()
                    .map(|deps| deps.iter().all(|dep| satisfied_steps.contains(dep)))
                    .unwrap_or(true);

                if deps_satisfied {
                    ordered_steps.push(step.clone());
                    satisfied_steps.insert(step.id.clone());
                    made_progress = true;
                    false // Remove from remaining
                } else {
                    true // Keep in remaining
                }
            });

            if !made_progress {
                // Circular dependency or missing dependency
                tracing::warn!(
                    "Circular dependency detected or missing steps. Adding remaining steps: {:?}",
                    remaining_steps.iter().map(|s| &s.id).collect::<Vec<_>>()
                );
                ordered_steps.extend(remaining_steps);
                break;
            }
        }

        ordered_steps
    }

    /// Substitute variables in a step
    fn substitute_variables_in_step(&self, step: &JourneyStep) -> JourneyStep {
        let mut substituted_step = step.clone();

        // Substitute in path
        substituted_step.path = self.substitute_variables(&step.path);

        // Substitute in request body
        if let Some(body) = &step.request_body {
            substituted_step.request_body = Some(self.substitute_variables_in_json(body));
        }

        substituted_step
    }

    /// Substitute variables in a string
    fn substitute_variables(&self, template: &str) -> String {
        let mut result = template.to_string();

        for (key, value) in &self.variables {
            let placeholder = format!("{{{}}}", key);
            let replacement = match value {
                Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }

        result
    }

    /// Substitute variables in JSON values
    fn substitute_variables_in_json(&self, value: &Value) -> Value {
        match value {
            Value::String(s) => Value::String(self.substitute_variables(s)),
            Value::Object(map) => {
                let mut new_map = serde_json::Map::new();
                for (k, v) in map {
                    new_map.insert(k.clone(), self.substitute_variables_in_json(v));
                }
                Value::Object(new_map)
            }
            Value::Array(arr) => Value::Array(
                arr.iter()
                    .map(|v| self.substitute_variables_in_json(v))
                    .collect(),
            ),
            _ => value.clone(),
        }
    }

    /// Extract variables from response using JSONPath-like expressions
    fn extract_variables_from_response(
        &self,
        extractions: &HashMap<String, String>,
        response: &Value,
    ) -> HashMap<String, Value> {
        let mut extracted = HashMap::new();

        for (var_name, json_path) in extractions {
            if let Some(extracted_value) = self.extract_value_from_json(response, json_path) {
                extracted.insert(var_name.clone(), extracted_value);
            }
        }

        extracted
    }

    /// Extract a value from JSON using simple JSONPath notation
    fn extract_value_from_json(&self, value: &Value, path: &str) -> Option<Value> {
        if path == "$" {
            return Some(value.clone());
        }

        if let Some(field_path) = path.strip_prefix("$.") {
            self.extract_nested_value(value, field_path)
        } else {
            None
        }
    }

    /// Extract nested value from JSON object
    fn extract_nested_value(&self, value: &Value, path: &str) -> Option<Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = value;

        for part in parts {
            match current {
                Value::Object(map) => {
                    current = map.get(part)?;
                }
                Value::Array(arr) => {
                    let index: usize = part.parse().ok()?;
                    current = arr.get(index)?;
                }
                _ => return None,
            }
        }

        Some(current.clone())
    }
}

/// Convenience function to record a journey from a file
pub async fn record_journey_from_file(filename: &str) -> AcceptanceResult<RecordedJourney> {
    // Load the journey
    let journey = crate::journey::JourneyLoader::load_journey(filename)?;

    // Create recorder from environment
    let mut recorder = JourneyRecorder::from_env()?;

    // Record the journey
    recorder.record_journey(journey).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_recording_config_from_env() {
        // Test with missing required env var
        unsafe {
            std::env::remove_var("UC_SERVER_URL");
        }
        assert!(RecordingConfig::from_env().is_err());

        // Test with valid config
        unsafe {
            std::env::set_var("UC_SERVER_URL", "http://localhost:8080");
            std::env::set_var("UC_AUTH_TOKEN", "test-token");
        }

        let config = RecordingConfig::from_env().unwrap();
        assert_eq!(config.server_url, "http://localhost:8080");
        assert_eq!(config.auth_token, Some("test-token".to_string()));
        assert_eq!(config.request_timeout_secs, 30);
    }

    #[test]
    fn test_variable_substitution() {
        let config = RecordingConfig {
            server_url: "http://localhost:8080".to_string(),
            auth_token: None,
            output_dir: PathBuf::from("test"),
            record_success_only: true,
            overwrite_existing: false,
            request_timeout_secs: 30,
        };

        let recorder = JourneyRecorder::new(config).unwrap();
        let recorder = recorder.with_variables({
            let mut vars = HashMap::new();
            vars.insert(
                "catalog_name".to_string(),
                Value::String("test_catalog".to_string()),
            );
            vars
        });

        let template = "/api/2.1/unity-catalog/catalogs/{catalog_name}";
        let result = recorder.substitute_variables(template);
        assert_eq!(result, "/api/2.1/unity-catalog/catalogs/test_catalog");
    }

    #[test]
    fn test_json_path_extraction() {
        let config = RecordingConfig {
            server_url: "http://localhost:8080".to_string(),
            auth_token: None,
            output_dir: PathBuf::from("test"),
            record_success_only: true,
            overwrite_existing: false,
            request_timeout_secs: 30,
        };

        let recorder = JourneyRecorder::new(config).unwrap();

        let response = json!({
            "name": "test_catalog",
            "created_at": 1234567890,
            "properties": {
                "environment": "test"
            }
        });

        let result = recorder.extract_value_from_json(&response, "$.name");
        assert_eq!(result, Some(Value::String("test_catalog".to_string())));

        let result = recorder.extract_value_from_json(&response, "$.properties.environment");
        assert_eq!(result, Some(Value::String("test".to_string())));
    }
}
