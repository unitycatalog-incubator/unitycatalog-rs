//! Integration Test Helper
//!
//! This module provides utilities for integration testing with real Unity Catalog servers,
//! including support for recording responses and configuring test environments.

use cloud_client::CloudClient;
use mockito::{Server, ServerGuard};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Once;
use unitycatalog_client::UnityCatalogClient;
use url::Url;

// Inline TestServer definition
pub struct TestServer {
    _server: ServerGuard,
    base_url: String,
}

impl TestServer {
    pub async fn new() -> Self {
        let server = Server::new_async().await;
        let base_url = server.url();

        Self {
            _server: server,
            base_url,
        }
    }

    pub fn create_client(&self) -> UnityCatalogClient {
        let cloud_client = CloudClient::new_unauthenticated();
        let base_url = Url::parse(&self.base_url).unwrap();
        UnityCatalogClient::new(cloud_client, base_url)
    }
}

// Inline JourneyRecorder stub (minimal version)
pub struct JourneyRecorder {
    _config: RecordingConfig,
}

impl JourneyRecorder {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let _config = RecordingConfig {
            server_url: env::var("UC_SERVER_URL").unwrap_or_default(),
            auth_token: env::var("UC_AUTH_TOKEN").ok(),
            output_dir: PathBuf::from("tests/test_data/journeys/recorded"),
            record_success_only: true,
            overwrite_existing: false,
            request_timeout_secs: 30,
        };
        Ok(Self { _config })
    }
}

#[derive(Debug, Clone)]
struct RecordingConfig {
    server_url: String,
    auth_token: Option<String>,
    output_dir: PathBuf,
    record_success_only: bool,
    overwrite_existing: bool,
    request_timeout_secs: u64,
}

static INIT: Once = Once::new();

/// Configuration for integration tests
#[derive(Debug, Clone)]
pub struct IntegrationConfig {
    /// Whether integration tests are enabled
    pub enabled: bool,
    /// Unity Catalog server URL
    pub server_url: Option<String>,
    /// Authentication token
    pub auth_token: Option<String>,
    /// Whether to record responses
    pub record_responses: bool,
    /// Whether to overwrite existing recordings
    pub overwrite_recordings: bool,
}

impl IntegrationConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let enabled = env::var("RUN_INTEGRATION_TESTS")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let server_url = env::var("UC_SERVER_URL").ok();
        let auth_token = env::var("UC_AUTH_TOKEN").ok();

        let record_responses = env::var("RECORD_JOURNEY_RESPONSES")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let overwrite_recordings = env::var("OVERWRITE_JOURNEY_RESPONSES")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        Self {
            enabled,
            server_url,
            auth_token,
            record_responses,
            overwrite_recordings,
        }
    }

    /// Check if integration tests should run
    pub fn should_run_integration_tests(&self) -> bool {
        self.enabled && self.server_url.is_some()
    }

    /// Check if recording mode is enabled
    pub fn is_recording_enabled(&self) -> bool {
        self.record_responses && self.server_url.is_some()
    }
}

/// Integration test setup that provides either mock or real server clients
pub struct IntegrationTestSetup {
    /// Unity Catalog client
    pub client: UnityCatalogClient,
    /// Mock server (if using mock mode)
    pub mock_server: Option<TestServer>,
    /// Configuration used
    pub config: IntegrationConfig,
    /// Journey recorder (if recording is enabled)
    pub recorder: Option<JourneyRecorder>,
}

impl IntegrationTestSetup {
    /// Create integration test setup based on environment configuration
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        init_test_logging();

        let config = IntegrationConfig::from_env();

