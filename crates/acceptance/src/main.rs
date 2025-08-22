use unitycatalog_acceptance::AcceptanceResult;
use unitycatalog_acceptance::journeys::SimpleCatalogJourney;
use unitycatalog_acceptance::simple_journey::{JourneyConfig, UserJourney};

#[tokio::main]
async fn main() -> AcceptanceResult<()> {
    dotenv::dotenv().ok();

    let config = JourneyConfig::default();

    let journey = SimpleCatalogJourney::new();
    let executor = config.create_executor(journey.name()).await?;
    let result = executor.execute_journey(&journey).await?;

    assert!(result.is_success());

    Ok(())
}
