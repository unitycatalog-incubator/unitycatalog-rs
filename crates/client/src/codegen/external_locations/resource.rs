// @generated — do not edit by hand.
use super::builders::*;
use super::client::ExternalLocationServiceClient;
/// A client scoped to a single `external_location`.
#[derive(Clone)]
pub struct ExternalLocationClient {
    pub(crate) external_location_name: String,
    pub(crate) client: ExternalLocationServiceClient,
}
impl ExternalLocationClient {
    /// Create a client bound to the resource's name components.
    pub fn new(
        external_location_name: impl Into<String>,
        client: ExternalLocationServiceClient,
    ) -> Self {
        Self {
            external_location_name: external_location_name.into(),
            client,
        }
    }
    /// Get an external location
    pub fn get(&self) -> GetExternalLocationBuilder {
        GetExternalLocationBuilder::new(self.client.clone(), &self.external_location_name)
    }
    /// Update an external location
    pub fn update(&self) -> UpdateExternalLocationBuilder {
        UpdateExternalLocationBuilder::new(self.client.clone(), &self.external_location_name)
    }
    /// Delete an external location
    pub fn delete(&self) -> DeleteExternalLocationBuilder {
        DeleteExternalLocationBuilder::new(self.client.clone(), &self.external_location_name)
    }
}
