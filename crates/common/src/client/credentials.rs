use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};

use super::utils::stream_paginated;
pub(super) use crate::api::codegen::credentials::CredentialClient as CredentialClientBase;
use crate::models::credentials::v1::*;
use crate::{Error, Result};

impl CredentialClientBase {
    pub fn list(
        &self,
        purpose: Option<Purpose>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<CredentialInfo>> {
        let max_results = max_results.into();
        let purpose = purpose.map(|p| p as i32);
        stream_paginated(max_results, move |max_results, page_token| async move {
            let request = ListCredentialsRequest {
                max_results,
                page_token,
                purpose,
            };
            let res = self
                .list_credentials(&request)
                .await
                .map_err(|e| Error::generic(e.to_string()))?;
            Ok((res.credentials, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}

#[derive(Clone)]
pub struct CredentialClient {
    name: String,
    client: CredentialClientBase,
}

impl CredentialClient {
    pub fn new(name: impl ToString, client: CredentialClientBase) -> Self {
        Self {
            name: name.to_string(),
            client,
        }
    }

    pub(super) async fn create(
        &self,
        purpose: Purpose,
        comment: Option<impl ToString>,
    ) -> Result<CredentialInfo> {
        let request = CreateCredentialRequest {
            name: self.name.clone(),
            purpose: purpose.into(),
            comment: comment.map(|s| s.to_string()),
            ..Default::default()
        };
        self.client.create_credential(&request).await
    }

    pub async fn get(&self) -> Result<CredentialInfo> {
        let request = GetCredentialRequest {
            name: self.name.clone(),
        };
        self.client.get_credential(&request).await
    }

    pub async fn update(
        &self,
        new_name: Option<impl ToString>,
        comment: Option<impl ToString>,
        owner: Option<impl ToString>,
        read_only: Option<bool>,
        skip_validation: Option<bool>,
        force: Option<bool>,
        credential: Option<update_credential_request::Credential>,
    ) -> Result<CredentialInfo> {
        let request = UpdateCredentialRequest {
            name: self.name.clone(),
            new_name: new_name.map(|s| s.to_string()),
            comment: comment.map(|s| s.to_string()),
            owner: owner.map(|s| s.to_string()),
            read_only,
            skip_validation,
            force,
            credential,
        };
        self.client.update_credential(&request).await
    }

    pub async fn delete(&self) -> Result<()> {
        let request = DeleteCredentialRequest {
            name: self.name.clone(),
        };
        self.client.delete_credential(&request).await
    }
}
