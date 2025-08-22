use unitycatalog_acceptance::JourneyConfig;
use unitycatalog_acceptance::journeys::all_journeys;

#[tokio::test]
async fn journey_tests() -> Result<(), Box<dyn std::error::Error>> {
    let cargo_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let path = std::path::Path::new(&cargo_dir).join("recordings");

    let config = JourneyConfig::default()
        .with_recording(false)
        .with_output_dir(path);

    for journey in all_journeys().iter_mut() {
        let result = config.execute_journey(journey.as_mut()).await?;
        assert!(result.is_success());
    }

    Ok(())
}
