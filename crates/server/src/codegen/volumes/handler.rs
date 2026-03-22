// @generated — do not edit by hand.
//! Handler trait for [`VolumeHandler`].
//!
//! Implement this trait to provide a custom backend for this service.
//! Register your implementation with the generated route setup functions.
//!
//! # Composability
//!
//! A single struct can implement multiple handler traits to serve multiple
//! services. Use [`axum::Router::merge`] to compose routers together.
//!
//! Service for managing volumes in Unity Catalog.
//! Volumes represent logical storage locations (managed or external) within a schema.
use crate::Result;
use crate::api::RequestContext;
use async_trait::async_trait;
use unitycatalog_common::models::volumes::v1::*;
#[async_trait]
pub trait VolumeHandler: Send + Sync + 'static {
    /// Lists volumes.
    async fn list_volumes(
        &self,
        request: ListVolumesRequest,
        context: RequestContext,
    ) -> Result<ListVolumesResponse>;
    async fn create_volume(
        &self,
        request: CreateVolumeRequest,
        context: RequestContext,
    ) -> Result<Volume>;
    async fn get_volume(
        &self,
        request: GetVolumeRequest,
        context: RequestContext,
    ) -> Result<Volume>;
    async fn update_volume(
        &self,
        request: UpdateVolumeRequest,
        context: RequestContext,
    ) -> Result<Volume>;
    async fn delete_volume(
        &self,
        request: DeleteVolumeRequest,
        context: RequestContext,
    ) -> Result<()>;
}
