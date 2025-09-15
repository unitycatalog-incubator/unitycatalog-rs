#![allow(unused_mut)]
use super::client::*;
use crate::{error::Result, utils::stream_paginated};
use futures::{Stream, future::BoxFuture};
use std::future::IntoFuture;
use unitycatalog_common::models::catalogs::v1::*;
/// Builder for creating requests
pub struct ListCatalogsBuilder {
    client: CatalogClient,
    request: ListCatalogsRequest,
}
impl ListCatalogsBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: CatalogClient) -> Self {
        let request = ListCatalogsRequest {
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
    pub(crate) fn into_stream(&self) -> impl Stream<Item = Result<ListCatalogsResponse>> {
        let request = self.request.clone();
        stream_paginated(request, move |mut request, page_token| async move {
            request.page_token = page_token;
            let res = self.client.list_catalogs(&request).await?;
            if let Some(ref mut remaining) = request.max_results {
                *remaining -= res.catalogs.len() as i32;
                if *remaining <= 0 {
                    request.max_results = Some(0);
                }
            }
            let next_page_token = res.next_page_token.clone();
            Ok((res, request, next_page_token))
        })
    }
}
impl IntoFuture for ListCatalogsBuilder {
    type Output = Result<ListCatalogsResponse>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_catalogs(&request).await })
    }
}
/// Builder for creating requests
pub struct CreateCatalogBuilder {
    client: CatalogClient,
    request: CreateCatalogRequest,
}
impl CreateCatalogBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: CatalogClient, name: impl Into<String>) -> Self {
        let request = CreateCatalogRequest {
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
    ///Storage root URL for managed tables within catalog.
    pub fn with_storage_root(mut self, storage_root: impl Into<Option<String>>) -> Self {
        self.request.storage_root = storage_root.into();
        self
    }
    /**The name of delta sharing provider.

    A Delta Sharing catalog is a catalog that is based on a Delta share on a remote sharing server.*/
    pub fn with_provider_name(mut self, provider_name: impl Into<Option<String>>) -> Self {
        self.request.provider_name = provider_name.into();
        self
    }
    ///The name of the share under the share provider.
    pub fn with_share_name(mut self, share_name: impl Into<Option<String>>) -> Self {
        self.request.share_name = share_name.into();
        self
    }
}
impl IntoFuture for CreateCatalogBuilder {
    type Output = Result<CatalogInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_catalog(&request).await })
    }
}
/// Builder for creating requests
pub struct GetCatalogBuilder {
    client: CatalogClient,
    request: GetCatalogRequest,
}
impl GetCatalogBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: CatalogClient, name: impl Into<String>) -> Self {
        let request = GetCatalogRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    ///Whether to include catalogs in the response for which the principal can only access selective metadata for
    pub fn with_include_browse(mut self, include_browse: impl Into<Option<bool>>) -> Self {
        self.request.include_browse = include_browse.into();
        self
    }
}
impl IntoFuture for GetCatalogBuilder {
    type Output = Result<CatalogInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_catalog(&request).await })
    }
}
/// Builder for creating requests
pub struct UpdateCatalogBuilder {
    client: CatalogClient,
    request: UpdateCatalogRequest,
}
impl UpdateCatalogBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: CatalogClient, name: impl Into<String>) -> Self {
        let request = UpdateCatalogRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    ///Username of new owner of catalog.
    pub fn with_owner(mut self, owner: impl Into<Option<String>>) -> Self {
        self.request.owner = owner.into();
        self
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
    ///Name of catalog.
    pub fn with_new_name(mut self, new_name: impl Into<Option<String>>) -> Self {
        self.request.new_name = new_name.into();
        self
    }
}
impl IntoFuture for UpdateCatalogBuilder {
    type Output = Result<CatalogInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_catalog(&request).await })
    }
}
/// Builder for creating requests
pub struct DeleteCatalogBuilder {
    client: CatalogClient,
    request: DeleteCatalogRequest,
}
impl DeleteCatalogBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: CatalogClient, name: impl Into<String>) -> Self {
        let request = DeleteCatalogRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    ///Force deletion even if the catalog is not empty.
    pub fn with_force(mut self, force: impl Into<Option<bool>>) -> Self {
        self.request.force = force.into();
        self
    }
}
impl IntoFuture for DeleteCatalogBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_catalog(&request).await })
    }
}