        if config.should_run_integration_tests() {
            Self::new_integration(config).await
        } else {
            Self::new_mock(config).await
        }
    }

    /// Create setup for integration testing against real server
    async fn new_integration(
        config: IntegrationConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let server_url = config
            .server_url
            .as_ref()
            .ok_or("UC_SERVER_URL not set for integration tests")?;

        println!("🔗 Running integration tests against: {}", server_url);

        // Create authenticated client
        let cloud_client = if let Some(token) = &config.auth_token {
            println!("🔐 Using authentication token");
            CloudClient::new_with_token(token.clone())
        } else {
            println!("⚠️ Running without authentication");
            CloudClient::new_unauthenticated()
        };

        let base_url = Url::parse(server_url)?;
        let client = UnityCatalogClient::new(cloud_client, base_url);

        // Create recorder if recording is enabled
        let recorder = if config.is_recording_enabled() {
            println!("🎬 Recording mode enabled");
            Some(JourneyRecorder::from_env()?)
        } else {
            None
        };

        Ok(Self {
            client,
            mock_server: None,
            config,
            recorder,
        })
    }

    /// Create setup for mock testing
    async fn new_mock(config: IntegrationConfig) -> Result<Self, Box<dyn std::error::Error>> {
        println!("🎭 Running tests with mock server");

        let mock_server = TestServer::new().await;
        let client = mock_server.create_client();

        Ok(Self {
            client,
            mock_server: Some(mock_server),
            config,
            recorder: None,
        })
    }

    /// Get the Unity Catalog client
    pub fn client(&self) -> &UnityCatalogClient {
        &self.client
    }

    /// Get the mock server (if available)
    pub fn mock_server(&self) -> Option<&TestServer> {
        self.mock_server.as_ref()
    }

    /// Check if running in integration mode
    pub fn is_integration_mode(&self) -> bool {
        self.mock_server.is_none()
    }

    /// Check if recording is enabled
    pub fn is_recording_enabled(&self) -> bool {
        self.recorder.is_some()
    }

    /// Get a mutable reference to the recorder
    pub fn recorder_mut(&mut self) -> Option<&mut JourneyRecorder> {
        self.recorder.as_mut()
    }

    /// Create test variables with common defaults
    pub fn create_test_variables(&self) -> HashMap<String, Value> {
        let mut variables = HashMap::new();

        // Add timestamp for unique naming
        let timestamp = chrono::Utc::now().timestamp();
        variables.insert(
            "timestamp".to_string(),
            Value::String(timestamp.to_string()),
        );

        // Add random suffix
        let suffix = uuid::Uuid::new_v4().to_string()[..8].to_string();
        variables.insert("test_suffix".to_string(), Value::String(suffix));

        // Add environment indicator
        let env_name = if self.is_integration_mode() {
            "integration"
        } else {
            "mock"
        };
        variables.insert("test_env".to_string(), Value::String(env_name.to_string()));

        variables
    }
}

/// Initialize test logging - call this once per test run
fn init_test_logging() {
    INIT.call_once(|| {
        // Initialize logging if not already done
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .is_test(true)
            .try_init();
    });
}

/// Helper function to skip integration tests if not configured
pub fn skip_if_no_integration_config() {
    let config = IntegrationConfig::from_env();
    if !config.should_run_integration_tests() {
        println!(
            "⏭️ Skipping integration test - UC_SERVER_URL not set or RUN_INTEGRATION_TESTS=false"
        );
        return;
    }
}

/// Macro to conditionally run integration tests
#[macro_export]
macro_rules! integration_test {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let config = crate::integration_test_helper::IntegrationConfig::from_env();
            if !config.should_run_integration_tests() {
                println!("⏭️ Skipping integration test - not configured");
                return;
            }
            $test_body
        }
    };
}

/// Fixture for rstest that provides integration test setup
pub async fn integration_test_setup() -> IntegrationTestSetup {
    IntegrationTestSetup::new()
        .await
        .expect("Failed to create integration test setup")
}

/// Fixture for rstest that provides test variables
pub fn test_variables() -> HashMap<String, Value> {
    let mut variables = HashMap::new();

    // Common test variables
    let timestamp = chrono::Utc::now().timestamp();
    variables.insert(
        "timestamp".to_string(),
        Value::String(timestamp.to_string()),
    );

    let suffix = uuid::Uuid::new_v4().to_string()[..8].to_string();
    variables.insert("test_suffix".to_string(), Value::String(suffix.clone()));

    // Test-specific catalog names
    variables.insert(
        "test_catalog_name".to_string(),
        Value::String(format!("test_catalog_{}", suffix)),
    );

    variables.insert(
        "test_schema_name".to_string(),
        Value::String(format!("test_schema_{}", suffix)),
    );

    variables.insert(
        "test_table_name".to_string(),
        Value::String(format!("test_table_{}", suffix)),
    );

    variables
}

