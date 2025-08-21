//! Core journey execution engine for Unity Catalog acceptance testing
//!
//! This module provides the primary functionality for defining, loading, and executing
//! user journeys. Journeys are multi-step workflows that test dependent API operations
//! with variable substitution and dependency management.

use crate::{AcceptanceError, AcceptanceResult, mock::TestServer};
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
    client: UnityCatalogClient,
    server: Option<TestServer>,
    context: JourneyContext,
}

impl JourneyExecutor {
    /// Create a new journey executor with a client and optional mock server
    pub fn new(client: UnityCatalogClient, server: Option<TestServer>) -> Self {
        Self {
            client,
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

    /// Set whether to continue on failure
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

        // Execute steps in dependency order
        let execution_order = self.resolve_execution_order(&journey.steps);

        for step in execution_order {
            // Check if dependencies are satisfied
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

                    let should_continue = step.continue_on_failure.unwrap_or(false)
                        || self.context.continue_on_failure;
                    if !should_continue {
                        break;
                    }
                    continue;
                }
            }

            // Execute the step
            let step_result = self.execute_step(&step).await;
            let step_success = step_result.success;

            step_results.push(step_result.clone());
            self.context
                .step_results
                .insert(step.id.clone(), step_result.clone());

            // Extract variables from successful steps
            if step_success {
                for (var_name, var_value) in step_result.extracted_variables {
                    self.context.variables.insert(var_name, var_value);
                }
            } else {
                overall_success = false;
                if let Some(error_msg) = &step_result.error_message {
                    failure_messages.push(format!("Step '{}': {}", step.id, error_msg));
                }

                // Check if we should continue on failure
                let should_continue =
                    step.continue_on_failure.unwrap_or(false) || self.context.continue_on_failure;
                if !should_continue {
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
        // Substitute variables in the step
        let resolved_step = self.substitute_variables_in_step(step);

        // Set up mock if we have a test server
        if let Some(server) = &self.server {
            self.setup_mock_for_step(server, &resolved_step);
        }

        // Execute the HTTP request
        match self.execute_http_request(&resolved_step).await {
            Ok((status_code, response_body)) => {
                let success = status_code == resolved_step.expected_status;
                let mut extracted_variables = HashMap::new();

                // Extract variables if the step was successful
                if success {
                    if let Some(extract_vars) = &resolved_step.extract_variables {
                        extracted_variables =
                            self.extract_variables_from_response(extract_vars, &response_body);
                    }
                }

                StepResult {
                    step: resolved_step.clone(),
                    success,
                    status_code,
                    response_body: Some(response_body),
                    error_message: if success {
                        None
                    } else {
                        Some(format!(
                            "Expected status {}, got {}",
                            resolved_step.expected_status, status_code
                        ))
                    },
                    extracted_variables,
                }
            }
            Err(e) => StepResult {
                step: resolved_step,
                success: false,
                status_code: 0,
                response_body: None,
                error_message: Some(e.to_string()),
                extracted_variables: HashMap::new(),
            },
        }
    }

    /// Resolve the execution order of steps based on dependencies
    fn resolve_execution_order(&self, steps: &[JourneyStep]) -> Vec<JourneyStep> {
        let mut ordered_steps = Vec::new();
        let mut remaining_steps: Vec<_> = steps.iter().cloned().collect();
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
                    "Circular dependency detected or missing steps. Remaining steps: {:?}",
                    remaining_steps.iter().map(|s| &s.id).collect::<Vec<_>>()
                );
                ordered_steps.extend(remaining_steps);
                break;
            }
        }

