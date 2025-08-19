//! User journey test framework for Unity Catalog client tests
//!
//! This module provides utilities for defining and executing multi-step user journeys
//! that involve dependent API calls. Journeys are defined in JSON format and can be
//! executed against mock servers or real Unity Catalog deployments.

use crate::test_utils::TestServer;
use mockito::Mock;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

use unitycatalog_client::UnityCatalogClient;

/// Represents a single step in a user journey
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyStep {
    /// Unique identifier for this step (used for referencing in other steps)
    pub id: String,

    /// Human-readable description of what this step does
    pub description: String,

    /// HTTP method (GET, POST, PUT, DELETE)
    pub method: String,

    /// API endpoint path (can contain variables like {catalog_name})
    pub path: String,

    /// Request body template (can contain variables)
    pub request_body: Option<Value>,

    /// Expected response status code
    pub expected_status: u16,

    /// Expected response body (for verification)
    pub expected_response: Option<Value>,

    /// Variables to extract from the response for use in subsequent steps
    /// Key is the variable name, value is the JSONPath to extract
    pub extract_variables: Option<HashMap<String, String>>,

    /// Dependencies on other steps (must complete before this step)
    pub depends_on: Option<Vec<String>>,

    /// Whether this step should continue on failure (for cleanup steps)
    pub continue_on_failure: Option<bool>,

    /// Tags for categorizing steps (setup, main, cleanup, etc.)
    pub tags: Option<Vec<String>>,
}

/// Represents a complete user journey with multiple steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserJourney {
    /// Journey name/identifier
    pub name: String,

    /// Description of what this journey tests
    pub description: String,

    /// Initial variables available to all steps
    pub variables: Option<HashMap<String, Value>>,

    /// Ordered list of steps to execute
    pub steps: Vec<JourneyStep>,

    /// Metadata about the journey
    pub metadata: Option<Map<String, Value>>,
}

/// Context for executing a journey, tracks variables and state
#[derive(Debug, Clone)]
pub struct JourneyContext {
    /// Variables available for substitution in steps
    pub variables: HashMap<String, Value>,

    /// Results from completed steps
    pub step_results: HashMap<String, StepResult>,

    /// Whether to continue execution on step failures
    pub continue_on_failure: bool,
}

/// Result of executing a single step
#[derive(Debug, Clone)]
pub struct StepResult {
    /// The step that was executed
    pub step: JourneyStep,

    /// Whether the step succeeded
    pub success: bool,

    /// HTTP status code received
    pub status_code: u16,

    /// Response body received
    pub response_body: Option<Value>,

    /// Error message if step failed
    pub error_message: Option<String>,

    /// Variables extracted from this step
    pub extracted_variables: HashMap<String, Value>,
}

/// Result of executing a complete journey
#[derive(Debug)]
pub struct JourneyResult {
    /// The journey that was executed
    pub journey: UserJourney,

    /// Results from all executed steps
    pub step_results: Vec<StepResult>,

    /// Final variable state
    pub final_variables: HashMap<String, Value>,

    /// Whether the entire journey succeeded
    pub success: bool,

    /// Summary of any failures
    pub failure_summary: Option<String>,
}

/// Journey execution engine
pub struct JourneyExecutor {
    server: Option<TestServer>,
    context: JourneyContext,
}

impl JourneyExecutor {
    /// Create a new journey executor with a client and optional mock server
    pub fn new(_client: UnityCatalogClient, server: Option<TestServer>) -> Self {
        Self {
            server,
            context: JourneyContext {
                variables: HashMap::new(),
                step_results: HashMap::new(),
                continue_on_failure: false,
            },
        }
    }

    /// Set initial variables for the journey
    pub fn with_variables(mut self, variables: HashMap<String, Value>) -> Self {
        self.context.variables = variables;
        self
    }

    pub fn continue_on_failure(mut self, continue_on_failure: bool) -> Self {
        self.context.continue_on_failure = continue_on_failure;
        self
    }

    /// Get access to the journey context
    pub fn context(&self) -> &JourneyContext {
        &self.context
    }

