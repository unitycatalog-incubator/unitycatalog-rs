#![allow(unused_mut)]
use futures::future::BoxFuture;
use std::future::IntoFuture;
use crate::error::Result;
use unitycatalog_common::models::temporary_credentials::v1::*;
use super::client::*;
/// Builder for creating requests
pub struct GenerateTemporaryTableCredentialsBuilder {
    client: TemporaryCredentialClient,
    request: GenerateTemporaryTableCredentialsRequest,
}
impl GenerateTemporaryTableCredentialsBuilder {
    /// Create a new builder instance
    pub fn new(
        client: TemporaryCredentialClient,
        table_id: impl Into<String>,
        operation: i32,
    ) -> Self {
        let request = GenerateTemporaryTableCredentialsRequest {
            table_id: table_id.into(),
            operation,
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for GenerateTemporaryTableCredentialsBuilder {
    type Output = Result<TemporaryCredential>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move {
            client.generate_temporary_table_credentials(&request).await
        })
    }
}
/// Builder for creating requests
pub struct GenerateTemporaryPathCredentialsBuilder {
    client: TemporaryCredentialClient,
    request: GenerateTemporaryPathCredentialsRequest,
}
impl GenerateTemporaryPathCredentialsBuilder {
    /// Create a new builder instance
    pub fn new(
        client: TemporaryCredentialClient,
        url: impl Into<String>,
        operation: i32,
    ) -> Self {
        let request = GenerateTemporaryPathCredentialsRequest {
            url: url.into(),
            operation,
            ..Default::default()
        };
        Self { client, request }
    }
    #[doc = concat!("Set ", "dry_run")]
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.request.dry_run = Some(dry_run);
        self
    }
}
impl IntoFuture for GenerateTemporaryPathCredentialsBuilder {
    type Output = Result<TemporaryCredential>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move {
            client.generate_temporary_path_credentials(&request).await
        })
    }
}
