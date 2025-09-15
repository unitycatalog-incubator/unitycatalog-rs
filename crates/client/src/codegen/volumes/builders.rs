#![allow(unused_mut)]
use super::client::*;
use crate::{error::Result, utils::stream_paginated};
use futures::{StreamExt, TryStreamExt, future::BoxFuture, stream::BoxStream};
use std::future::IntoFuture;
use unitycatalog_common::models::volumes::v1::*;
/// Builder for creating requests
pub struct ListVolumesBuilder {
    client: VolumeClient,
    request: ListVolumesRequest,
}
impl ListVolumesBuilder {
    /// Create a new builder instance
    pub(crate) fn new(
        client: VolumeClient,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
    ) -> Self {
        let request = ListVolumesRequest {
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
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
    pub fn into_stream(self) -> BoxStream<'static, Result<VolumeInfo>> {
        stream_paginated(self, move |mut builder, page_token| async move {
            builder.request.page_token = page_token;
            let res = builder.client.list_volumes(&builder.request).await?;
            if let Some(ref mut remaining) = builder.request.max_results {
                *remaining -= res.volumes.len() as i32;
                if *remaining <= 0 {
                    builder.request.max_results = Some(0);
                }
            }
            let next_page_token = res.next_page_token.clone();
            Ok((res, builder, next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.volumes.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}
impl IntoFuture for ListVolumesBuilder {
    type Output = Result<ListVolumesResponse>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_volumes(&request).await })
    }
}
/// Builder for creating requests
pub struct CreateVolumeBuilder {
    client: VolumeClient,
    request: CreateVolumeRequest,
}
impl CreateVolumeBuilder {
    /// Create a new builder instance
    pub(crate) fn new(
        client: VolumeClient,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        name: impl Into<String>,
        volume_type: VolumeType,
    ) -> Self {
        let request = CreateVolumeRequest {
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            name: name.into(),
            volume_type: volume_type as i32,
            ..Default::default()
        };
        Self { client, request }
    }
    ///The storage location on the cloud
    pub fn with_storage_location(mut self, storage_location: impl Into<Option<String>>) -> Self {
        self.request.storage_location = storage_location.into();
        self
    }
    ///The storage location on the cloud
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
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
pub struct GetVolumeBuilder {
    client: VolumeClient,
    request: GetVolumeRequest,
}
impl GetVolumeBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: VolumeClient, name: impl Into<String>) -> Self {
        let request = GetVolumeRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    ///Whether to include schemas in the response for which the principal can only access selective metadata for
    pub fn with_include_browse(mut self, include_browse: impl Into<Option<bool>>) -> Self {
        self.request.include_browse = include_browse.into();
        self
    }
}
impl IntoFuture for GetVolumeBuilder {
    type Output = Result<VolumeInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_volume(&request).await })
    }
}
/// Builder for creating requests
pub struct UpdateVolumeBuilder {
    client: VolumeClient,
    request: UpdateVolumeRequest,
}
impl UpdateVolumeBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: VolumeClient, name: impl Into<String>) -> Self {
        let request = UpdateVolumeRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    ///New name for the volume.
    pub fn with_new_name(mut self, new_name: impl Into<Option<String>>) -> Self {
        self.request.new_name = new_name.into();
        self
    }
    ///The comment attached to the volume
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    ///The identifier of the user who owns the volume
    pub fn with_owner(mut self, owner: impl Into<Option<String>>) -> Self {
        self.request.owner = owner.into();
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
/// Builder for creating requests
pub struct DeleteVolumeBuilder {
    client: VolumeClient,
    request: DeleteVolumeRequest,
}
impl DeleteVolumeBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: VolumeClient, name: impl Into<String>) -> Self {
        let request = DeleteVolumeRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for DeleteVolumeBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_volume(&request).await })
    }
}
