//! Simple Framework Test
//!
//! A basic test demonstrating the new simplified journey framework
//! using only working components.

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
        // Minimal configuration for unit tests
        JourneyConfig {
            recording_enabled: false,
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
async fn test_multiple_simple_journeys() -> AcceptanceResult<()> {
    if !should_run_integration_tests() {
        println!("⏭️  Skipping integration test (RUN_INTEGRATION_TESTS not set)");
        return Ok(());
    }

    let (config, _temp_dir) = create_test_config();
    let executor = config.create_executor()?;

    let journeys: Vec<Box<dyn UserJourney>> = vec![
        Box::new(SimpleCatalogJourney::with_catalog_name("test_catalog_1")),
        Box::new(SimpleCatalogJourney::with_catalog_name("test_catalog_2")),
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

    if should_run_integration_tests() {
        let executor = config.create_executor()?;
        let journey = SimpleCatalogJourney::with_catalog_name("recording_test_catalog");
        let result = executor.execute_journey(&journey).await?;

        assert!(result.is_success(), "Recording test journey should succeed");

        // Check that files were created
        let journey_dir = temp_dir.path().join("simple_catalog_example");
        assert!(journey_dir.exists(), "Journey directory should be created");

        let summary_file = journey_dir.join("journey_summary.json");
        assert!(summary_file.exists(), "Summary file should be created");

        // Check for at least one step file
        let files = std::fs::read_dir(&journey_dir)?;
        let step_files: Vec<_> = files
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("001_"))
            .collect();

        assert!(!step_files.is_empty(), "Should have at least one step file");

        println!("✅ Journey recording test passed");
    } else {
        println!("⏭️  Skipping recording test (RUN_INTEGRATION_TESTS not set)");
    }

    Ok(())
}

#[test]
fn test_journey_properties() {
    // Test that journey implementations have correct properties
    let journey = SimpleCatalogJourney::new();
    assert_eq!(journey.name(), "simple_catalog_example");
    assert!(!journey.description().is_empty());
    assert!(journey.tags().contains(&"catalog"));
    assert!(journey.tags().contains(&"simple"));
    assert!(journey.tags().contains(&"example"));

    println!("✅ Journey properties test passed");
}

#[test]
fn test_journey_config_defaults() {
    // Test configuration creation with defaults
    let config = JourneyConfig::default();

    // These should have sensible defaults
    assert!(config.timeout_seconds > 0);
    assert!(!config.server_url.is_empty());

    println!("✅ Journey config defaults test passed");
}

#[test]
fn test_custom_catalog_name() {
    let journey = SimpleCatalogJourney::with_catalog_name("my_custom_catalog");
    assert_eq!(journey.catalog_name(), "my_custom_catalog");
    assert_eq!(journey.name(), "simple_catalog_example");

    println!("✅ Custom catalog name test passed");
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

    if should_run_integration_tests() {
        println!("🚀 Integration tests will run against Unity Catalog server");
    } else {
        println!(
            "🏃 Only unit tests will run (set RUN_INTEGRATION_TESTS=true for integration tests)"
        );
    }
}
