#![allow(unused_mut)]
use super::client::*;
use crate::error::Result;
use futures::future::BoxFuture;
use std::future::IntoFuture;
use unitycatalog_common::models::catalogs::v1::*;
/// Builder for creating requests
pub struct CreateCatalogBuilder {
    client: CatalogClient,
    request: CreateCatalogRequest,
}
impl CreateCatalogBuilder {
    /// Create a new builder instance
    pub fn new(client: CatalogClient, name: impl Into<String>) -> Self {
        let request = CreateCatalogRequest {
            name: name.into(),
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
    #[doc = concat!("Set ", "storage_root")]
    pub fn with_storage_root(mut self, storage_root: impl Into<Option<String>>) -> Self {
        self.request.storage_root = storage_root.into();
        self
    }
    #[doc = concat!("Set ", "provider_name")]
    pub fn with_provider_name(mut self, provider_name: impl Into<Option<String>>) -> Self {
        self.request.provider_name = provider_name.into();
        self
    }
    #[doc = concat!("Set ", "share_name")]
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
    pub fn new(client: CatalogClient, name: impl Into<String>) -> Self {
        let request = GetCatalogRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    #[doc = concat!("Set ", "include_browse")]
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
    pub fn new(client: CatalogClient, name: impl Into<String>) -> Self {
        let request = UpdateCatalogRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
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
    #[doc = concat!("Set ", "new_name")]
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
