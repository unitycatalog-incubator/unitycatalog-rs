use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::recipients::v1::*;

use super::utils::stream_paginated;
use crate::Result;
pub(super) use crate::codegen::recipients::RecipientClient as RecipientClientBase;
use crate::codegen::recipients::builders::{
    CreateRecipientBuilder, GetRecipientBuilder, UpdateRecipientBuilder,
};

impl RecipientClientBase {
    pub fn list(
        &self,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<RecipientInfo>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |mut max_results, page_token| async move {
            let request = ListRecipientsRequest {
                max_results,
                page_token,
            };
            let res = self.list_recipients(&request).await?;

            // Update max_results for next page based on items received
            if let Some(ref mut remaining) = max_results {
                *remaining -= res.recipients.len() as i32;
                if *remaining <= 0 {
                    max_results = Some(0);
                }
            }

            Ok((res.recipients, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}

#[derive(Clone)]
pub struct RecipientClient {
    name: String,
    client: RecipientClientBase,
}

impl RecipientClient {
    pub fn new(name: impl ToString, client: RecipientClientBase) -> Self {
        Self {
            name: name.to_string(),
            client,
        }
    }

    /// Create a new recipient using the builder pattern.
    pub fn create(
        &self,
        authentication_type: AuthenticationType,
        owner: impl Into<String>,
    ) -> CreateRecipientBuilder {
        CreateRecipientBuilder::new(
            self.client.clone(),
            &self.name,
            authentication_type,
            owner.into(),
        )
    }

    /// Get a recipient using the builder pattern.
    pub fn get(&self) -> GetRecipientBuilder {
        GetRecipientBuilder::new(self.client.clone(), &self.name)
    }

    /// Update this recipient using the builder pattern.
    pub fn update(&self) -> UpdateRecipientBuilder {
        UpdateRecipientBuilder::new(self.client.clone(), &self.name)
    }

    pub async fn delete(&self) -> Result<()> {
        let request = DeleteRecipientRequest {
            name: self.name.clone(),
        };
        self.client.delete_recipient(&request).await
    }
}
