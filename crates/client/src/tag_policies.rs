use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::tags::v1::*;

use super::utils::stream_paginated;
use crate::Result;
pub(super) use crate::codegen::tag_policies::TagPolicyClient as TagPolicyClientBase;
use crate::codegen::tag_policies::{
    DeleteTagPolicyBuilder, GetTagPolicyBuilder, UpdateTagPolicyBuilder,
};

impl TagPolicyClientBase {
    pub fn list(&self, max_results: impl Into<Option<i32>>) -> BoxStream<'_, Result<TagPolicy>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |mut max_results, page_token| async move {
            let request = ListTagPoliciesRequest {
                max_results,
                page_token,
            };
            let res = self.list_tag_policies(&request).await?;

            // Update max_results for next page based on items received
            if let Some(ref mut remaining) = max_results {
                *remaining -= res.tag_policies.len() as i32;
                if *remaining <= 0 {
                    max_results = Some(0);
                }
            }

            Ok((res.tag_policies, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}

/// Client scoped to a single governed tag definition (tag policy), keyed by tag key.
#[derive(Clone)]
pub struct TagPolicyClient {
    tag_key: String,
    client: TagPolicyClientBase,
}

impl TagPolicyClient {
    pub fn new(tag_key: impl ToString, client: TagPolicyClientBase) -> Self {
        Self {
            tag_key: tag_key.to_string(),
            client,
        }
    }

    /// Get this tag policy using the builder pattern.
    pub fn get(&self) -> GetTagPolicyBuilder {
        GetTagPolicyBuilder::new(self.client.clone(), &self.tag_key)
    }

    /// Update this tag policy using the builder pattern.
    pub fn update(&self) -> UpdateTagPolicyBuilder {
        UpdateTagPolicyBuilder::new(self.client.clone(), &self.tag_key)
    }

    /// Delete this tag policy.
    pub fn delete(&self) -> DeleteTagPolicyBuilder {
        DeleteTagPolicyBuilder::new(self.client.clone(), &self.tag_key)
    }
}
