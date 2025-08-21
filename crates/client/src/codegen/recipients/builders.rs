#![allow(unused_mut)]
use super::client::*;
use crate::error::Result;
use futures::future::BoxFuture;
use std::future::IntoFuture;
use unitycatalog_common::models::recipients::v1::*;
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
        authentication_type: AuthenticationType,
        owner: impl Into<String>,
    ) -> Self {
        let request = CreateRecipientRequest {
            name: name.into(),
            authentication_type: authentication_type as i32,
            owner: owner.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    #[doc = concat!("Set ", "comment")]
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    #[doc = concat!("Set ", "properties")]
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
    pub fn with_expiration_time(mut self, expiration_time: impl Into<Option<i64>>) -> Self {
        self.request.expiration_time = expiration_time.into();
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
pub struct GetRecipientBuilder {
    client: RecipientClient,
    request: GetRecipientRequest,
}
impl GetRecipientBuilder {
    /// Create a new builder instance
    pub fn new(client: RecipientClient, name: impl Into<String>) -> Self {
        let request = GetRecipientRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for GetRecipientBuilder {
    type Output = Result<RecipientInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_recipient(&request).await })
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
    pub fn with_new_name(mut self, new_name: impl Into<Option<String>>) -> Self {
        self.request.new_name = new_name.into();
        self
    }
    #[doc = concat!("Set ", "owner")]
    pub fn with_owner(mut self, owner: impl Into<Option<String>>) -> Self {
        self.request.owner = owner.into();
        self
    }
    #[doc = concat!("Set ", "comment")]
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    #[doc = concat!("Set ", "properties")]
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
    pub fn with_expiration_time(mut self, expiration_time: impl Into<Option<i64>>) -> Self {
        self.request.expiration_time = expiration_time.into();
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
