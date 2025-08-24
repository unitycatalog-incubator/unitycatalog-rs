use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::credentials::v1::*;

use super::utils::stream_paginated;
use crate::Result;
pub(super) use crate::codegen::credentials::CredentialClient as CredentialClientBase;
use crate::codegen::credentials::{
    CreateCredentialBuilder, DeleteCredentialBuilder, GetCredentialBuilder, UpdateCredentialBuilder,
};

impl CredentialClientBase {
    pub fn list(
        &self,
        purpose: Option<Purpose>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<CredentialInfo>> {
        let max_results = max_results.into();
        let purpose = purpose.map(|p| p as i32);
        stream_paginated(max_results, move |mut max_results, page_token| async move {
            let request = ListCredentialsRequest {
                max_results,
                page_token,
                purpose,
            };
            let res = self.list_credentials(&request).await?;

            // Update max_results for next page based on items received
            if let Some(ref mut remaining) = max_results {
                *remaining -= res.credentials.len() as i32;
                if *remaining <= 0 {
                    max_results = Some(0);
                }
            }

            Ok((res.credentials, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}

#[derive(Clone)]
pub struct CredentialClient {
    name: String,
    client: CredentialClientBase,
}

impl CredentialClient {
    pub fn new(name: impl ToString, client: CredentialClientBase) -> Self {
        Self {
            name: name.to_string(),
            client,
        }
    }

    /// Create a new credential using the builder pattern.
    pub fn create(&self, purpose: Purpose) -> CreateCredentialBuilder {
        CreateCredentialBuilder::new(self.client.clone(), &self.name, purpose)
    }

    /// Get a credential using the builder pattern.
    pub fn get(&self) -> GetCredentialBuilder {
        GetCredentialBuilder::new(self.client.clone(), &self.name)
    }

    /// Update this credential using the builder pattern.
    pub fn update(&self) -> UpdateCredentialBuilder {
        UpdateCredentialBuilder::new(self.client.clone(), &self.name)
    }

    /// Delete this credential using the builder pattern.
    pub fn delete(&self) -> DeleteCredentialBuilder {
        DeleteCredentialBuilder::new(self.client.clone(), &self.name)
    }
}
