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
