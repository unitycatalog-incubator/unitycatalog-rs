//! Example journeys demonstrating the simplified journey framework
//!
//! This module contains concrete journey implementations that show how to:
//! - Use the actual UnityCatalogClient for operations
//! - Record responses automatically
//! - Write clean, maintainable journey code
//! - Handle setup and cleanup properly

pub mod simple_catalog;
pub use simple_catalog::SimpleCatalogJourney;

use crate::simple_journey::UserJourney;

/// Get all available example journeys
pub fn all_journeys() -> Vec<Box<dyn UserJourney>> {
    vec![Box::new(SimpleCatalogJourney::new())]
}

/// Get journeys by tag
pub fn journeys_with_tag(tag: &str) -> Vec<Box<dyn UserJourney>> {
    all_journeys()
        .into_iter()
        .filter(|journey| journey.tags().contains(&tag))
        .collect()
}

/// Get smoke test journeys (quick validation)
pub fn smoke_test_journeys() -> Vec<Box<dyn UserJourney>> {
    vec![Box::new(SimpleCatalogJourney::new())]
}

/// Get comprehensive integration test journeys
pub fn integration_test_journeys() -> Vec<Box<dyn UserJourney>> {
    journeys_with_tag("integration")
}
