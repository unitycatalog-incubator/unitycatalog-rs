// @generated — do not edit by hand.
//! Handler trait for [`TemporaryCredentialHandler`].
//!
//! Implement this trait to provide a custom backend for this service.
//! Register your implementation with the generated route setup functions.
//!
//! # Composability
//!
//! A single struct can implement multiple handler traits to serve multiple
//! services. Use [`axum::Router::merge`] to compose routers together.
//!
//! Service for generating temporary credentials to access tables and storage paths.
//! Credentials are short-lived and scoped to a specific operation (read or read/write).
use crate::Result;
use crate::api::RequestContext;
use async_trait::async_trait;
use unitycatalog_common::models::temporary_credentials::v1::*;
#[async_trait]
pub trait TemporaryCredentialHandler: Send + Sync + 'static {
    /// Generate a new set of credentials for a table.
    async fn generate_temporary_table_credentials(
        &self,
        request: GenerateTemporaryTableCredentialsRequest,
        context: RequestContext,
    ) -> Result<TemporaryCredential>;
    /// Generate a new set of credentials for a path.
    async fn generate_temporary_path_credentials(
        &self,
        request: GenerateTemporaryPathCredentialsRequest,
        context: RequestContext,
    ) -> Result<TemporaryCredential>;
}
