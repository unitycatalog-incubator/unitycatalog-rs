#![allow(unused_mut)]
use futures::future::BoxFuture;
use std::future::IntoFuture;
use crate::error::Result;
use unitycatalog_common::models::recipients::v1::*;
use super::client::*;
/// Builder for creating requests
pub struct CreateRecipientBuilder {
    client: RecipientClient,
    request: CreateRecipientRequest,
}
impl CreateRecipientBuilder {
    /// Create a new builder instance
    pub fn new(
        client: RecipientClient,
        name: impl Into<String>,
        authentication_type: i32,
        owner: impl Into<String>,
    ) -> Self {
        let request = CreateRecipientRequest {
            name: name.into(),
            authentication_type,
            owner: owner.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    #[doc = concat!("Set ", "comment")]
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.request.comment = Some(comment.into());
        self
    }
    #[doc = concat!("Set ", "properties", " property")]
    pub fn with_properties<I, K, V>(mut self, properties: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.request.properties = properties
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self
    }
    #[doc = concat!("Set ", "expiration_time")]
    pub fn with_expiration_time(mut self, expiration_time: i64) -> Self {
        self.request.expiration_time = Some(expiration_time);
        self
    }
}
impl IntoFuture for CreateRecipientBuilder {
    type Output = Result<RecipientInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_recipient(&request).await })
    }
}
/// Builder for creating requests
pub struct UpdateRecipientBuilder {
    client: RecipientClient,
    request: UpdateRecipientRequest,
}
impl UpdateRecipientBuilder {
    /// Create a new builder instance
    pub fn new(client: RecipientClient, name: impl Into<String>) -> Self {
        let request = UpdateRecipientRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
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
    #[doc = concat!("Set ", "properties", " property")]
    pub fn with_properties<I, K, V>(mut self, properties: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.request.properties = properties
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self
    }
    #[doc = concat!("Set ", "expiration_time")]
    pub fn with_expiration_time(mut self, expiration_time: i64) -> Self {
        self.request.expiration_time = Some(expiration_time);
        self
    }
}
impl IntoFuture for UpdateRecipientBuilder {
    type Output = Result<RecipientInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_recipient(&request).await })
    }
}
