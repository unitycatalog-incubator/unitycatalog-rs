#![allow(unused_mut)]
use futures::future::BoxFuture;
use std::future::IntoFuture;
use crate::error::Result;
use unitycatalog_common::models::external_locations::v1::*;
use super::client::*;
/// Builder for creating requests
pub struct CreateExternalLocationBuilder {
    client: ExternalLocationClient,
    request: CreateExternalLocationRequest,
}
impl CreateExternalLocationBuilder {
    /// Create a new builder instance
    pub fn new(
        client: ExternalLocationClient,
        name: impl Into<String>,
        url: impl Into<String>,
        credential_name: impl Into<String>,
    ) -> Self {
        let request = CreateExternalLocationRequest {
            name: name.into(),
            url: url.into(),
            credential_name: credential_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    #[doc = concat!("Set ", "read_only")]
    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.request.read_only = Some(read_only);
        self
    }
    #[doc = concat!("Set ", "comment")]
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.request.comment = Some(comment.into());
        self
    }
    #[doc = concat!("Set ", "skip_validation")]
    pub fn with_skip_validation(mut self, skip_validation: bool) -> Self {
        self.request.skip_validation = Some(skip_validation);
        self
    }
}
impl IntoFuture for CreateExternalLocationBuilder {
    type Output = Result<ExternalLocationInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_external_location(&request).await })
    }
}
/// Builder for creating requests
pub struct UpdateExternalLocationBuilder {
    client: ExternalLocationClient,
    request: UpdateExternalLocationRequest,
}
impl UpdateExternalLocationBuilder {
    /// Create a new builder instance
    pub fn new(client: ExternalLocationClient, name: impl Into<String>) -> Self {
        let request = UpdateExternalLocationRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    #[doc = concat!("Set ", "url")]
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.request.url = Some(url.into());
        self
    }
    #[doc = concat!("Set ", "credential_name")]
    pub fn with_credential_name(mut self, credential_name: impl Into<String>) -> Self {
        self.request.credential_name = Some(credential_name.into());
        self
    }
    #[doc = concat!("Set ", "read_only")]
    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.request.read_only = Some(read_only);
        self
    }
    #[doc = concat!("Set ", "owner")]
    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.request.owner = Some(owner.into());
        self
    }
    #[doc = concat!("Set ", "comment")]
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.request.comment = Some(comment.into());
        self
    }
    #[doc = concat!("Set ", "new_name")]
    pub fn with_new_name(mut self, new_name: impl Into<String>) -> Self {
        self.request.new_name = Some(new_name.into());
        self
    }
    #[doc = concat!("Set ", "force")]
    pub fn with_force(mut self, force: bool) -> Self {
        self.request.force = Some(force);
        self
    }
    #[doc = concat!("Set ", "skip_validation")]
    pub fn with_skip_validation(mut self, skip_validation: bool) -> Self {
        self.request.skip_validation = Some(skip_validation);
        self
    }
}
impl IntoFuture for UpdateExternalLocationBuilder {
    type Output = Result<ExternalLocationInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_external_location(&request).await })
    }
}
