use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use reqwest::IntoUrl;

use super::utils::stream_paginated;
pub(super) use crate::codegen::external_locations::ExternalLocationClient as ExternalLocationClientBase;
use crate::models::external_locations::v1::*;
use crate::{Error, Result};

impl ExternalLocationClientBase {
    pub fn list(
        &self,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<ExternalLocationInfo>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |max_results, page_token| async move {
            let request = ListExternalLocationsRequest {
                max_results,
                page_token,
                include_browse: None,
            };
            let res = self.list_external_locations(&request).await?;
            Ok((res.external_locations, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}

#[derive(Clone)]
pub struct ExternalLocationClient {
    name: String,
    client: ExternalLocationClientBase,
}

impl ExternalLocationClient {
    pub fn new(name: impl ToString, client: ExternalLocationClientBase) -> Self {
        Self {
            name: name.to_string(),
            client,
        }
    }

    pub(super) async fn create(
        &self,
        url: impl IntoUrl,
        credential_name: impl Into<String>,
        comment: Option<impl ToString>,
    ) -> Result<ExternalLocationInfo> {
        let request = CreateExternalLocationRequest {
            name: self.name.clone(),
            url: url
                .into_url()
                .map(|u| u.to_string())
                .map_err(|e| Error::generic(e.to_string()))?,
            credential_name: credential_name.into(),
            comment: comment.map(|c| c.to_string()),
            ..Default::default()
        };
        self.client.create_external_location(&request).await
    }

    pub async fn get(&self) -> Result<ExternalLocationInfo> {
        let request = GetExternalLocationRequest {
            name: self.name.clone(),
        };
        self.client.get_external_location(&request).await
    }

    pub async fn update(
        &self,
        new_name: Option<impl ToString>,
        url: Option<impl IntoUrl>,
        credential_name: Option<impl Into<String>>,
        comment: Option<impl ToString>,
        owner: Option<impl ToString>,
        read_only: Option<bool>,
        skip_validation: Option<bool>,
        force: Option<bool>,
    ) -> Result<ExternalLocationInfo> {
        let url = if let Some(url) = url {
            Some(
                url.into_url()
                    .map(|u| u.to_string())
                    .map_err(|e| Error::generic(e.to_string()))?,
            )
        } else {
            None
        };

        let request = UpdateExternalLocationRequest {
            name: self.name.clone(),
            new_name: new_name.map(|s| s.to_string()),
            url,
            credential_name: credential_name.map(|s| s.into()),
            comment: comment.map(|s| s.to_string()),
            owner: owner.map(|s| s.to_string()),
            read_only,
            skip_validation,
            force,
        };
        self.client.update_external_location(&request).await
    }

    pub async fn delete(&self, force: impl Into<Option<bool>>) -> Result<()> {
        let request = DeleteExternalLocationRequest {
            name: self.name.clone(),
            force: force.into(),
        };
        self.client.delete_external_location(&request).await
    }
}
