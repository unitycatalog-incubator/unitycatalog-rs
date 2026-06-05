// @generated — do not edit by hand.
#![allow(unused_mut)]
use super::super::stream_paginated;
use super::client::*;
use crate::Result;
use futures::{StreamExt, TryStreamExt, future::BoxFuture, stream::BoxStream};
use std::future::IntoFuture;
use unitycatalog_common::models::tags::v1::*;
/// Builder for listing tag policies
pub struct ListTagPoliciesBuilder {
    client: TagPolicyClient,
    request: ListTagPoliciesRequest,
}
impl ListTagPoliciesBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TagPolicyClient`.
    pub(crate) fn new(client: TagPolicyClient) -> Self {
        let request = ListTagPoliciesRequest {
            ..Default::default()
        };
        Self { client, request }
    }
    /// The maximum number of results per page that should be returned.
    pub fn with_max_results(mut self, max_results: impl Into<Option<i32>>) -> Self {
        self.request.max_results = max_results.into();
        self
    }
    /// Opaque pagination token to go to next page based on previous query.
    pub fn with_page_token(mut self, page_token: impl Into<Option<String>>) -> Self {
        self.request.page_token = page_token.into();
        self
    }
    /// Convert paginated request into stream of results
    pub fn into_stream(self) -> BoxStream<'static, Result<TagPolicy>> {
        let remaining = self.request.max_results;
        stream_paginated(
            (self, remaining),
            move |(mut builder, mut remaining), page_token| async move {
                builder.request.page_token = page_token;
                let res = builder.client.list_tag_policies(&builder.request).await?;
                if let Some(ref mut rem) = remaining {
                    *rem -= res.tag_policies.len() as i32;
                }
                let next_page_token = if remaining.is_some_and(|r| r <= 0) {
                    None
                } else {
                    res.next_page_token.clone()
                };
                Ok((res, (builder, remaining), next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.tag_policies.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}
impl IntoFuture for ListTagPoliciesBuilder {
    type Output = Result<ListTagPoliciesResponse>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_tag_policies(&request).await })
    }
}
/// Builder for creating a tag policy
pub struct CreateTagPolicyBuilder {
    client: TagPolicyClient,
    request: CreateTagPolicyRequest,
}
impl CreateTagPolicyBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TagPolicyClient`.
    pub(crate) fn new(client: TagPolicyClient) -> Self {
        let request = CreateTagPolicyRequest {
            ..Default::default()
        };
        Self { client, request }
    }
    /// The tag policy to create.
    pub fn with_tag_policy(mut self, tag_policy: impl Into<Option<TagPolicy>>) -> Self {
        self.request.tag_policy = tag_policy.into();
        self
    }
}
impl IntoFuture for CreateTagPolicyBuilder {
    type Output = Result<TagPolicy>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_tag_policy(&request).await })
    }
}
/// Builder for getting a tag policy
pub struct GetTagPolicyBuilder {
    client: TagPolicyClient,
    request: GetTagPolicyRequest,
}
impl GetTagPolicyBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TagPolicyClient`.
    pub(crate) fn new(client: TagPolicyClient, tag_key: impl Into<String>) -> Self {
        let request = GetTagPolicyRequest {
            tag_key: tag_key.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for GetTagPolicyBuilder {
    type Output = Result<TagPolicy>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_tag_policy(&request).await })
    }
}
/// Builder for updating a tag policy
pub struct UpdateTagPolicyBuilder {
    client: TagPolicyClient,
    request: UpdateTagPolicyRequest,
}
impl UpdateTagPolicyBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TagPolicyClient`.
    pub(crate) fn new(client: TagPolicyClient, tag_key: impl Into<String>) -> Self {
        let request = UpdateTagPolicyRequest {
            tag_key: tag_key.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// The tag policy with the updated fields.
    pub fn with_tag_policy(mut self, tag_policy: impl Into<Option<TagPolicy>>) -> Self {
        self.request.tag_policy = tag_policy.into();
        self
    }
    /// The list of fields to update, as a comma-separated string.
    pub fn with_update_mask(mut self, update_mask: impl Into<Option<String>>) -> Self {
        self.request.update_mask = update_mask.into();
        self
    }
}
impl IntoFuture for UpdateTagPolicyBuilder {
    type Output = Result<TagPolicy>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_tag_policy(&request).await })
    }
}
/// Builder for deleting a tag policy
pub struct DeleteTagPolicyBuilder {
    client: TagPolicyClient,
    request: DeleteTagPolicyRequest,
}
impl DeleteTagPolicyBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TagPolicyClient`.
    pub(crate) fn new(client: TagPolicyClient, tag_key: impl Into<String>) -> Self {
        let request = DeleteTagPolicyRequest {
            tag_key: tag_key.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for DeleteTagPolicyBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_tag_policy(&request).await })
    }
}
