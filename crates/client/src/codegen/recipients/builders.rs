#![allow(unused_mut)]
use super::client::*;
use crate::{error::Result, utils::stream_paginated};
use futures::{Stream, future::BoxFuture};
use std::future::IntoFuture;
use unitycatalog_common::models::recipients::v1::*;
/// Builder for creating requests
pub struct ListRecipientsBuilder {
    client: RecipientClient,
    request: ListRecipientsRequest,
}
impl ListRecipientsBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: RecipientClient) -> Self {
        let request = ListRecipientsRequest {
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
    pub(crate) fn into_stream(&self) -> impl Stream<Item = Result<ListRecipientsResponse>> {
        let request = self.request.clone();
        stream_paginated(request, move |mut request, page_token| async move {
            request.page_token = page_token;
            let res = self.client.list_recipients(&request).await?;
            if let Some(ref mut remaining) = request.max_results {
                *remaining -= res.recipients.len() as i32;
                if *remaining <= 0 {
                    request.max_results = Some(0);
                }
            }
            let next_page_token = res.next_page_token.clone();
            Ok((res, request, next_page_token))
        })
    }
}
impl IntoFuture for ListRecipientsBuilder {
    type Output = Result<ListRecipientsResponse>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_recipients(&request).await })
    }
}
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
/// Builder for creating requests
pub struct DeleteRecipientBuilder {
    client: RecipientClient,
    request: DeleteRecipientRequest,
}
impl DeleteRecipientBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: RecipientClient, name: impl Into<String>) -> Self {
        let request = DeleteRecipientRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for DeleteRecipientBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_recipient(&request).await })
    }
}