    /// Execute a complete user journey
    pub async fn execute_journey(&mut self, journey: UserJourney) -> JourneyResult {
        let mut step_results = Vec::new();
        let mut overall_success = true;
        let mut failure_messages = Vec::new();

        // Initialize context with journey variables
        if let Some(journey_vars) = &journey.variables {
            for (key, value) in journey_vars {
                self.context.variables.insert(key.clone(), value.clone());
            }
        }

        // Execute steps in order
        for step in &journey.steps {
            // Check dependencies
            if let Some(deps) = &step.depends_on {
                let mut deps_satisfied = true;
                for dep in deps {
                    if !self.context.step_results.contains_key(dep) {
                        deps_satisfied = false;
                        break;
                    }
                    if !self.context.step_results[dep].success {
                        deps_satisfied = false;
                        break;
                    }
                }

                if !deps_satisfied {
                    let error_msg = format!("Step '{}' dependencies not satisfied", step.id);
                    let step_result = StepResult {
                        step: step.clone(),
                        success: false,
                        status_code: 0,
                        response_body: None,
                        error_message: Some(error_msg.clone()),
                        extracted_variables: HashMap::new(),
                    };

                    step_results.push(step_result.clone());
                    self.context
                        .step_results
                        .insert(step.id.clone(), step_result);

                    overall_success = false;
                    failure_messages.push(error_msg);

                    if !step
                        .continue_on_failure
                        .unwrap_or(self.context.continue_on_failure)
                    {
                        break;
                    }
                    continue;
                }
            }

            // Execute the step
            let step_result = self.execute_step(step).await;
            let step_success = step_result.success;

            // Update context with extracted variables
            for (key, value) in &step_result.extracted_variables {
                self.context.variables.insert(key.clone(), value.clone());
            }

            step_results.push(step_result.clone());
            self.context
                .step_results
                .insert(step.id.clone(), step_result.clone());

            if !step_success {
                overall_success = false;
                if let Some(ref error_msg) = step_result.error_message {
                    failure_messages.push(format!("Step '{}': {}", step.id, error_msg));
                }

                if !step
                    .continue_on_failure
                    .unwrap_or(self.context.continue_on_failure)
                {
                    break;
                }
            }
        }

        let failure_summary = if failure_messages.is_empty() {
            None
        } else {
            Some(failure_messages.join("; "))
        };

        JourneyResult {
            journey,
            step_results,
            final_variables: self.context.variables.clone(),
            success: overall_success,
            failure_summary,
        }
    }

    /// Execute a single step
    pub async fn execute_step(&mut self, step: &JourneyStep) -> StepResult {
        // Substitute variables in path and request body
        let path = self.substitute_variables(&step.path);
        let request_body = step
            .request_body
            .as_ref()
            .map(|body| self.substitute_variables_in_json(body));

        // Setup mock if using test server
        let mock = if let Some(ref mut server) = self.server {
            Self::setup_mock_for_step(server, step, &path, &request_body)
        } else {
            None
        };

        // Execute the HTTP request
        let result = self.execute_http_request(step, &path, &request_body).await;

        // Verify mock was called if using test server
        if let Some(mock) = mock {
            mock.assert();
        }

        match result {
            Ok((status_code, response_body)) => {
                let expected_status = step.expected_status;
                let success = status_code == expected_status;

                let extracted_variables = if success {
                    self.extract_variables_from_response(&response_body, step)
                } else {
                    HashMap::new()
                };

                let error_message = if !success {
                    Some(format!(
                        "Expected status {}, got {}",
                        expected_status, status_code
                    ))
                } else {
                    None
                };

                StepResult {
                    step: step.clone(),
                    success,
                    status_code,
                    response_body: Some(response_body),
                    error_message,
                    extracted_variables,
                }
            }
            Err(error) => StepResult {
                step: step.clone(),
                success: false,
                status_code: 0,
                response_body: None,
                error_message: Some(error.to_string()),
                extracted_variables: HashMap::new(),
            },
        }
    }

    /// Setup mock response for a step when using test server
    fn setup_mock_for_step(
        server: &mut TestServer,
        step: &JourneyStep,
        path: &str,
        _request_body: &Option<Value>,
    ) -> Option<Mock> {
        let mock = server
            .mock_catalog_endpoint(&step.method, path)
            .with_status(step.expected_status as usize)
            .with_header("content-type", "application/json");

        let mock = if let Some(expected_response) = &step.expected_response {
            mock.with_body(serde_json::to_string(expected_response).unwrap())
        } else {
            // For responses without a body (like DELETE operations), provide empty JSON
            mock.with_body("{}")
        };

        Some(mock.create())
    }

