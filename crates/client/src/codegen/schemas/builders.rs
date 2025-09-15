#![allow(unused_mut)]
use super::client::*;
use crate::{error::Result, utils::stream_paginated};
use futures::{Stream, future::BoxFuture};
use std::future::IntoFuture;
use unitycatalog_common::models::schemas::v1::*;
/// Builder for creating requests
pub struct ListSchemasBuilder {
    client: SchemaClient,
    request: ListSchemasRequest,
}
impl ListSchemasBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: SchemaClient, catalog_name: impl Into<String>) -> Self {
        let request = ListSchemasRequest {
            catalog_name: catalog_name.into(),
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
    ///Whether to include schemas in the response for which the principal can only access selective metadata for
    pub fn with_include_browse(mut self, include_browse: impl Into<Option<bool>>) -> Self {
        self.request.include_browse = include_browse.into();
        self
    }
    /// Convert paginated request into stream of results
    pub(crate) fn into_stream(&self) -> impl Stream<Item = Result<ListSchemasResponse>> {
        let request = self.request.clone();
        stream_paginated(request, move |mut request, page_token| async move {
            request.page_token = page_token;
            let res = self.client.list_schemas(&request).await?;
            if let Some(ref mut remaining) = request.max_results {
                *remaining -= res.schemas.len() as i32;
                if *remaining <= 0 {
                    request.max_results = Some(0);
                }
            }
            let next_page_token = res.next_page_token.clone();
            Ok((res, request, next_page_token))
        })
    }
}
impl IntoFuture for ListSchemasBuilder {
    type Output = Result<ListSchemasResponse>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_schemas(&request).await })
    }
}
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
/// Builder for creating requests
pub struct DeleteSchemaBuilder {
    client: SchemaClient,
    request: DeleteSchemaRequest,
}
impl DeleteSchemaBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: SchemaClient, full_name: impl Into<String>) -> Self {
        let request = DeleteSchemaRequest {
            full_name: full_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    ///Force deletion even if the schema is not empty.
    pub fn with_force(mut self, force: impl Into<Option<bool>>) -> Self {
        self.request.force = force.into();
        self
    }
}
impl IntoFuture for DeleteSchemaBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_schema(&request).await })
    }
}
