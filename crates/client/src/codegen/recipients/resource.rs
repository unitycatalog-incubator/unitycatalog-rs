// @generated — do not edit by hand.
use super::builders::*;
use super::client::RecipientServiceClient;
/// A client scoped to a single `recipient`.
#[derive(Clone)]
pub struct RecipientClient {
    pub(crate) recipient_name: String,
    pub(crate) client: RecipientServiceClient,
}
impl RecipientClient {
    /// Create a client bound to the resource's name components.
    pub fn new(recipient_name: impl Into<String>, client: RecipientServiceClient) -> Self {
        Self {
            recipient_name: recipient_name.into(),
            client,
        }
    }
    /// Get a recipient by name.
    pub fn get(&self) -> GetRecipientBuilder {
        GetRecipientBuilder::new(self.client.clone(), &self.recipient_name)
    }
    /// Update a recipient.
    pub fn update(&self) -> UpdateRecipientBuilder {
        UpdateRecipientBuilder::new(self.client.clone(), &self.recipient_name)
    }
    /// Delete a recipient.
    pub fn delete(&self) -> DeleteRecipientBuilder {
        DeleteRecipientBuilder::new(self.client.clone(), &self.recipient_name)
    }
}
