#![allow(unused_mut)]
use super::client::*;
use crate::{error::Result, utils::stream_paginated};
use futures::{Stream, future::BoxFuture};
use std::future::IntoFuture;
use unitycatalog_common::models::shares::v1::*;
/// Builder for creating requests
pub struct ListSharesBuilder {
    client: ShareClient,
    request: ListSharesRequest,
}
impl ListSharesBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: ShareClient) -> Self {
        let request = ListSharesRequest {
            ..Default::default()
        };
        Self { client, request }
    }
    ///The maximum number of results per page that should be returned.
    pub fn with_max_results(mut self, max_results: impl Into<Option<i32>>) -> Self {
        self.request.max_results = max_results.into();
        self
    }
    ///Opaque pagination token to go to next page based on previous query.
    pub fn with_page_token(mut self, page_token: impl Into<Option<String>>) -> Self {
        self.request.page_token = page_token.into();
        self
    }
    /// Convert paginated request into stream of results
    pub(crate) fn into_stream(&self) -> impl Stream<Item = Result<ListSharesResponse>> {
        let request = self.request.clone();
        stream_paginated(request, move |mut request, page_token| async move {
            request.page_token = page_token;
            let res = self.client.list_shares(&request).await?;
            if let Some(ref mut remaining) = request.max_results {
                *remaining -= res.shares.len() as i32;
                if *remaining <= 0 {
                    request.max_results = Some(0);
                }
            }
            let next_page_token = res.next_page_token.clone();
            Ok((res, request, next_page_token))
        })
    }
}
impl IntoFuture for ListSharesBuilder {
    type Output = Result<ListSharesResponse>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_shares(&request).await })
    }
}
/// Builder for creating requests
pub struct CreateShareBuilder {
    client: ShareClient,
    request: CreateShareRequest,
}
impl CreateShareBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: ShareClient, name: impl Into<String>) -> Self {
        let request = CreateShareRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    ///User-provided free-form text description.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
}
impl IntoFuture for CreateShareBuilder {
    type Output = Result<ShareInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_share(&request).await })
    }
}
/// Builder for creating requests
pub struct GetShareBuilder {
    client: ShareClient,
    request: GetShareRequest,
}
impl GetShareBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: ShareClient, name: impl Into<String>) -> Self {
        let request = GetShareRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    ///Query for data to include in the share.
    pub fn with_include_shared_data(
        mut self,
        include_shared_data: impl Into<Option<bool>>,
    ) -> Self {
        self.request.include_shared_data = include_shared_data.into();
        self
    }
}
impl IntoFuture for GetShareBuilder {
    type Output = Result<ShareInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_share(&request).await })
    }
}
/// Builder for creating requests
pub struct UpdateShareBuilder {
    client: ShareClient,
    request: UpdateShareRequest,
}
impl UpdateShareBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: ShareClient, name: impl Into<String>) -> Self {
        let request = UpdateShareRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    ///Array of shared data object updates.
    pub fn with_updates<I>(mut self, updates: I) -> Self
    where
        I: IntoIterator<Item = DataObjectUpdate>,
    {
        self.request.updates = updates.into_iter().collect();
        self
    }
    ///A new name for the share.
    pub fn with_new_name(mut self, new_name: impl Into<Option<String>>) -> Self {
        self.request.new_name = new_name.into();
        self
    }
    ///Owner of the share.
    pub fn with_owner(mut self, owner: impl Into<Option<String>>) -> Self {
        self.request.owner = owner.into();
        self
    }
    ///User-provided free-form text description.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
}
impl IntoFuture for UpdateShareBuilder {
    type Output = Result<ShareInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_share(&request).await })
    }
}
/// Builder for creating requests
pub struct DeleteShareBuilder {
    client: ShareClient,
    request: DeleteShareRequest,
}
impl DeleteShareBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: ShareClient, name: impl Into<String>) -> Self {
        let request = DeleteShareRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for DeleteShareBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_share(&request).await })
    }
}
