#![allow(unused_mut)]
use super::client::*;
use crate::error::Result;
use futures::future::BoxFuture;
use std::future::IntoFuture;
use unitycatalog_common::models::external_locations::v1::*;
/// Builder for creating requests
pub struct CreateExternalLocationBuilder {
    client: ExternalLocationClient,
    request: CreateExternalLocationRequest,
}
impl CreateExternalLocationBuilder {
    /// Create a new builder instance
    pub(crate) fn new(
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
    ///Indicates whether the external location is read-only.
    pub fn with_read_only(mut self, read_only: impl Into<Option<bool>>) -> Self {
        self.request.read_only = read_only.into();
        self
    }
    ///User-provided free-form text description.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    ///Skips validation of the storage credential associated with the external location.
    pub fn with_skip_validation(mut self, skip_validation: impl Into<Option<bool>>) -> Self {
        self.request.skip_validation = skip_validation.into();
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
pub struct GetExternalLocationBuilder {
    client: ExternalLocationClient,
    request: GetExternalLocationRequest,
}
impl GetExternalLocationBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: ExternalLocationClient, name: impl Into<String>) -> Self {
        let request = GetExternalLocationRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for GetExternalLocationBuilder {
    type Output = Result<ExternalLocationInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_external_location(&request).await })
    }
}
/// Builder for creating requests
pub struct UpdateExternalLocationBuilder {
    client: ExternalLocationClient,
    request: UpdateExternalLocationRequest,
}
impl UpdateExternalLocationBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: ExternalLocationClient, name: impl Into<String>) -> Self {
        let request = UpdateExternalLocationRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    ///Path URL of the external location.
    pub fn with_url(mut self, url: impl Into<Option<String>>) -> Self {
        self.request.url = url.into();
        self
    }
    ///Name of the storage credential used with this location.
    pub fn with_credential_name(mut self, credential_name: impl Into<Option<String>>) -> Self {
        self.request.credential_name = credential_name.into();
        self
    }
    ///Indicates whether the external location is read-only.
    pub fn with_read_only(mut self, read_only: impl Into<Option<bool>>) -> Self {
        self.request.read_only = read_only.into();
        self
    }
    ///owner of the external location.
    pub fn with_owner(mut self, owner: impl Into<Option<String>>) -> Self {
        self.request.owner = owner.into();
        self
    }
    ///User-provided free-form text description.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    ///new name of the external location.
    pub fn with_new_name(mut self, new_name: impl Into<Option<String>>) -> Self {
        self.request.new_name = new_name.into();
        self
    }
    ///force update of the external location.
    pub fn with_force(mut self, force: impl Into<Option<bool>>) -> Self {
        self.request.force = force.into();
        self
    }
    ///Skips validation of the storage credential associated with the external location.
    pub fn with_skip_validation(mut self, skip_validation: impl Into<Option<bool>>) -> Self {
        self.request.skip_validation = skip_validation.into();
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
