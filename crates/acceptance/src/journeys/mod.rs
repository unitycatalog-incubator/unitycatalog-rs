//! Journey implementations organized by tier and resource type
//!
//! Journeys are grouped into tiers based on complexity and dependencies:
//!
//! - **Tier 1**: Basic CRUD — compatible with all implementations, no external dependencies
//! - **Tier 2**: Governance — credentials, external locations, volumes, temporary credentials
//! - **Tier 3**: Delta Sharing — shares and recipients
//! - **Tier 4**: Advanced — UDFs and cross-resource workflows

mod cross_resource;
mod tier1;
mod tier2;
mod tier3;
mod tier4;

use crate::execution::{ImplementationProfile, JourneyFilter, UserJourney};

/// All available journeys, unfiltered.
///
/// Journeys without recordings will be skipped gracefully in replay mode —
/// they must be recorded against a live server before their replay tests activate.
/// See `JOURNEY_CATALOG.md` for the full list and recording status.
pub fn all_journeys() -> Vec<Box<dyn UserJourney>> {
    vec![
        // ── Tier 1: Basic CRUD ──────────────────────────────────────────────
        Box::new(tier1::CatalogSimpleJourney::new()),
        Box::new(tier1::CatalogHierarchyJourney::new()),
        Box::new(tier1::SchemaLifecycleJourney::new()),
        Box::new(tier1::TableManagedLifecycleJourney::new()),
        Box::new(tier1::MetricViewLifecycleJourney::new()),
        // ── Tier 2: Governance ──────────────────────────────────────────────
        Box::new(tier2::CredentialLifecycleJourney::new()),
        Box::new(tier2::VolumeManagedLifecycleJourney::new()),
        Box::new(tier2::ExternalLocationLifecycleJourney::new()),
        Box::new(tier2::VolumeExternalLifecycleJourney::new()),
        Box::new(tier2::TableExternalLifecycleJourney::new()),
        Box::new(tier2::TemporaryTableCredentialsJourney::new()),
        Box::new(tier2::TemporaryVolumeCredentialsJourney::new()),
        Box::new(tier2::TemporaryPathCredentialsJourney::new()),
        // ── Tier 3: Delta Sharing ────────────────────────────────────────────
        Box::new(tier3::ShareLifecycleJourney::new()),
        Box::new(tier3::RecipientLifecycleJourney::new()),
        Box::new(tier3::ProviderLifecycleJourney::new()),
        // ── Tier 4: Advanced ─────────────────────────────────────────────────
        Box::new(tier4::FunctionLifecycleJourney::new()),
        Box::new(cross_resource::LakehouseHierarchyJourney::new()),
        Box::new(cross_resource::GovernanceSetupJourney::new()),
    ]
}

/// Journeys filtered by a [`JourneyFilter`].
pub fn journeys_for_filter(filter: &JourneyFilter) -> Vec<Box<dyn UserJourney>> {
    all_journeys()
        .into_iter()
        .filter(|j| filter.matches(j.as_ref()))
        .collect()
}

/// Journeys compatible with the given [`ImplementationProfile`].
pub fn journeys_for_profile(profile: &ImplementationProfile) -> Vec<Box<dyn UserJourney>> {
    journeys_for_filter(&profile.filter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_journeys_registration() {
        let journeys = all_journeys();
        // Must have at least the two original journeys
        let names: Vec<&str> = journeys.iter().map(|j| j.name()).collect();
        assert!(names.contains(&"enhanced_catalog"));
        assert!(names.contains(&"catalog_hierarchy"));
        assert!(names.contains(&"schema_lifecycle"));
        assert!(names.contains(&"table_managed_lifecycle"));
        assert!(names.contains(&"volume_managed_lifecycle"));
        assert!(names.contains(&"share_lifecycle"));
        assert!(names.contains(&"recipient_lifecycle"));
        assert!(names.contains(&"provider_lifecycle"));
        assert!(names.contains(&"function_lifecycle"));
        assert!(names.contains(&"lakehouse_hierarchy"));
    }

    #[test]
    fn test_journey_descriptions() {
        for journey in all_journeys() {
            assert!(!journey.name().is_empty(), "Journey name must not be empty");
            assert!(
                !journey.description().is_empty(),
                "Journey description must not be empty"
            );
        }
    }

    #[test]
    fn test_journeys_have_metadata() {
        for journey in all_journeys() {
            let meta = journey.metadata();
            assert!(
                !meta.implementations.is_empty(),
                "Journey '{}' must declare at least one implementation tag",
                journey.name()
            );
        }
    }
}
