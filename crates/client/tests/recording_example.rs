//! Example demonstrating the journey recording infrastructure
//!
//! This file shows how to use the recording system to capture real
//! Unity Catalog server responses for testing and validation.

use std::collections::HashMap;
use std::env;

use crate::integration_test_helper::{IntegrationConfig, IntegrationTestSetup};
use crate::journey_integration_recorder::{
    JourneyRecorder, RecordingConfig, record_journey_from_file,
};
use crate::test_utils::journeys::JourneyLoader;

mod integration_test_helper;
mod journey_integration_recorder;
mod test_utils;

/// Example 1: Basic recording usage with environment configuration
#[tokio::test]
async fn example_basic_recording() {
    println!("📖 Example 1: Basic Recording Usage");

    // Check if recording is configured
    let config = IntegrationConfig::from_env();

    if !config.is_recording_enabled() {
        println!("⏭️ Skipping - recording not enabled");
        println!("   To enable: export RECORD_JOURNEY_RESPONSES=true");
        println!("   Set server: export UC_SERVER_URL=http://localhost:8080");
        return;
    }

    println!("🎬 Recording is enabled!");
    println!("   Server: {:?}", config.server_url);
    println!("   Auth configured: {}", config.auth_token.is_some());

    // Record a simple journey
    match record_journey_from_file("simple_example.json").await {
        Ok(recorded) => {
            println!("✅ Recording successful!");
            println!("   Journey: {}", recorded.journey.name);
            println!("   Steps recorded: {}", recorded.recorded_steps.len());
            println!(
                "   Success rate: {}/{}",
                recorded.metadata.successful_steps, recorded.metadata.total_steps
            );

            // Show first recorded step as example
            if let Some(first_step) = recorded.recorded_steps.first() {
                println!(
                    "   First step: {} -> HTTP {}",
                    first_step.step.id, first_step.response.status_code
                );
            }
        }
        Err(e) => {
            println!("❌ Recording failed: {}", e);

            // Common failure scenarios
            if e.to_string().contains("UC_SERVER_URL") {
                println!("   💡 Set UC_SERVER_URL environment variable");
            } else if e.to_string().contains("connection") {
                println!("   💡 Check that Unity Catalog server is running");
            } else if e.to_string().contains("401") {
                println!("   💡 Check authentication token (UC_AUTH_TOKEN)");
            }
        }
    }
}

/// Example 2: Custom recording configuration
#[tokio::test]
async fn example_custom_recording_config() {
    println!("📖 Example 2: Custom Recording Configuration");

    // Only run if we have a server URL
    let server_url = match env::var("UC_SERVER_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("⏭️ Skipping - UC_SERVER_URL not set");
            return;
        }
    };

    // Create custom configuration
    let config = RecordingConfig {
        server_url: server_url.clone(),
        auth_token: env::var("UC_AUTH_TOKEN").ok(),
        output_dir: "target/test_recordings".into(),
        record_success_only: false, // Record errors too
        overwrite_existing: true,
        request_timeout_secs: 15,
    };

    println!("🔧 Custom configuration:");
    println!("   Server: {}", config.server_url);
    println!("   Output: {}", config.output_dir.display());
    println!("   Record errors: {}", !config.record_success_only);
    println!("   Timeout: {}s", config.request_timeout_secs);

    // Create recorder with custom variables
    let mut custom_variables = HashMap::new();
    custom_variables.insert(
        "environment".to_string(),
        serde_json::Value::String("example".to_string()),
    );
    custom_variables.insert(
        "run_id".to_string(),
        serde_json::Value::String(uuid::Uuid::new_v4().to_string()),
    );

    match JourneyRecorder::new(config) {
        Ok(recorder) => {
            let recorder = recorder.with_variables(custom_variables);
            println!("✅ Custom recorder created successfully");
            println!(
                "   Variables: {:?}",
                recorder.variables().keys().collect::<Vec<_>>()
            );
        }
        Err(e) => {
            println!("❌ Failed to create recorder: {}", e);
        }
    }
}

/// Example 3: Recording different journey types
#[tokio::test]
async fn example_recording_different_journeys() {
    println!("📖 Example 3: Recording Different Journey Types");

    let config = IntegrationConfig::from_env();

    if !config.is_recording_enabled() {
        println!("⏭️ Skipping - recording not enabled");
        return;
    }

    let journey_files = vec![
        "simple_example.json",
        "catalog_lifecycle.json",
        "hierarchical_data_structure.json",
        "error_handling.json",
    ];

    for journey_file in journey_files {
        println!("🎬 Recording journey: {}", journey_file);

        // Check if journey file exists
        match JourneyLoader::load_journey(journey_file) {
            Ok(journey) => {
                println!(
                    "   Loaded: {} ({} steps)",
                    journey.name,
                    journey.steps.len()
                );

                // In a real scenario, you'd record here
                // For this example, we just validate the journey
                let validation_errors = JourneyLoader::validate_journey(&journey);
                if validation_errors.is_empty() {
                    println!("   ✅ Journey is valid and ready for recording");
                } else {
                    println!("   ❌ Journey validation failed: {:?}", validation_errors);
                }
            }
            Err(e) => {
                println!("   ⚠️ Could not load journey: {}", e);
            }
        }
    }
}

