use unitycatalog_acceptance::journeys::{journeys_for_filter, journeys_for_profile};
use unitycatalog_acceptance::{ImplementationProfile, JourneyConfig, JourneyFilter};

/// Default replay test — runs all journeys that have recordings.
///
/// Journeys without a recordings directory are skipped gracefully.
/// Use environment variables to narrow the run:
/// - `UC_JOURNEY_INCLUDE` — comma-separated journey names to run
/// - `UC_JOURNEY_EXCLUDE` — comma-separated journey names to skip
/// - `UC_JOURNEY_IMPL`    — filter by implementation: oss_rust, oss_java, managed_databricks
/// - `UC_JOURNEY_MAX_TIER` — filter by tier: tier1, tier2, tier3, tier4
#[tokio::test]
async fn journey_tests() -> Result<(), Box<dyn std::error::Error>> {
    let cargo_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let path = std::path::Path::new(&cargo_dir).join("recordings");

    let config = JourneyConfig::default()
        .with_recording(false)
        .with_output_dir(path);

    let filter = JourneyFilter::from_env();

    for journey in journeys_for_filter(&filter).iter_mut() {
        let result = config.execute_journey(journey.as_mut()).await?;
        assert!(
            result.is_success(),
            "Journey '{}' failed: {:?}",
            journey.name(),
            result.error_message
        );
    }

    Ok(())
}

/// Live integration test — only runs when `UC_INTEGRATION_PROFILE` is set.
///
/// Examples:
/// ```bash
/// # Run against the local Rust OSS server
/// UC_INTEGRATION_PROFILE=oss_rust UC_INTEGRATION_URL=http://localhost:8080 \
///   cargo test -p unitycatalog-acceptance -- journey_tests_live
///
/// # Record against Databricks managed UC
/// UC_INTEGRATION_PROFILE=managed_databricks \
///   UC_INTEGRATION_URL=https://my-workspace.azuredatabricks.net \
///   UC_INTEGRATION_TOKEN=dapi... \
///   UC_INTEGRATION_STORAGE_ROOT=s3://my-bucket/uc-test/ \
///   UC_INTEGRATION_RECORD=true \
///   cargo test -p unitycatalog-acceptance -- journey_tests_live
/// ```
#[tokio::test]
async fn journey_tests_live() -> Result<(), Box<dyn std::error::Error>> {
    let profile_name = match std::env::var("UC_INTEGRATION_PROFILE") {
        Ok(v) => v,
        Err(_) => return Ok(()), // skip when not configured
    };

    let base_profile = match profile_name.as_str() {
        "oss_rust" => ImplementationProfile::oss_rust(
            std::env::var("UC_INTEGRATION_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
        ),
        "managed_databricks" => ImplementationProfile::managed_databricks(
            std::env::var("UC_INTEGRATION_URL")
                .expect("UC_INTEGRATION_URL required for managed_databricks profile"),
            std::env::var("UC_INTEGRATION_TOKEN")
                .expect("UC_INTEGRATION_TOKEN required for managed_databricks profile"),
            std::env::var("UC_INTEGRATION_STORAGE_ROOT").unwrap_or_else(|_| "s3://".to_string()),
        ),
        other => return Err(format!("Unknown UC_INTEGRATION_PROFILE: {}", other).into()),
    };

    let profile = ImplementationProfile::from_env(base_profile);
    let recording_enabled = std::env::var("UC_INTEGRATION_RECORD").unwrap_or_default() == "true";

    let cargo_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let path = std::path::Path::new(&cargo_dir).join("recordings");

    let config = JourneyConfig::for_profile(&profile)
        .with_recording(recording_enabled)
        .with_output_dir(path);

    for journey in journeys_for_profile(&profile).iter_mut() {
        let result = config.execute_journey(journey.as_mut()).await?;
        assert!(
            result.is_success(),
            "Journey '{}' failed: {:?}",
            journey.name(),
            result.error_message
        );
    }

    Ok(())
}
