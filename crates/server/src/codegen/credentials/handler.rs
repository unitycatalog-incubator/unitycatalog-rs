// @generated — do not edit by hand.
//! Handler trait for [`CredentialHandler`].
//!
//! Implement this trait to provide a custom backend for this service.
//! Register your implementation with the generated route setup functions.
//!
//! # Composability
//!
//! A single struct can implement multiple handler traits to serve multiple
//! services. Use [`axum::Router::merge`] to compose routers together.
//!
//! Manage credentials to access external data sources and services
//! as well as generate signed urls for the Delta Sharing service.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::credentials::v1::*;
#[async_trait]
pub trait CredentialHandler<Cx = crate::api::RequestContext>: Send + Sync + 'static {
    async fn list_credentials(
        &self,
        request: ListCredentialsRequest,
        context: Cx,
    ) -> Result<ListCredentialsResponse>;
    async fn create_credential(
        &self,
        request: CreateCredentialRequest,
        context: Cx,
    ) -> Result<Credential>;
    async fn get_credential(
        &self,
        request: GetCredentialRequest,
        context: Cx,
    ) -> Result<Credential>;
    async fn update_credential(
        &self,
        request: UpdateCredentialRequest,
        context: Cx,
    ) -> Result<Credential>;
    async fn delete_credential(&self, request: DeleteCredentialRequest, context: Cx) -> Result<()>;
}
