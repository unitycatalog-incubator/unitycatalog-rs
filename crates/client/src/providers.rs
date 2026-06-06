use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::providers::v1::*;

use super::utils::stream_paginated;
use crate::Result;
pub use crate::codegen::providers::ProviderClient;
pub(super) use crate::codegen::providers::ProviderServiceClient;

impl ProviderServiceClient {
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
