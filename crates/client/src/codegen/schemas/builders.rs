#![allow(unused_mut)]
use futures::future::BoxFuture;
use std::future::IntoFuture;
use crate::error::Result;
use unitycatalog_common::models::schemas::v1::*;
use super::client::*;
/// Builder for creating requests
pub struct CreateSchemaBuilder {
    client: SchemaClient,
    request: CreateSchemaRequest,
}
impl CreateSchemaBuilder {
    /// Create a new builder instance
    pub fn new(
        client: SchemaClient,
        name: impl Into<String>,
        catalog_name: impl Into<String>,
    ) -> Self {
        let request = CreateSchemaRequest {
            name: name.into(),
            catalog_name: catalog_name.into(),
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
}
impl IntoFuture for CreateSchemaBuilder {
    type Output = Result<SchemaInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_schema(&request).await })
    }
}
/// Builder for creating requests
pub struct UpdateSchemaBuilder {
    client: SchemaClient,
    request: UpdateSchemaRequest,
}
impl UpdateSchemaBuilder {
    /// Create a new builder instance
    pub fn new(
        client: SchemaClient,
        full_name: impl Into<String>,
        new_name: impl Into<String>,
    ) -> Self {
        let request = UpdateSchemaRequest {
            full_name: full_name.into(),
            new_name: new_name.into(),
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
}
impl IntoFuture for UpdateSchemaBuilder {
    type Output = Result<SchemaInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_schema(&request).await })
    }
}