/// Example 4: Integration test setup with recording support
#[tokio::test]
async fn example_integration_test_setup() {
    println!("📖 Example 4: Integration Test Setup");

    match IntegrationTestSetup::new().await {
        Ok(setup) => {
            println!("✅ Test setup created successfully");
            println!(
                "   Mode: {}",
                if setup.is_integration_mode() {
                    "Integration"
                } else {
                    "Mock"
                }
            );
            println!(
                "   Recording: {}",
                if setup.is_recording_enabled() {
                    "Enabled"
                } else {
                    "Disabled"
                }
            );

            // Show available test variables
            let variables = setup.create_test_variables();
            println!(
                "   Test variables: {:?}",
                variables.keys().collect::<Vec<_>>()
            );

            if setup.is_integration_mode() {
                println!("   🔗 Running against real server");
            } else {
                println!("   🎭 Running against mock server");
            }
        }
        Err(e) => {
            println!("❌ Failed to create test setup: {}", e);
        }
    }
}

/// Example 5: Environment configuration inspection
#[tokio::test]
async fn example_environment_inspection() {
    println!("📖 Example 5: Environment Configuration");

    // Check all relevant environment variables
    let env_vars = vec![
        "UC_SERVER_URL",
        "UC_AUTH_TOKEN",
        "RECORD_JOURNEY_RESPONSES",
        "OVERWRITE_JOURNEY_RESPONSES",
        "RECORD_SUCCESS_ONLY",
        "JOURNEY_RECORDING_DIR",
        "JOURNEY_REQUEST_TIMEOUT",
        "RUN_INTEGRATION_TESTS",
    ];

    println!("🔧 Environment Configuration:");
    for var in env_vars {
        match env::var(var) {
            Ok(value) => {
                // Mask sensitive values
                let display_value = if var.contains("TOKEN") || var.contains("AUTH") {
                    if value.is_empty() {
                        "not set"
                    } else {
                        "***masked***"
                    }
                } else {
                    &value
                };
                println!("   {}: {}", var, display_value);
            }
            Err(_) => {
                println!("   {}: not set", var);
            }
        }
    }

    // Show configuration status
    let config = IntegrationConfig::from_env();
    println!();
    println!("📊 Configuration Status:");
    println!(
        "   Integration tests: {}",
        if config.should_run_integration_tests() {
            "✅ Enabled"
        } else {
            "❌ Disabled"
        }
    );
    println!(
        "   Recording mode: {}",
        if config.is_recording_enabled() {
            "✅ Enabled"
        } else {
            "❌ Disabled"
        }
    );

    // Provide helpful setup instructions
    if !config.should_run_integration_tests() && !config.is_recording_enabled() {
        println!();
        println!("💡 To enable recording:");
        println!("   export UC_SERVER_URL=\"http://localhost:8080\"");
        println!("   export RECORD_JOURNEY_RESPONSES=true");
        println!("   # Optional: export UC_AUTH_TOKEN=\"your-token\"");
        println!("   cargo test recording_example -- --nocapture");
    }
}

/// Example 6: Recording workflow demonstration
#[tokio::test]
async fn example_complete_workflow() {
    println!("📖 Example 6: Complete Recording Workflow");

    let config = IntegrationConfig::from_env();

    println!("Step 1: Check configuration");
    if config.is_recording_enabled() {
        println!("   ✅ Recording is enabled");
    } else {
        println!("   ❌ Recording is disabled");
        println!("   This example will show the workflow structure only");
    }

    println!();
    println!("Step 2: Load journey definitions");
    let journey_files = ["simple_example.json", "catalog_lifecycle.json"];

    for journey_file in journey_files {
        match JourneyLoader::load_journey(journey_file) {
            Ok(journey) => {
                println!(
                    "   ✅ Loaded: {} ({} steps)",
                    journey.name,
                    journey.steps.len()
                );

                // Show journey structure
                println!(
                    "      Steps: {}",
                    journey
                        .steps
                        .iter()
                        .map(|s| s.id.as_str())
                        .collect::<Vec<_>>()
                        .join(" → ")
                );
            }
            Err(e) => {
                println!("   ❌ Failed to load {}: {}", journey_file, e);
            }
        }
    }

    println!();
    println!("Step 3: Recording execution");
    if config.is_recording_enabled() {
        println!(
            "   🎬 Would record journeys against: {:?}",
            config.server_url
        );
        println!("   💾 Would save to: tests/test_data/journeys/recorded/");
    } else {
        println!("   ⏭️ Skipped - recording not enabled");
    }

    println!();
    println!("Step 4: Verification");
    println!("   📁 Check output directory exists");
    std::fs::create_dir_all("tests/test_data/journeys/recorded").ok();

    if std::path::Path::new("tests/test_data/journeys/recorded").exists() {
        println!("   ✅ Output directory ready");
    } else {
        println!("   ❌ Could not create output directory");
    }

    println!();
    println!("🎉 Workflow demonstration complete!");

    if !config.is_recording_enabled() {
        println!();
        println!("💡 To run actual recording:");
        println!("   1. Start Unity Catalog server");
        println!("   2. export UC_SERVER_URL=\"http://localhost:8080\"");
        println!("   3. export RECORD_JOURNEY_RESPONSES=true");
        println!("   4. cargo test journey_tests -- --nocapture");
    }
}
