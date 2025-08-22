//! Example journeys demonstrating the simplified journey framework
//!
//! This module contains concrete journey implementations that show how to:
//! - Use the actual UnityCatalogClient for operations
//! - Record responses automatically
//! - Write clean, maintainable journey code
//! - Handle setup and cleanup properly

mod catalog_hierarchy;
mod catalog_simple;

pub use catalog_hierarchy::CatalogHierarchyJourney;
pub use catalog_simple::CatalogSimpleJourney;

use crate::execution::UserJourney;

/// Get all available example journeys
pub fn all_journeys() -> Vec<Box<dyn UserJourney>> {
    vec![
        Box::new(CatalogSimpleJourney::new()),
        Box::new(CatalogHierarchyJourney::new()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_journeys_registration() {
        let journeys = all_journeys();
        assert_eq!(journeys.len(), 2);

        let journey_names: Vec<&str> = journeys.iter().map(|j| j.name()).collect();
        assert!(journey_names.contains(&"enhanced_catalog"));
        assert!(journey_names.contains(&"catalog_hierarchy"));
    }

    #[test]
    fn test_journey_descriptions() {
        let journeys = all_journeys();

        for journey in journeys {
            assert!(!journey.name().is_empty());
            assert!(!journey.description().is_empty());
        }
    }
}
