//! Journey Integration Recorder
//!
//! This module provides functionality to record real server responses for journey tests.
//! It allows capturing actual API responses from a deployed Unity Catalog server and
//! saving them for later use in mock tests or validation.

use chrono::{DateTime, Utc};
use cloud_client::CloudClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use tokio::time::{Duration, timeout};
use unitycatalog_client::UnityCatalogClient;
use url::Url;

// Inline the necessary types instead of importing from test_utils

/// Journey step definition (inline version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyStep {
    pub id: String,
    pub description: String,
    pub method: String,
    pub path: String,
    pub request_body: Option<Value>,
    pub expected_status: u16,
    pub expected_response: Option<Value>,
    pub extract_variables: Option<HashMap<String, String>>,
    pub depends_on: Option<Vec<String>>,
    pub continue_on_failure: Option<bool>,
    pub tags: Option<Vec<String>>,
}

/// User journey definition (inline version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserJourney {
    pub name: String,
    pub description: String,
    pub variables: Option<HashMap<String, Value>>,
    pub steps: Vec<JourneyStep>,
    pub metadata: Option<HashMap<String, Value>>,
}

/// Journey loader (inline version)
pub struct JourneyLoader;

impl JourneyLoader {
    pub fn load_journey(filename: &str) -> Result<UserJourney, Box<dyn std::error::Error>> {
        let path = format!("tests/test_data/journeys/{}", filename);
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read journey file {}: {}", path, e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse journey JSON from {}: {}", path, e).into())
    }
}

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
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let server_url =
            env::var("UC_SERVER_URL").map_err(|_| "UC_SERVER_URL environment variable not set")?;

        let auth_token = env::var("UC_AUTH_TOKEN").ok();

        let output_dir = env::var("JOURNEY_RECORDING_DIR")
            .unwrap_or_else(|_| "tests/test_data/journeys/recorded".to_string());

        let record_success_only = env::var("RECORD_SUCCESS_ONLY")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let overwrite_existing = env::var("OVERWRITE_JOURNEY_RESPONSES")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let request_timeout_secs = env::var("JOURNEY_REQUEST_TIMEOUT")
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
        env::var("RECORD_JOURNEY_RESPONSES")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false)
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
    /// Original journey definition
    pub journey: UserJourney,
    /// Recorded steps with responses
    pub recorded_steps: Vec<RecordedStep>,
    /// Final variables after journey execution
    pub final_variables: HashMap<String, Value>,
    /// Recording metadata
    pub metadata: RecordingMetadata,
}

/// Metadata about the recording session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingMetadata {
    /// When the recording was made
    pub recorded_at: DateTime<Utc>,
    /// Server URL used for recording
    pub server_url: String,
    /// Unity Catalog version (if available)
    pub server_version: Option<String>,
    /// Total steps recorded
    pub total_steps: usize,
    /// Number of successful steps
    pub successful_steps: usize,
    /// Recording configuration used
    pub config_summary: String,
}

/// Journey recorder that executes journeys against real servers and captures responses
pub struct JourneyRecorder {
    /// Recording configuration
    config: RecordingConfig,
    /// Unity Catalog client for making requests
    client: UnityCatalogClient,
    /// Current journey variables
    variables: HashMap<String, Value>,
}

impl JourneyRecorder {
    /// Create a new journey recorder with the given configuration
    pub fn new(config: RecordingConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Create authenticated client if token is provided
        let cloud_client = if let Some(token) = &config.auth_token {
            CloudClient::new_with_token(token.clone())
        } else {
            CloudClient::new_unauthenticated()
        };

        let base_url = Url::parse(&config.server_url)?;
        let client = UnityCatalogClient::new(cloud_client, base_url);

        Ok(Self {
            config,
            client,
            variables: HashMap::new(),
        })
    }

