use unitycatalog_acceptance::journeys::SimpleCatalogJourney;
use unitycatalog_acceptance::{AcceptanceResult, JourneyConfig};

#[tokio::main]
async fn main() -> AcceptanceResult<()> {
    dotenv::dotenv().ok();

    let config = JourneyConfig::default();

    let executor = config.create_executor()?;
    let journey = SimpleCatalogJourney::new();
    let result = executor.execute_journey(&journey).await?;

    assert!(result.is_success());

    Ok(())
}
