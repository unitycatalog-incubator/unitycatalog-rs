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
    pub(crate) fn new(
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
    ///Description about the recipient.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    /**Recipient properties as map of string key-value pairs.

    When provided in update request, the specified properties will override the existing properties.
    To add and remove properties, one would need to perform a read-modify-write.*/
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
    ///Expiration timestamp of the token, in epoch milliseconds.
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
    pub(crate) fn new(client: RecipientClient, name: impl Into<String>) -> Self {
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
    pub(crate) fn new(client: RecipientClient, name: impl Into<String>) -> Self {
        let request = UpdateRecipientRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    ///New name for the recipient
    pub fn with_new_name(mut self, new_name: impl Into<Option<String>>) -> Self {
        self.request.new_name = new_name.into();
        self
    }
    ///Username of the recipient owner.
    pub fn with_owner(mut self, owner: impl Into<Option<String>>) -> Self {
        self.request.owner = owner.into();
        self
    }
    ///Description about the recipient.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    /**Recipient properties as map of string key-value pairs.

    When provided in update request, the specified properties will override the existing properties.
    To add and remove properties, one would need to perform a read-modify-write.*/
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
    ///Expiration timestamp of the token, in epoch milliseconds.
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
