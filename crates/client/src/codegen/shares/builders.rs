#![allow(unused_mut)]
use super::client::*;
use crate::error::Result;
use futures::future::BoxFuture;
use std::future::IntoFuture;
use unitycatalog_common::models::shares::v1::*;
/// Builder for creating requests
pub struct CreateShareBuilder {
    client: ShareClient,
    request: CreateShareRequest,
}
impl CreateShareBuilder {
    /// Create a new builder instance
    pub fn new(client: ShareClient, name: impl Into<String>) -> Self {
        let request = CreateShareRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    #[doc = concat!("Set ", "comment")]
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.request.comment = Some(comment.into());
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
pub struct UpdateShareBuilder {
    client: ShareClient,
    request: UpdateShareRequest,
}
impl UpdateShareBuilder {
    /// Create a new builder instance
    pub fn new(client: ShareClient, name: impl Into<String>) -> Self {
        let request = UpdateShareRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    #[doc = concat!("Set ", "updates")]
    pub fn with_updates<I>(mut self, updates: I) -> Self
    where
        I: IntoIterator<Item = DataObjectUpdate>,
    {
        self.request.updates = updates.into_iter().collect();
        self
    }
    #[doc = concat!("Set ", "new_name")]
    pub fn with_new_name(mut self, new_name: impl Into<String>) -> Self {
        self.request.new_name = Some(new_name.into());
        self
    }
    #[doc = concat!("Set ", "owner")]
    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.request.owner = Some(owner.into());
        self
    }
    #[doc = concat!("Set ", "comment")]
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.request.comment = Some(comment.into());
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
