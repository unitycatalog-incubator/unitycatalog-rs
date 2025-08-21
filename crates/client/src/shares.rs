use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::shares::v1::*;

use super::utils::stream_paginated;
use crate::Result;
pub(super) use crate::codegen::shares::ShareClient as ShareClientBase;
use crate::codegen::shares::builders::{CreateShareBuilder, GetShareBuilder, UpdateShareBuilder};

impl ShareClientBase {
    pub fn list(&self, max_results: impl Into<Option<i32>>) -> BoxStream<'_, Result<ShareInfo>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |mut max_results, page_token| async move {
            let request = ListSharesRequest {
                max_results,
                page_token,
            };
            let res = self.list_shares(&request).await?;

            // Update max_results for next page based on items received
            if let Some(ref mut remaining) = max_results {
                *remaining -= res.shares.len() as i32;
                if *remaining <= 0 {
                    max_results = Some(0);
                }
            }

            Ok((res.shares, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}

#[derive(Clone)]
pub struct ShareClient {
    name: String,
    client: ShareClientBase,
}

impl ShareClient {
    pub fn new(name: impl ToString, client: ShareClientBase) -> Self {
        Self {
            name: name.to_string(),
            client,
        }
    }

    /// Create a new share using the builder pattern.
    pub fn create(&self) -> CreateShareBuilder {
        CreateShareBuilder::new(self.client.clone(), &self.name)
    }

    /// Get a share using the builder pattern.
    pub fn get(&self) -> GetShareBuilder {
        GetShareBuilder::new(self.client.clone(), &self.name)
    }

    pub async fn delete(&self) -> Result<()> {
        let request = DeleteShareRequest {
            name: self.name.clone(),
        };
        self.client.delete_share(&request).await
    }

    /// Update this share using the builder pattern.
    pub fn update(&self) -> UpdateShareBuilder {
        UpdateShareBuilder::new(self.client.clone(), &self.name)
    }
}