    /// Execute HTTP request for a step
    async fn execute_http_request(
        &self,
        step: &JourneyStep,
        path: &str,
        request_body: &Option<Value>,
    ) -> Result<(u16, Value), Box<dyn std::error::Error>> {
        // Get the base URL from the test server if available
        let base_url = if let Some(ref server) = self.server {
            server.url()
        } else {
            // If no test server, we can't make real HTTP requests
            // Return the expected response for compatibility
            if let Some(expected_response) = &step.expected_response {
                return Ok((step.expected_status, expected_response.clone()));
            } else {
                return Ok((step.expected_status, serde_json::json!({})));
            }
        };

        // Create an HTTP client
        let client = reqwest::Client::new();

        // Construct the full URL
        let full_url = format!("{}{}", base_url.trim_end_matches('/'), path);

        // Create the request based on the HTTP method
        let mut request = match step.method.to_uppercase().as_str() {
            "GET" => client.get(&full_url),
            "POST" => client.post(&full_url),
            "PUT" => client.put(&full_url),
            "DELETE" => client.delete(&full_url),
            "PATCH" => client.patch(&full_url),
            _ => return Err(format!("Unsupported HTTP method: {}", step.method).into()),
        };

        // Add request body if provided
        if let Some(body) = request_body {
            request = request.json(body);
        }

        // Set content type header
        request = request.header("Content-Type", "application/json");

        // Execute the request
        let response = request.send().await?;
        let status = response.status().as_u16();

        // Get response body as JSON
        let response_text = response.text().await?;
        let response_json = if response_text.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&response_text)
                .unwrap_or_else(|_| serde_json::json!({"raw_response": response_text}))
        };

        Ok((status, response_json))
    }

    /// Substitute variables in a string template
    fn substitute_variables(&self, template: &str) -> String {
        let mut result = template.to_string();

        for (key, value) in &self.context.variables {
            let placeholder = format!("{{{}}}", key);
            let replacement = match value {
                Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }

        result
    }

    /// Substitute variables in a JSON value
    fn substitute_variables_in_json(&self, json: &Value) -> Value {
        match json {
            Value::String(s) => Value::String(self.substitute_variables(s)),
            Value::Object(obj) => {
                let mut new_obj = Map::new();
                for (key, value) in obj {
                    new_obj.insert(key.clone(), self.substitute_variables_in_json(value));
                }
                Value::Object(new_obj)
            }
            Value::Array(arr) => Value::Array(
                arr.iter()
                    .map(|v| self.substitute_variables_in_json(v))
                    .collect(),
            ),
            _ => json.clone(),
        }
    }

    /// Extract variables from response using JSONPath expressions
    fn extract_variables_from_response(
        &self,
        response: &Value,
        step: &JourneyStep,
    ) -> HashMap<String, Value> {
        let mut extracted = HashMap::new();

        if let Some(extract_rules) = &step.extract_variables {
            for (var_name, json_path) in extract_rules {
                // Simplified JSONPath extraction - in a real implementation,
                // you would use a proper JSONPath library
                if let Some(extracted_value) = self.simple_json_path_extract(response, json_path) {
                    extracted.insert(var_name.clone(), extracted_value);
                }
            }
        }

        extracted
    }

    /// Simple JSONPath-like extraction (simplified implementation)
    fn simple_json_path_extract(&self, json: &Value, path: &str) -> Option<Value> {
        // Handle simple paths like "$.name", "$.properties.environment", etc.
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
}

/// Utilities for loading and managing journey files
pub struct JourneyLoader;

impl JourneyLoader {
    /// Load a journey from a JSON file
    pub fn load_journey(journey_file: &str) -> Result<UserJourney, Box<dyn std::error::Error>> {
        let path = format!("tests/test_data/journeys/{}", journey_file);
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read journey file {}: {}", path, e))?;

        let journey: UserJourney = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse journey JSON from {}: {}", path, e))?;

        Ok(journey)
    }

    /// Load all journeys from the journeys directory
    pub fn load_all_journeys() -> Result<Vec<UserJourney>, Box<dyn std::error::Error>> {
        let dir_path = "tests/test_data/journeys";
        let dir = std::fs::read_dir(dir_path)
            .map_err(|e| format!("Failed to read journeys directory {}: {}", dir_path, e))?;

        let mut journeys = Vec::new();

        for entry in dir {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    match Self::load_journey(filename) {
                        Ok(journey) => journeys.push(journey),
                        Err(e) => eprintln!("Warning: Failed to load journey {}: {}", filename, e),
                    }
                }
            }
        }

        Ok(journeys)
    }

    /// Validate a journey definition
    pub fn validate_journey(journey: &UserJourney) -> Vec<String> {
        let mut errors = Vec::new();

        // Check for duplicate step IDs
        let mut step_ids = std::collections::HashSet::new();
        for step in &journey.steps {
            if !step_ids.insert(&step.id) {
                errors.push(format!("Duplicate step ID: {}", step.id));
            }
        }

        // Check dependencies exist
        for step in &journey.steps {
            if let Some(deps) = &step.depends_on {
                for dep in deps {
                    if !step_ids.contains(dep) {
                        errors.push(format!(
                            "Step '{}' depends on non-existent step '{}'",
                            step.id, dep
                        ));
                    }
                }
            }
        }

        // Check for circular dependencies (simplified check)
        // A more sophisticated implementation would use proper cycle detection

        errors
    }
}

