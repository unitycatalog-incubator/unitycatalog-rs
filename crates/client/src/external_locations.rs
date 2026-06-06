use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::external_locations::v1::*;

use super::utils::stream_paginated;
use crate::Result;
pub use crate::codegen::external_locations::ExternalLocationClient;
pub(super) use crate::codegen::external_locations::ExternalLocationServiceClient;

impl ExternalLocationServiceClient {
    pub fn list(
        &self,
        max_results: impl Into<Option<i32>>,
        include_browse: impl Into<Option<bool>>,
    ) -> BoxStream<'_, Result<ExternalLocation>> {
        let max_results = max_results.into();
        let include_browse = include_browse.into();
        stream_paginated(max_results, move |mut max_results, page_token| async move {
            let request = ListExternalLocationsRequest {
                max_results,
                page_token,
                include_browse,
            };
            let res = self.list_external_locations(&request).await?;

            // Update max_results for next page based on items received
            if let Some(ref mut remaining) = max_results {
                *remaining -= res.external_locations.len() as i32;
                if *remaining <= 0 {
                    max_results = Some(0);
                }
            }

            Ok((res.external_locations, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}
