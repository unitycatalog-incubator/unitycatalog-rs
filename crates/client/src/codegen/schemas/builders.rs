#![allow(unused_mut)]
use super::client::*;
use crate::error::Result;
use futures::future::BoxFuture;
use std::future::IntoFuture;
use unitycatalog_common::models::schemas::v1::*;
/// Builder for creating requests
pub struct CreateSchemaBuilder {
    client: SchemaClient,
    request: CreateSchemaRequest,
}
impl CreateSchemaBuilder {
    /// Create a new builder instance
    pub(crate) fn new(
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
    ///User-provided free-form text description.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    ///A map of key-value properties attached to the securable.
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
pub struct GetSchemaBuilder {
    client: SchemaClient,
    request: GetSchemaRequest,
}
impl GetSchemaBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: SchemaClient, full_name: impl Into<String>) -> Self {
        let request = GetSchemaRequest {
            full_name: full_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for GetSchemaBuilder {
    type Output = Result<SchemaInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_schema(&request).await })
    }
}
/// Builder for creating requests
pub struct UpdateSchemaBuilder {
    client: SchemaClient,
    request: UpdateSchemaRequest,
}
impl UpdateSchemaBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: SchemaClient, full_name: impl Into<String>) -> Self {
        let request = UpdateSchemaRequest {
            full_name: full_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    ///User-provided free-form text description.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    /**A map of key-value properties attached to the securable.

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
    ///Name of schema.
    pub fn with_new_name(mut self, new_name: impl Into<Option<String>>) -> Self {
        self.request.new_name = new_name.into();
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
