use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::providers::v1::*;

use super::utils::stream_paginated;
use crate::Result;
use crate::codegen::providers::DeleteProviderBuilder;
pub(super) use crate::codegen::providers::ProviderClient as ProviderClientBase;
use crate::codegen::providers::builders::{
    CreateProviderBuilder, GetProviderBuilder, UpdateProviderBuilder,
};

impl ProviderClientBase {
    pub fn list(&self, max_results: impl Into<Option<i32>>) -> BoxStream<'_, Result<Provider>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |mut max_results, page_token| async move {
            let request = ListProvidersRequest {
                max_results,
                page_token,
            };
            let res = self.list_providers(&request).await?;

            // Update max_results for next page based on items received
            if let Some(ref mut remaining) = max_results {
                *remaining -= res.providers.len() as i32;
                if *remaining <= 0 {
                    max_results = Some(0);
                }
            }

            Ok((res.providers, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}

#[derive(Clone)]
pub struct ProviderClient {
    name: String,
    client: ProviderClientBase,
}

impl ProviderClient {
    pub fn new(name: impl ToString, client: ProviderClientBase) -> Self {
        Self {
            name: name.to_string(),
            client,
        }
    }

    /// Create a new provider using the builder pattern.
    pub fn create(&self, authentication_type: ProviderAuthenticationType) -> CreateProviderBuilder {
        CreateProviderBuilder::new(self.client.clone(), &self.name, authentication_type)
    }

    /// Get a provider using the builder pattern.
    pub fn get(&self) -> GetProviderBuilder {
        GetProviderBuilder::new(self.client.clone(), &self.name)
    }

    /// Update this provider using the builder pattern.
    pub fn update(&self) -> UpdateProviderBuilder {
        UpdateProviderBuilder::new(self.client.clone(), &self.name)
    }

    pub fn delete(&self) -> DeleteProviderBuilder {
        DeleteProviderBuilder::new(self.client.clone(), &self.name)
    }
}
