use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::schemas::v1::*;

use super::utils::stream_paginated;
use crate::Result;
pub use crate::codegen::schemas::SchemaClient;
pub(super) use crate::codegen::schemas::SchemaServiceClient;

impl SchemaServiceClient {
    pub fn list(
        &self,
        catalog_name: impl Into<String>,
        max_results: impl Into<Option<i32>>,
        include_browse: impl Into<Option<bool>>,
    ) -> BoxStream<'_, Result<Schema>> {
        let max_results = max_results.into();
        let catalog_name = catalog_name.into();
        let include_browse = include_browse.into();
        stream_paginated(
            (catalog_name, max_results, include_browse),
            move |(catalog_name, mut max_results, include_browse), page_token| async move {
                let request = ListSchemasRequest {
                    catalog_name: catalog_name.clone(),
                    max_results,
                    page_token,
                    include_browse: None,
                };
                let res = self.list_schemas(&request).await?;

                // Update max_results for next page based on items received
                if let Some(ref mut remaining) = max_results {
                    *remaining -= res.schemas.len() as i32;
                    if *remaining <= 0 {
                        max_results = Some(0);
                    }
                }

                Ok((
                    res.schemas,
                    (catalog_name, max_results, include_browse),
                    res.next_page_token,
                ))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}
