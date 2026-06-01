// @generated — do not edit by hand.
//! Handler trait for [`DeltaCommitHandler`].
//!
//! Implement this trait to provide a custom backend for this service, then mount the
//! generated handler functions (in the sibling `server` module) onto an `axum::Router`
//! with your implementation as state.
//!
//! # Composability
//!
//! A single struct can implement multiple handler traits to serve multiple
//! services. Use [`axum::Router::merge`] to compose per-service routers together.
//!
//! Service implementing the Delta catalog-managed commits ("commit coordinator")
//! protocol. The catalog is the source of truth for which commit wins each
//! version.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::delta_commits::v1::*;
#[async_trait]
pub trait DeltaCommitHandler<Cx = crate::api::RequestContext>: Send + Sync + 'static {
    /// Ratify a staged commit at the requested version (first-writer-wins), and/or
    /// notify the catalog that commits have been backfilled to the Delta log.
    async fn commit(&self, request: CommitRequest, context: Cx) -> Result<()>;
    /// Return ratified-but-unpublished commits for a table, plus the latest
    /// version the catalog tracks.
    async fn get_commits(
        &self,
        request: GetCommitsRequest,
        context: Cx,
    ) -> Result<GetCommitsResponse>;
}
