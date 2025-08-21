#![allow(unused_mut)]
use super::client::*;
use crate::error::Result;
use futures::future::BoxFuture;
use std::future::IntoFuture;
use unitycatalog_common::models::credentials::v1::*;
/// Builder for creating requests
pub struct CreateCredentialBuilder {
    client: CredentialClient,
    request: CreateCredentialRequest,
}
impl CreateCredentialBuilder {
    /// Create a new builder instance
    pub fn new(client: CredentialClient, name: impl Into<String>, purpose: i32) -> Self {
        let request = CreateCredentialRequest {
            name: name.into(),
            purpose,
            ..Default::default()
        };
        Self { client, request }
    }
    #[doc = concat!("Set ", "comment")]
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.request.comment = Some(comment.into());
        self
    }
    #[doc = concat!("Set ", "read_only")]
    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.request.read_only = Some(read_only);
        self
    }
    #[doc = concat!("Set ", "skip_validation")]
    pub fn with_skip_validation(mut self, skip_validation: bool) -> Self {
        self.request.skip_validation = Some(skip_validation);
        self
    }
}
impl IntoFuture for CreateCredentialBuilder {
    type Output = Result<CredentialInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_credential(&request).await })
    }
}
/// Builder for creating requests
pub struct UpdateCredentialBuilder {
    client: CredentialClient,
    request: UpdateCredentialRequest,
}
impl UpdateCredentialBuilder {
    /// Create a new builder instance
    pub fn new(client: CredentialClient, name: impl Into<String>) -> Self {
        let request = UpdateCredentialRequest {
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
    #[doc = concat!("Set ", "skip_validation")]
    pub fn with_skip_validation(mut self, skip_validation: bool) -> Self {
        self.request.skip_validation = Some(skip_validation);
        self
    }
    #[doc = concat!("Set ", "force")]
    pub fn with_force(mut self, force: bool) -> Self {
        self.request.force = Some(force);
        self
    }
}
impl IntoFuture for UpdateCredentialBuilder {
    type Output = Result<CredentialInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_credential(&request).await })
    }
}
