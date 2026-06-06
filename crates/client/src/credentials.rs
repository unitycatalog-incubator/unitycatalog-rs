use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::credentials::v1::*;

use super::utils::stream_paginated;
use crate::Result;
pub use crate::codegen::credentials::CredentialClient;
pub(super) use crate::codegen::credentials::CredentialServiceClient;

impl CredentialServiceClient {
    pub fn list(
        &self,
        purpose: Option<Purpose>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<Credential>> {
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
