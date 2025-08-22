//! Example journeys demonstrating the simplified journey framework
//!
//! This module contains concrete journey implementations that show how to:
//! - Use the actual UnityCatalogClient for operations
//! - Record responses automatically
//! - Write clean, maintainable journey code
//! - Handle setup and cleanup properly

mod catalog_simple;

pub use catalog_simple::CatalogSimpleJourney;

use crate::execution::UserJourney;

/// Get all available example journeys
pub fn all_journeys() -> Vec<Box<dyn UserJourney>> {
    vec![Box::new(CatalogSimpleJourney::new())]
}