/// Environment variable helper functions
pub mod env_helpers {
    use std::env;

    /// Set environment variable for testing
    pub fn set_test_env_var(key: &str, value: &str) {
        unsafe {
            env::set_var(key, value);
        }
    }

    /// Remove environment variable for testing
    pub fn remove_test_env_var(key: &str) {
        unsafe {
            env::remove_var(key);
        }
    }

    /// Temporarily set environment variable for a test
    pub fn with_env_var<F, R>(key: &str, value: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let original = env::var(key).ok();
        unsafe {
            env::set_var(key, value);
        }

        let result = f();

        match original {
            Some(val) => unsafe { env::set_var(key, val) },
            None => unsafe { env::remove_var(key) },
        }

        result
    }

    /// Get required environment variable or panic with helpful message
    pub fn require_env_var(key: &str) -> String {
        env::var(key).unwrap_or_else(|_| {
            panic!(
                "Required environment variable {} not set. \
                 Set it with: export {}=<value>",
                key, key
            )
        })
    }

    /// Check if integration testing is enabled
    pub fn is_integration_enabled() -> bool {
        env::var("RUN_INTEGRATION_TESTS")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false)
    }

    /// Print environment configuration for debugging
    pub fn print_env_config() {
        println!("🔧 Environment Configuration:");
        println!(
            "  RUN_INTEGRATION_TESTS: {}",
            env::var("RUN_INTEGRATION_TESTS").unwrap_or("not set".to_string())
        );
        println!(
            "  UC_SERVER_URL: {}",
            env::var("UC_SERVER_URL").unwrap_or("not set".to_string())
        );
        println!(
            "  UC_AUTH_TOKEN: {}",
            if env::var("UC_AUTH_TOKEN").is_ok() {
                "set"
            } else {
                "not set"
            }
        );
        println!(
            "  RECORD_JOURNEY_RESPONSES: {}",
            env::var("RECORD_JOURNEY_RESPONSES").unwrap_or("not set".to_string())
        );
        println!(
            "  OVERWRITE_JOURNEY_RESPONSES: {}",
            env::var("OVERWRITE_JOURNEY_RESPONSES").unwrap_or("not set".to_string())
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_config_from_env() {
        // Test with defaults
        env_helpers::remove_test_env_var("RUN_INTEGRATION_TESTS");
        env_helpers::remove_test_env_var("UC_SERVER_URL");

        let config = IntegrationConfig::from_env();
        assert!(!config.enabled);
        assert!(config.server_url.is_none());
        assert!(!config.should_run_integration_tests());

        // Test with integration enabled
        env_helpers::set_test_env_var("RUN_INTEGRATION_TESTS", "true");
        env_helpers::set_test_env_var("UC_SERVER_URL", "http://test.example.com");

        let config = IntegrationConfig::from_env();
        assert!(config.enabled);
        assert_eq!(
            config.server_url,
            Some("http://test.example.com".to_string())
        );
        assert!(config.should_run_integration_tests());

        // Clean up
        env_helpers::remove_test_env_var("RUN_INTEGRATION_TESTS");
        env_helpers::remove_test_env_var("UC_SERVER_URL");
    }

    #[test]
    fn test_env_helpers() {
        let key = "TEST_ENV_VAR";
        let value = "test_value";

        // Test setting and removing
        env_helpers::set_test_env_var(key, value);
        assert_eq!(env::var(key).unwrap(), value);

        env_helpers::remove_test_env_var(key);
        assert!(env::var(key).is_err());

        // Test with_env_var
        let result = env_helpers::with_env_var(key, value, || env::var(key).unwrap());
        assert_eq!(result, value);
        assert!(env::var(key).is_err()); // Should be cleaned up
    }

    #[tokio::test]
    async fn test_mock_setup() {
        // Force mock mode by not setting integration env vars
        env_helpers::remove_test_env_var("RUN_INTEGRATION_TESTS");
        env_helpers::remove_test_env_var("UC_SERVER_URL");

        let setup = IntegrationTestSetup::new().await.unwrap();
        assert!(!setup.is_integration_mode());
        assert!(setup.mock_server().is_some());
        assert!(!setup.is_recording_enabled());
    }
}