        ordered_steps
    }

    /// Set up mock response for a step
    fn setup_mock_for_step(&self, _server: &TestServer, step: &JourneyStep) {
        // This would set up the mock response based on the step's expected_response
        // Implementation depends on the mock server library being used
        tracing::debug!(
            "Setting up mock for step: {} {} {}",
            step.method,
            step.path,
            step.id
        );
    }

    /// Execute an HTTP request for a step
    async fn execute_http_request(&self, step: &JourneyStep) -> AcceptanceResult<(u16, Value)> {
        let client = reqwest::Client::new();

        // Build the URL
        let base_url = if let Some(server) = &self.server {
            server.url()
        } else {
            "http://localhost:8080" // Default UC server
        };

        let url = format!("{}{}", base_url, step.path);

        // Build the request
        let mut request_builder = match step.method.to_uppercase().as_str() {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
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

        // Parse response body
        let response_text = response.text().await?;
        let response_body: Value = if response_text.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&response_text).unwrap_or_else(|_| Value::String(response_text))
        };

        Ok((status_code, response_body))
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

    /// Substitute variables in JSON values
    fn substitute_variables_in_json(&self, value: &Value) -> Value {
        match value {
            Value::String(s) => Value::String(self.substitute_variables(s)),
            Value::Object(map) => {
                let mut new_map = Map::new();
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
            if let Some(extracted_value) = self.simple_json_path_extract(response, json_path) {
                extracted.insert(var_name.clone(), extracted_value);
            }
        }

        extracted
    }

    /// Simple JSONPath extraction (supports basic $.field notation)
    fn simple_json_path_extract(&self, value: &Value, path: &str) -> Option<Value> {
        if path == "$" {
            return Some(value.clone());
        }

        if path.starts_with("$.") {
            let field_path = &path[2..];
            let parts: Vec<&str> = field_path.split('.').collect();

            let mut current = value;
            for part in parts {
                match current {
                    Value::Object(map) => {
                        if let Some(next_value) = map.get(part) {
                            current = next_value;
                        } else {
                            return None;
                        }
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

/// Journey loader for reading journey definitions from files
pub struct JourneyLoader;

impl JourneyLoader {
    /// Load a journey from a JSON file
    pub fn load_journey(filename: &str) -> AcceptanceResult<UserJourney> {
        let path = format!("journeys/{}", filename);
        let content = std::fs::read_to_string(&path).map_err(|e| AcceptanceError::Io(e))?;

        let journey: UserJourney = serde_json::from_str(&content)?;
        Ok(journey)
    }

    /// Load all journeys from a directory
    pub fn load_all_journeys(dir: &str) -> AcceptanceResult<Vec<UserJourney>> {
        let mut journeys = Vec::new();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)?;
                let journey: UserJourney = serde_json::from_str(&content)?;
                journeys.push(journey);
            }
        }

        Ok(journeys)
    }

    /// Validate a journey for common issues
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
        for step in &journey.steps {
            if let Some(deps) = &step.depends_on {
                if deps.contains(&step.id) {
                    errors.push(format!(
                        "Step '{}' has circular dependency on itself",
                        step.id
                    ));
                }
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_variable_substitution() {
        let mut executor = JourneyExecutor::new(
            // We'll need to create a dummy client for testing
            unitycatalog_client::UnityCatalogClient::new(
                cloud_client::CloudClient::new_unauthenticated(),
                url::Url::parse("http://localhost:8080").unwrap(),
            ),
            None,
        );

        executor.context.variables.insert(
            "catalog_name".to_string(),
            Value::String("test_catalog".to_string()),
        );

        let template = "/api/2.1/unity-catalog/catalogs/{catalog_name}";
        let result = executor.substitute_variables(template);
        assert_eq!(result, "/api/2.1/unity-catalog/catalogs/test_catalog");
    }

    #[test]
    fn test_json_path_extraction() {
        let executor = JourneyExecutor::new(
            unitycatalog_client::UnityCatalogClient::new(
                cloud_client::CloudClient::new_unauthenticated(),
                url::Url::parse("http://localhost:8080").unwrap(),
            ),
            None,
        );

        let response = json!({
            "name": "test_catalog",
            "created_at": 1234567890,
            "properties": {
                "environment": "test"
            }
        });

        let result = executor.simple_json_path_extract(&response, "$.name");
        assert_eq!(result, Some(Value::String("test_catalog".to_string())));

        let result = executor.simple_json_path_extract(&response, "$.properties.environment");
        assert_eq!(result, Some(Value::String("test".to_string())));
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
                    method: "GET".to_string(),
                    path: "/test".to_string(),
                    request_body: None,
                    expected_status: 200,
                    expected_response: None,
                    extract_variables: None,
                    depends_on: None,
                    continue_on_failure: None,
                    tags: None,
                },
                JourneyStep {
                    id: "step2".to_string(),
                    description: "Second step".to_string(),
                    method: "POST".to_string(),
                    path: "/test".to_string(),
                    request_body: None,
                    expected_status: 201,
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
