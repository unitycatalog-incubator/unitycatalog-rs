use unitycatalog_acceptance::AcceptanceResult;
use unitycatalog_acceptance::journeys::SimpleCatalogJourney;
use unitycatalog_acceptance::simple_journey::JourneyConfig;

#[tokio::main]
async fn main() -> AcceptanceResult<()> {
    dotenv::dotenv().ok();

    let config = JourneyConfig::default();

    let mut journey = SimpleCatalogJourney::new();
    let result = config.execute_journey(&mut journey).await?;

    assert!(result.is_success());

    Ok(())
}