    /// Create a recorder from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let config = RecordingConfig::from_env()?;
        Self::new(config)
    }

    /// Set initial variables for the journey
    pub fn with_variables(mut self, variables: HashMap<String, Value>) -> Self {
        self.variables = variables;
        self
    }

    /// Get current variables
    pub fn variables(&self) -> &HashMap<String, Value> {
        &self.variables
    }

    /// Record a complete journey, executing all steps against the real server
    pub async fn record_journey(
        &mut self,
        journey: UserJourney,
    ) -> Result<RecordedJourney, Box<dyn std::error::Error>> {
        println!("🎬 Recording journey: {}", journey.name);
        println!("📡 Server: {}", self.config.server_url);

        // Ensure output directory exists
        fs::create_dir_all(&self.config.output_dir)?;

        // Merge journey variables with existing variables
        if let Some(journey_vars) = &journey.variables {
            for (key, value) in journey_vars {
                self.variables.insert(key.clone(), value.clone());
            }
        }

        let mut recorded_steps = Vec::new();
        let mut successful_steps = 0;
        let recording_start = Utc::now();

        // Execute steps in dependency order
        let execution_order = self.resolve_step_dependencies(&journey.steps)?;

        for step_id in execution_order {
            let step = journey
                .steps
                .iter()
                .find(|s| s.id == step_id)
                .ok_or_else(|| format!("Step not found: {}", step_id))?;

            println!("🔄 Recording step: {} - {}", step.id, step.description);

            match self.record_step(step).await {
                Ok(recorded_step) => {
                    // Extract variables from the response
                    if let Some(extractions) = &step.extract_variables {
                        for (var_name, json_path) in extractions {
                            if let Some(value) = self
                                .extract_value_from_json(&recorded_step.response.body, json_path)
                            {
                                self.variables.insert(var_name.clone(), value);
                                println!(
                                    "📝 Extracted variable: {} = {}",
                                    var_name, self.variables[var_name]
                                );
                            }
                        }
                    }

                    if recorded_step.response.status_code >= 200
                        && recorded_step.response.status_code < 300
                    {
                        successful_steps += 1;
                        println!(
                            "✅ Step succeeded: HTTP {}",
                            recorded_step.response.status_code
                        );
                    } else {
                        println!(
                            "⚠️ Step failed: HTTP {}",
                            recorded_step.response.status_code
                        );
                    }

                    recorded_steps.push(recorded_step);
                }
                Err(e) => {
                    println!("❌ Step failed: {}", e);
                    if !step.continue_on_failure.unwrap_or(false) {
                        return Err(e);
                    }
                }
            }
        }

        let metadata = RecordingMetadata {
            recorded_at: recording_start,
            server_url: self.config.server_url.clone(),
            server_version: None, // Could be extracted from server info endpoint
            total_steps: recorded_steps.len(),
            successful_steps,
            config_summary: format!(
                "success_only={}, overwrite={}, timeout={}s",
                self.config.record_success_only,
                self.config.overwrite_existing,
                self.config.request_timeout_secs
            ),
        };

        let recorded_journey = RecordedJourney {
            journey,
            recorded_steps,
            final_variables: self.variables.clone(),
            metadata,
        };

        // Save the recorded journey
        self.save_recorded_journey(&recorded_journey).await?;

        println!(
            "🎉 Journey recording completed: {}/{} steps successful",
            successful_steps, recorded_journey.metadata.total_steps
        );

        Ok(recorded_journey)
    }

    /// Record a single step execution
    async fn record_step(
        &self,
        step: &JourneyStep,
    ) -> Result<RecordedStep, Box<dyn std::error::Error>> {
        // Substitute variables in the path and request body
        let path = self.substitute_variables(&step.path);
        let request_body = step
            .request_body
            .as_ref()
            .map(|body| self.substitute_variables_in_json(body));

        // Execute the HTTP request with timeout
        let request_future = self.execute_http_request(step, &path, &request_body);
        let (status_code, response_body, headers) = timeout(
            Duration::from_secs(self.config.request_timeout_secs),
            request_future,
        )
        .await??;

        // Check if we should record this response
        if self.config.record_success_only && (status_code < 200 || status_code >= 300) {
            return Err(format!("Skipping non-successful response: HTTP {}", status_code).into());
        }

        let recorded_response = RecordedResponse {
            status_code,
            body: response_body,
            headers,
            recorded_at: Utc::now(),
            method: step.method.clone(),
            path: path.clone(),
            request_body,
        };

        // Extract variables for the recorded step
        let mut extracted_variables = HashMap::new();
        if let Some(extractions) = &step.extract_variables {
            for (var_name, json_path) in extractions {
                if let Some(value) =
                    self.extract_value_from_json(&recorded_response.body, json_path)
                {
                    extracted_variables.insert(var_name.clone(), value);
                }
            }
        }

        Ok(RecordedStep {
            step: step.clone(),
            response: recorded_response,
            extracted_variables,
        })
    }

    /// Execute HTTP request against the real server
    async fn execute_http_request(
        &self,
        step: &JourneyStep,
        path: &str,
        request_body: &Option<Value>,
    ) -> Result<(u16, Value, HashMap<String, String>), Box<dyn std::error::Error>> {
        // Use the Unity Catalog client's underlying HTTP client
        // For now, we'll use reqwest directly to have more control over the recording
        let http_client = reqwest::Client::new();

        let full_url = format!("{}{}", self.config.server_url.trim_end_matches('/'), path);

        let mut request = match step.method.to_uppercase().as_str() {
            "GET" => http_client.get(&full_url),
            "POST" => http_client.post(&full_url),
            "PUT" => http_client.put(&full_url),
            "DELETE" => http_client.delete(&full_url),
            "PATCH" => http_client.patch(&full_url),
            _ => return Err(format!("Unsupported HTTP method: {}", step.method).into()),
        };

        // Add authentication header if token is available
        if let Some(token) = &self.config.auth_token {
            request = request.bearer_auth(token);
        }

        // Add request body if provided
        if let Some(body) = request_body {
            request = request.json(body);
        }

        // Set standard headers
        request = request
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        // Execute the request
        let response = request.send().await?;
        let status_code = response.status().as_u16();

        // Capture important headers
        let mut headers = HashMap::new();
        for (name, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                match name.as_str() {
                    "content-type" | "content-length" | "x-request-id" | "x-trace-id" => {
                        headers.insert(name.to_string(), value_str.to_string());
                    }
                    _ => {}
                }
            }
        }

        // Get response body
        let response_text = response.text().await?;
        let response_json = if response_text.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&response_text)
                .unwrap_or_else(|_| serde_json::json!({"raw_response": response_text}))
        };

        Ok((status_code, response_json, headers))
    }

    /// Save recorded journey to disk
    async fn save_recorded_journey(
        &self,
        recorded_journey: &RecordedJourney,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let filename = format!("{}_recorded.json", recorded_journey.journey.name);
        let file_path = self.config.output_dir.join(&filename);

        // Check if file exists and we shouldn't overwrite
        if file_path.exists() && !self.config.overwrite_existing {
            println!(
                "⏭️ Skipping existing file: {} (use OVERWRITE_JOURNEY_RESPONSES=true to overwrite)",
                filename
            );
            return Ok(());
        }

        let json_content = serde_json::to_string_pretty(recorded_journey)?;
        fs::write(&file_path, json_content)?;

        println!("💾 Saved recorded journey: {}", file_path.display());
        Ok(())
    }

    /// Resolve step execution order based on dependencies
    fn resolve_step_dependencies(
        &self,
        steps: &[JourneyStep],
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut execution_order = Vec::new();
        let mut completed_steps = std::collections::HashSet::new();
        let mut remaining_steps: std::collections::HashMap<String, &JourneyStep> =
            steps.iter().map(|step| (step.id.clone(), step)).collect();

        while !remaining_steps.is_empty() {
            let mut progress_made = false;

            // Find steps that can be executed (all dependencies satisfied)
            let ready_steps: Vec<String> = remaining_steps
                .iter()
                .filter(|(_, step)| {
                    step.depends_on
                        .as_ref()
                        .map(|deps| deps.iter().all(|dep| completed_steps.contains(dep)))
                        .unwrap_or(true)
                })
                .map(|(id, _)| id.clone())
                .collect();

            for step_id in ready_steps {
                execution_order.push(step_id.clone());
                completed_steps.insert(step_id.clone());
                remaining_steps.remove(&step_id);
                progress_made = true;
            }

            if !progress_made {
                return Err("Circular dependency detected in journey steps".into());
            }
        }

        Ok(execution_order)
    }

    /// Substitute variables in a string
    fn substitute_variables(&self, template: &str) -> String {
        let mut result = template.to_string();
        for (key, value) in &self.variables {
            let placeholder = format!("{{{}}}", key);
            let value_str = match value {
                Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &value_str);
        }
        result
    }

    /// Substitute variables in JSON value
    fn substitute_variables_in_json(&self, json: &Value) -> Value {
        match json {
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
            _ => json.clone(),
        }
    }

    /// Extract value from JSON using simple JSONPath-like syntax
    fn extract_value_from_json(&self, json: &Value, path: &str) -> Option<Value> {
        if path.starts_with("$.") {
            let path = &path[2..]; // Remove "$."
            self.extract_nested_value(json, path)
        } else {
            None
        }
    }

    /// Extract nested value from JSON
    fn extract_nested_value(&self, json: &Value, path: &str) -> Option<Value> {
        if path.is_empty() {
            return Some(json.clone());
        }

        let parts: Vec<&str> = path.splitn(2, '.').collect();
        let current_key = parts[0];
        let remaining_path = parts.get(1).unwrap_or(&"");

        match json {
            Value::Object(map) => {
                if let Some(value) = map.get(current_key) {
                    if remaining_path.is_empty() {
                        Some(value.clone())
                    } else {
                        self.extract_nested_value(value, remaining_path)
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Convenience function to record a journey from a file
pub async fn record_journey_from_file(
    journey_file: &str,
) -> Result<RecordedJourney, Box<dyn std::error::Error>> {
    if !RecordingConfig::is_recording_enabled() {
        return Err("Recording not enabled. Set RECORD_JOURNEY_RESPONSES=true".into());
    }

    let journey = JourneyLoader::load_journey(journey_file)?;
    let mut recorder = JourneyRecorder::from_env()?;

    // Add some default variables that are commonly needed
    let mut default_variables = HashMap::new();
    default_variables.insert(
        "timestamp".to_string(),
        Value::String(Utc::now().timestamp().to_string()),
    );
    default_variables.insert(
        "test_suffix".to_string(),
        Value::String(uuid::Uuid::new_v4().to_string()[..8].to_string()),
    );

    recorder = recorder.with_variables(default_variables);
    recorder.record_journey(journey).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_config_from_env() {
        // Set up environment variables
        unsafe {
            env::set_var("UC_SERVER_URL", "http://test.example.com");
            env::set_var("UC_AUTH_TOKEN", "test-token");
            env::set_var("RECORD_SUCCESS_ONLY", "false");
        }

        let config = RecordingConfig::from_env().unwrap();

        assert_eq!(config.server_url, "http://test.example.com");
        assert_eq!(config.auth_token, Some("test-token".to_string()));
        assert!(!config.record_success_only);

        // Clean up
        unsafe {
            env::remove_var("UC_SERVER_URL");
            env::remove_var("UC_AUTH_TOKEN");
            env::remove_var("RECORD_SUCCESS_ONLY");
        }
    }

    #[test]
    fn test_variable_substitution() {
        let mut variables = HashMap::new();
        variables.insert(
            "catalog_name".to_string(),
            Value::String("test_catalog".to_string()),
        );
        variables.insert(
            "id".to_string(),
            Value::Number(serde_json::Number::from(123)),
        );

        let config = RecordingConfig {
            server_url: "http://test.example.com".to_string(),
            auth_token: None,
            output_dir: PathBuf::from("test"),
            record_success_only: true,
            overwrite_existing: false,
            request_timeout_secs: 30,
        };

        let recorder = JourneyRecorder {
            config,
            client: UnityCatalogClient::new(
                CloudClient::new_unauthenticated(),
                Url::parse("http://test.example.com").unwrap(),
            ),
            variables,
        };

        let result = recorder.substitute_variables("/catalogs/{catalog_name}/items/{id}");
        assert_eq!(result, "/catalogs/test_catalog/items/123");
    }

    #[test]
    fn test_json_path_extraction() {
        let recorder = JourneyRecorder {
            config: RecordingConfig {
                server_url: "http://test.example.com".to_string(),
                auth_token: None,
                output_dir: PathBuf::from("test"),
                record_success_only: true,
                overwrite_existing: false,
                request_timeout_secs: 30,
            },
            client: UnityCatalogClient::new(
                CloudClient::new_unauthenticated(),
                Url::parse("http://test.example.com").unwrap(),
            ),
            variables: HashMap::new(),
        };

        let json = serde_json::json!({
            "catalog": {
                "name": "test_catalog",
                "id": "12345"
            }
        });

        let result = recorder.extract_value_from_json(&json, "$.catalog.name");
        assert_eq!(result, Some(Value::String("test_catalog".to_string())));

        let result = recorder.extract_value_from_json(&json, "$.catalog.id");
        assert_eq!(result, Some(Value::String("12345".to_string())));
    }
}
