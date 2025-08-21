#![allow(unused_mut)]
use futures::future::BoxFuture;
use std::future::IntoFuture;
use crate::error::Result;
use unitycatalog_common::models::volumes::v1::*;
use super::client::*;
/// Builder for creating requests
pub struct CreateVolumeBuilder {
    client: VolumeClient,
    request: CreateVolumeRequest,
}
impl CreateVolumeBuilder {
    /// Create a new builder instance
    pub fn new(
        client: VolumeClient,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        name: impl Into<String>,
        volume_type: i32,
    ) -> Self {
        let request = CreateVolumeRequest {
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            name: name.into(),
            volume_type,
            ..Default::default()
        };
        Self { client, request }
    }
    #[doc = concat!("Set ", "storage_location")]
    pub fn with_storage_location(mut self, storage_location: impl Into<String>) -> Self {
        self.request.storage_location = Some(storage_location.into());
        self
    }
    #[doc = concat!("Set ", "comment")]
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.request.comment = Some(comment.into());
        self
    }
}
impl IntoFuture for CreateVolumeBuilder {
    type Output = Result<VolumeInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_volume(&request).await })
    }
}
/// Builder for creating requests
pub struct UpdateVolumeBuilder {
    client: VolumeClient,
    request: UpdateVolumeRequest,
}
impl UpdateVolumeBuilder {
    /// Create a new builder instance
    pub fn new(client: VolumeClient, name: impl Into<String>) -> Self {
        let request = UpdateVolumeRequest {
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
    #[doc = concat!("Set ", "comment")]
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.request.comment = Some(comment.into());
        self
    }
    #[doc = concat!("Set ", "owner")]
    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.request.owner = Some(owner.into());
        self
    }
    #[doc = concat!("Set ", "include_browse")]
    pub fn with_include_browse(mut self, include_browse: bool) -> Self {
        self.request.include_browse = Some(include_browse);
        self
    }
}
impl IntoFuture for UpdateVolumeBuilder {
    type Output = Result<VolumeInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_volume(&request).await })
    }
}
