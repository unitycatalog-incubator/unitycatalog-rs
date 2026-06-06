use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::volumes::v1::*;

use super::utils::stream_paginated;
use crate::Result;
pub use crate::codegen::volumes::VolumeClient;
pub(super) use crate::codegen::volumes::client::VolumeServiceClient;

impl VolumeServiceClient {
    pub fn list(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        max_results: impl Into<Option<i32>>,
        include_browse: impl Into<Option<bool>>,
    ) -> BoxStream<'_, Result<Volume>> {
        let max_results = max_results.into();
        let catalog_name = catalog_name.into();
        let schema_name = schema_name.into();
        let include_browse = include_browse.into();
        stream_paginated(
            (catalog_name, schema_name, max_results, include_browse),
            move |(catalog_name, schema_name, max_results, include_browse), page_token| async move {
                let request = ListVolumesRequest {
                    catalog_name: catalog_name.clone(),
                    schema_name: schema_name.clone(),
                    max_results,
                    page_token,
                    include_browse,
                };
                let res = self.list_volumes(&request).await?;
                Ok((
                    res.volumes,
                    (catalog_name, schema_name, max_results, include_browse),
                    res.next_page_token,
                ))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}
