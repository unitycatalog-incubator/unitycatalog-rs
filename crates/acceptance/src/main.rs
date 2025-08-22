use unitycatalog_acceptance::AcceptanceResult;
use unitycatalog_acceptance::journey::JourneyConfig;
use unitycatalog_acceptance::journeys::all_journeys;

#[tokio::main]
async fn main() -> AcceptanceResult<()> {
    dotenv::dotenv().ok();

    let config = JourneyConfig::default();

    for journey in all_journeys().iter_mut() {
        let result = config.execute_journey(journey.as_mut()).await?;
        assert!(result.is_success());
    }

    Ok(())
}
