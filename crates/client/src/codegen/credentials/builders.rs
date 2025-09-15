#![allow(unused_mut)]
use super::client::*;
use crate::{error::Result, utils::stream_paginated};
use futures::{StreamExt, TryStreamExt, future::BoxFuture, stream::BoxStream};
use std::future::IntoFuture;
use unitycatalog_common::models::credentials::v1::*;
/// Builder for creating requests
pub struct ListCredentialsBuilder {
    client: CredentialClient,
    request: ListCredentialsRequest,
}
impl ListCredentialsBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: CredentialClient) -> Self {
        let request = ListCredentialsRequest {
            ..Default::default()
        };
        Self { client, request }
    }
    ///Return only credentials for the specified purpose.
    pub fn with_purpose(mut self, purpose: impl Into<Option<Purpose>>) -> Self {
        self.request.purpose = purpose.into().map(|e| e as i32);
        self
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
    /// Convert paginated request into stream of results
    pub fn into_stream(self) -> BoxStream<'static, Result<CredentialInfo>> {
        stream_paginated(self, move |mut builder, page_token| async move {
            builder.request.page_token = page_token;
            let res = builder.client.list_credentials(&builder.request).await?;
            if let Some(ref mut remaining) = builder.request.max_results {
                *remaining -= res.credentials.len() as i32;
                if *remaining <= 0 {
                    builder.request.max_results = Some(0);
                }
            }
            let next_page_token = res.next_page_token.clone();
            Ok((res, builder, next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.credentials.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}
impl IntoFuture for ListCredentialsBuilder {
    type Output = Result<ListCredentialsResponse>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_credentials(&request).await })
    }
}
/// Builder for creating requests
pub struct CreateCredentialBuilder {
    client: CredentialClient,
    request: CreateCredentialRequest,
}
impl CreateCredentialBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: CredentialClient, name: impl Into<String>, purpose: Purpose) -> Self {
        let request = CreateCredentialRequest {
            name: name.into(),
            purpose: purpose as i32,
            ..Default::default()
        };
        Self { client, request }
    }
    ///Comment associated with the credential.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    ///Whether the credential is usable only for read operations. Only applicable when purpose is STORAGE.
    pub fn with_read_only(mut self, read_only: impl Into<Option<bool>>) -> Self {
        self.request.read_only = read_only.into();
        self
    }
    ///Supplying true to this argument skips validation of the created set of credentials.
    pub fn with_skip_validation(mut self, skip_validation: impl Into<Option<bool>>) -> Self {
        self.request.skip_validation = skip_validation.into();
        self
    }
    #[doc = concat!("Set ", "azure_service_principal")]
    pub fn with_azure_service_principal(
        mut self,
        azure_service_principal: impl Into<Option<AzureServicePrincipal>>,
    ) -> Self {
        self.request.azure_service_principal = azure_service_principal.into();
        self
    }
    #[doc = concat!("Set ", "azure_managed_identity")]
    pub fn with_azure_managed_identity(
        mut self,
        azure_managed_identity: impl Into<Option<AzureManagedIdentity>>,
    ) -> Self {
        self.request.azure_managed_identity = azure_managed_identity.into();
        self
    }
    #[doc = concat!("Set ", "azure_storage_key")]
    pub fn with_azure_storage_key(
        mut self,
        azure_storage_key: impl Into<Option<AzureStorageKey>>,
    ) -> Self {
        self.request.azure_storage_key = azure_storage_key.into();
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
pub struct GetCredentialBuilder {
    client: CredentialClient,
    request: GetCredentialRequest,
}
impl GetCredentialBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: CredentialClient, name: impl Into<String>) -> Self {
        let request = GetCredentialRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for GetCredentialBuilder {
    type Output = Result<CredentialInfo>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_credential(&request).await })
    }
}
/// Builder for creating requests
pub struct UpdateCredentialBuilder {
    client: CredentialClient,
    request: UpdateCredentialRequest,
}
impl UpdateCredentialBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: CredentialClient, name: impl Into<String>) -> Self {
        let request = UpdateCredentialRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    ///Name of credential.
    pub fn with_new_name(mut self, new_name: impl Into<Option<String>>) -> Self {
        self.request.new_name = new_name.into();
        self
    }
    ///Comment associated with the credential.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    ///Whether the credential is usable only for read operations. Only applicable when purpose is STORAGE.
    pub fn with_read_only(mut self, read_only: impl Into<Option<bool>>) -> Self {
        self.request.read_only = read_only.into();
        self
    }
    ///Username of current owner of credential.
    pub fn with_owner(mut self, owner: impl Into<Option<String>>) -> Self {
        self.request.owner = owner.into();
        self
    }
    ///Supply true to this argument to skip validation of the updated credential.
    pub fn with_skip_validation(mut self, skip_validation: impl Into<Option<bool>>) -> Self {
        self.request.skip_validation = skip_validation.into();
        self
    }
    /**Force an update even if there are dependent services (when purpose is SERVICE)
    or dependent external locations and external tables (when purpose is STORAGE).*/
    pub fn with_force(mut self, force: impl Into<Option<bool>>) -> Self {
        self.request.force = force.into();
        self
    }
    #[doc = concat!("Set ", "azure_service_principal")]
    pub fn with_azure_service_principal(
        mut self,
        azure_service_principal: impl Into<Option<AzureServicePrincipal>>,
    ) -> Self {
        self.request.azure_service_principal = azure_service_principal.into();
        self
    }
    #[doc = concat!("Set ", "azure_managed_identity")]
    pub fn with_azure_managed_identity(
        mut self,
        azure_managed_identity: impl Into<Option<AzureManagedIdentity>>,
    ) -> Self {
        self.request.azure_managed_identity = azure_managed_identity.into();
        self
    }
    #[doc = concat!("Set ", "azure_storage_key")]
    pub fn with_azure_storage_key(
        mut self,
        azure_storage_key: impl Into<Option<AzureStorageKey>>,
    ) -> Self {
        self.request.azure_storage_key = azure_storage_key.into();
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
/// Builder for creating requests
pub struct DeleteCredentialBuilder {
    client: CredentialClient,
    request: DeleteCredentialRequest,
}
impl DeleteCredentialBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: CredentialClient, name: impl Into<String>) -> Self {
        let request = DeleteCredentialRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for DeleteCredentialBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_credential(&request).await })
    }
}