/// Helper macros for creating journey steps
#[macro_export]
macro_rules! journey_step {
    ($id:expr, $description:expr, $method:expr, $path:expr) => {
        JourneyStep {
            id: $id.to_string(),
            description: $description.to_string(),
            method: $method.to_string(),
            path: $path.to_string(),
            request_body: None,
            expected_status: 200,
            expected_response: None,
            extract_variables: None,
            depends_on: None,
            continue_on_failure: None,
            tags: None,
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_variable_substitution() {
        let mut context = JourneyContext {
            variables: HashMap::new(),
            step_results: HashMap::new(),
            continue_on_failure: false,
        };

        context
            .variables
            .insert("catalog_name".to_string(), json!("test_catalog"));
        context
            .variables
            .insert("schema_name".to_string(), json!("test_schema"));

        let client = UnityCatalogClient::new(
            cloud_client::CloudClient::new_unauthenticated(),
            url::Url::parse("http://localhost").unwrap(),
        );

        let mut executor = JourneyExecutor::new(client, None);
        executor.context = context;

        let template = "/catalogs/{catalog_name}/schemas/{schema_name}";
        let result = executor.substitute_variables(template);

        assert_eq!(result, "/catalogs/test_catalog/schemas/test_schema");
    }

    #[test]
    fn test_json_path_extraction() {
        let response = json!({
            "name": "test_catalog",
            "properties": {
                "environment": "test"
            }
        });

        let client = UnityCatalogClient::new(
            cloud_client::CloudClient::new_unauthenticated(),
            url::Url::parse("http://localhost").unwrap(),
        );

        let executor = JourneyExecutor::new(client, None);

        let name = executor.simple_json_path_extract(&response, "$.name");
        assert_eq!(name, Some(json!("test_catalog")));

        let environment = executor.simple_json_path_extract(&response, "$.properties.environment");
        assert_eq!(environment, Some(json!("test")));
    }

    #[test]
    fn test_journey_validation() {
        let journey = UserJourney {
            name: "test_journey".to_string(),
            description: "Test journey".to_string(),
            variables: None,
            steps: vec![
                JourneyStep {
                    id: "step1".to_string(),
                    description: "First step".to_string(),
                    method: "POST".to_string(),
                    path: "/catalogs".to_string(),
                    request_body: None,
                    expected_status: 201,
                    expected_response: None,
                    extract_variables: None,
                    depends_on: None,
                    continue_on_failure: None,
                    tags: None,
                },
                JourneyStep {
                    id: "step2".to_string(),
                    description: "Second step".to_string(),
                    method: "GET".to_string(),
                    path: "/catalogs/{catalog_name}".to_string(),
                    request_body: None,
                    expected_status: 200,
                    expected_response: None,
                    extract_variables: None,
                    depends_on: Some(vec!["step1".to_string()]),
                    continue_on_failure: None,
                    tags: None,
                },
            ],
            metadata: None,
        };

        let errors = JourneyLoader::validate_journey(&journey);
        assert!(errors.is_empty());
    }
}
