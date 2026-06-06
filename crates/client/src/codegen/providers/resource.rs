// @generated — do not edit by hand.
use super::builders::*;
use super::client::ProviderServiceClient;
/// A client scoped to a single `provider`.
#[derive(Clone)]
pub struct ProviderClient {
    pub(crate) provider_name: String,
    pub(crate) client: ProviderServiceClient,
}
impl ProviderClient {
    /// Create a client bound to the resource's name components.
    pub fn new(provider_name: impl Into<String>, client: ProviderServiceClient) -> Self {
        Self {
            provider_name: provider_name.into(),
            client,
        }
    }
    /// Get a provider by name.
    pub fn get(&self) -> GetProviderBuilder {
        GetProviderBuilder::new(self.client.clone(), &self.provider_name)
    }
    /// Update a provider.
    pub fn update(&self) -> UpdateProviderBuilder {
        UpdateProviderBuilder::new(self.client.clone(), &self.provider_name)
    }
    /// Delete a provider.
    pub fn delete(&self) -> DeleteProviderBuilder {
        DeleteProviderBuilder::new(self.client.clone(), &self.provider_name)
    }
}
