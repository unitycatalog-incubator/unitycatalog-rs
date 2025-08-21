//! Integration tests for the simplified journey framework
//!
//! These tests demonstrate how to use the new journey framework and serve as
//! both validation and documentation for the system.
//!
//! To run these tests:
//! ```bash
//! # Against a mock server (fast, default)
//! cargo test
//!
//! # Against a real Unity Catalog server (set environment variables)
//! export RUN_INTEGRATION_TESTS=true
//! export UC_SERVER_URL="http://localhost:8080"
//! export UC_AUTH_TOKEN="your-token"  # Optional
//! export RECORD_JOURNEY_RESPONSES="true"
//! cargo test
//! ```

use std::env;
use tempfile::TempDir;
use unitycatalog_acceptance::{
    AcceptanceResult,
    journeys::SimpleCatalogJourney,
    simple_journey::{JourneyConfig, UserJourney},
};

/// Check if integration tests should run
fn should_run_integration_tests() -> bool {
    env::var("RUN_INTEGRATION_TESTS").unwrap_or_default() == "true"
}

/// Create a test configuration with temporary directory
fn create_test_config() -> (JourneyConfig, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let config = if should_run_integration_tests() {
        JourneyConfig {
            recording_enabled: true,
            output_dir: temp_dir.path().to_path_buf(),
            server_url: env::var("UC_SERVER_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            auth_token: env::var("UC_AUTH_TOKEN").ok(),
            timeout_seconds: 30,
        }
    } else {
        // Use mock server configuration for unit tests
        JourneyConfig {
            recording_enabled: false, // Disable recording in unit tests
            output_dir: temp_dir.path().to_path_buf(),
            server_url: "http://mock-server:8080".to_string(),
            auth_token: None,
            timeout_seconds: 5,
        }
    };

    (config, temp_dir)
}

#[tokio::test]
async fn test_simple_catalog_journey() -> AcceptanceResult<()> {
    if !should_run_integration_tests() {
        println!("⏭️  Skipping integration test (RUN_INTEGRATION_TESTS not set)");
        return Ok(());
    }

    let (config, _temp_dir) = create_test_config();
    let executor = config.create_executor()?;

    let journey = SimpleCatalogJourney::new();
    let result = executor.execute_journey(&journey).await?;

    assert!(
        result.is_success(),
        "Simple catalog journey should succeed: {}",
        result.summary()
    );
    assert!(
        result.steps_completed > 0,
        "Should have completed some steps"
    );

    println!("✅ Simple catalog journey test passed");
    Ok(())
}

#[tokio::test]
async fn test_multiple_journeys_execution() -> AcceptanceResult<()> {
    if !should_run_integration_tests() {
        println!("⏭️  Skipping integration test (RUN_INTEGRATION_TESTS not set)");
        return Ok(());
    }

    let (config, _temp_dir) = create_test_config();
    let executor = config.create_executor()?;

    let journeys: Vec<Box<dyn UserJourney>> = vec![
        Box::new(SimpleCatalogJourney::with_catalog_name(
            "multi_test_catalog_1",
        )),
        Box::new(SimpleCatalogJourney::with_catalog_name(
            "multi_test_catalog_2",
        )),
    ];

    let journey_refs: Vec<&dyn UserJourney> = journeys.iter().map(|j| j.as_ref()).collect();
    let results = executor.execute_journeys(journey_refs).await?;

    assert_eq!(results.len(), 2, "Should have results for both journeys");

    for result in &results {
        assert!(
            result.is_success(),
            "Journey '{}' should succeed: {}",
            result.journey_name,
            result.summary()
        );
    }

    println!("✅ Multiple journeys execution test passed");
    Ok(())
}

#[tokio::test]
async fn test_journey_recording() -> AcceptanceResult<()> {
    let (mut config, temp_dir) = create_test_config();
    config.recording_enabled = true; // Force recording for this test

    // Create a mock journey for testing recording
    struct RecordingTestJourney;

    #[async_trait::async_trait]
    impl UserJourney for RecordingTestJourney {
        fn name(&self) -> &str {
            "recording_test"
        }

        fn description(&self) -> &str {
            "Test journey for recording functionality"
        }

        async fn execute(
            &self,
            _client: &unitycatalog_client::UnityCatalogClient,
            recorder: &mut unitycatalog_acceptance::simple_journey::JourneyRecorder,
        ) -> AcceptanceResult<()> {
            // Record some test data
            recorder
                .record_step(
                    "test_step_1",
                    "First test step",
                    &serde_json::json!({"test": "data1", "step": 1}),
                )
                .await?;

            recorder
                .record_step(
                    "test_step_2",
                    "Second test step",
                    &serde_json::json!({"test": "data2", "step": 2}),
                )
                .await?;

            Ok(())
        }
    }

    if should_run_integration_tests() {
        let executor = config.create_executor()?;
        let journey = RecordingTestJourney;
        let result = executor.execute_journey(&journey).await?;

        assert!(result.is_success(), "Recording test journey should succeed");

        // Check that files were created
        let journey_dir = temp_dir.path().join("recording_test");
        assert!(journey_dir.exists(), "Journey directory should be created");

        let step1_file = journey_dir.join("001_test_step_1.json");
        let step2_file = journey_dir.join("002_test_step_2.json");
        let summary_file = journey_dir.join("journey_summary.json");

        assert!(step1_file.exists(), "Step 1 file should be created");
        assert!(step2_file.exists(), "Step 2 file should be created");
        assert!(summary_file.exists(), "Summary file should be created");

        // Verify file contents
        let step1_content = tokio::fs::read_to_string(&step1_file).await?;
        let step1_json: serde_json::Value = serde_json::from_str(&step1_content)?;
        assert_eq!(step1_json["test"], "data1");
        assert_eq!(step1_json["step"], 1);

        println!("✅ Journey recording test passed");
    } else {
        println!("⏭️  Skipping recording test (RUN_INTEGRATION_TESTS not set)");
    }

    Ok(())
}

#[tokio::test]
async fn test_journey_error_handling() -> AcceptanceResult<()> {
    let (config, _temp_dir) = create_test_config();

    // Create a journey that will fail
    struct FailingJourney;

    #[async_trait::async_trait]
    impl UserJourney for FailingJourney {
        fn name(&self) -> &str {
            "failing_test"
        }

        fn description(&self) -> &str {
            "Test journey that intentionally fails"
        }

        async fn execute(
            &self,
            _client: &unitycatalog_client::UnityCatalogClient,
            recorder: &mut unitycatalog_acceptance::simple_journey::JourneyRecorder,
        ) -> AcceptanceResult<()> {
            // Record a successful step first
            recorder
                .record_step(
                    "success_step",
                    "Successful step before failure",
                    &serde_json::json!({"status": "success"}),
                )
                .await?;

            // Then fail
            Err(unitycatalog_acceptance::AcceptanceError::JourneyExecution(
                "Intentional test failure".to_string(),
            ))
        }
    }

    if should_run_integration_tests() {
        let executor = config.create_executor()?;
        let journey = FailingJourney;
        let result = executor.execute_journey(&journey).await?;

        assert!(!result.is_success(), "Failing journey should not succeed");
        assert!(result.error_message.is_some(), "Should have error message");
        assert!(
            result.steps_completed > 0,
            "Should have completed some steps before failing"
        );

        println!("✅ Journey error handling test passed");
    } else {
        println!("⏭️  Skipping error handling test (RUN_INTEGRATION_TESTS not set)");
    }

    Ok(())
}

#[test]
fn test_journey_properties() {
    // Test that journey implementations have correct properties
    let catalog_journey = SimpleCatalogJourney::new();
    assert_eq!(catalog_journey.name(), "simple_catalog_example");
    assert!(!catalog_journey.description().is_empty());
    assert!(catalog_journey.tags().contains(&"catalog"));
    assert!(catalog_journey.tags().contains(&"simple"));
    assert!(catalog_journey.tags().contains(&"example"));

    println!("✅ Journey properties test passed");
}

#[test]
fn test_journey_config_from_env() {
    // Test configuration creation from environment variables
    let original_env: Vec<(String, Option<String>)> = vec![
        "RECORD_JOURNEY_RESPONSES",
        "JOURNEY_RECORDING_DIR",
        "UC_SERVER_URL",
        "UC_AUTH_TOKEN",
        "REQUEST_TIMEOUT_SECS",
    ]
    .into_iter()
    .map(|key| (key.to_string(), env::var(key).ok()))
    .collect();

    // Set test environment variables
    unsafe {
        env::set_var("RECORD_JOURNEY_RESPONSES", "true");
        env::set_var("JOURNEY_RECORDING_DIR", "/tmp/test_recordings");
        env::set_var("UC_SERVER_URL", "http://test-server:9090");
        env::set_var("UC_AUTH_TOKEN", "test-token");
        env::set_var("REQUEST_TIMEOUT_SECS", "45");
    }

    let config = JourneyConfig::default();

    assert!(config.recording_enabled);
    assert_eq!(config.output_dir.to_string_lossy(), "/tmp/test_recordings");
    assert_eq!(config.server_url, "http://test-server:9090");
    assert_eq!(config.auth_token, Some("test-token".to_string()));
    assert_eq!(config.timeout_seconds, 45);

    // Restore original environment
    unsafe {
        for (key, value) in original_env {
            match value {
                Some(val) => env::set_var(&key, val),
                None => env::remove_var(&key),
            }
        }
    }

    println!("✅ Journey config from environment test passed");
}

#[tokio::test]
async fn test_concurrent_journeys() -> AcceptanceResult<()> {
    if !should_run_integration_tests() {
        println!("⏭️  Skipping integration test (RUN_INTEGRATION_TESTS not set)");
        return Ok(());
    }

    let (config, _temp_dir) = create_test_config();
    let executor = std::sync::Arc::new(config.create_executor()?);

    // Run multiple journeys concurrently
    let mut handles = Vec::new();

    for i in 0..3 {
        let executor = executor.clone();
        let handle = tokio::spawn(async move {
            let journey = SimpleCatalogJourney::with_catalog_name(format!("concurrent_test_{}", i));
            executor.execute_journey(&journey).await
        });
        handles.push(handle);
    }

    // Wait for all journeys to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap()?;
        results.push(result);
    }

    // Verify all succeeded
    for (i, result) in results.iter().enumerate() {
        assert!(
            result.is_success(),
            "Concurrent journey {} should succeed: {}",
            i,
            result.summary()
        );
    }

    println!("✅ Concurrent journeys test passed");
    Ok(())
}

/// Integration test helper to print environment information
#[tokio::test]
async fn test_environment_info() {
    println!("🔧 Test Environment Information");
    println!("==============================");
    println!(
        "RUN_INTEGRATION_TESTS: {}",
        env::var("RUN_INTEGRATION_TESTS").unwrap_or("not set".to_string())
    );
    println!(
        "UC_SERVER_URL: {}",
        env::var("UC_SERVER_URL").unwrap_or("not set".to_string())
    );
    println!(
        "UC_AUTH_TOKEN: {}",
        if env::var("UC_AUTH_TOKEN").is_ok() {
            "set"
        } else {
            "not set"
        }
    );
    println!(
        "RECORD_JOURNEY_RESPONSES: {}",
        env::var("RECORD_JOURNEY_RESPONSES").unwrap_or("not set".to_string())
    );
    println!(
        "JOURNEY_RECORDING_DIR: {}",
        env::var("JOURNEY_RECORDING_DIR").unwrap_or("not set".to_string())
    );

    if should_run_integration_tests() {
        println!("🚀 Integration tests will run against Unity Catalog server");
    } else {
        println!(
            "🏃 Only unit tests will run (set RUN_INTEGRATION_TESTS=true for integration tests)"
        );
    }
}
