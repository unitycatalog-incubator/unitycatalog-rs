use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use reqwest::IntoUrl;
use unitycatalog_common::models::external_locations::v1::*;

use super::utils::stream_paginated;
pub(super) use crate::codegen::external_locations::ExternalLocationClient as ExternalLocationClientBase;
use crate::codegen::external_locations::builders::{
    CreateExternalLocationBuilder, GetExternalLocationBuilder, UpdateExternalLocationBuilder,
};
use crate::{Error, Result};

impl ExternalLocationClientBase {
    pub fn list(
        &self,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<ExternalLocationInfo>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |mut max_results, page_token| async move {
            let request = ListExternalLocationsRequest {
                max_results,
                page_token,
                include_browse: None,
            };
            let res = self.list_external_locations(&request).await?;

            // Update max_results for next page based on items received
            if let Some(ref mut remaining) = max_results {
                *remaining -= res.external_locations.len() as i32;
                if *remaining <= 0 {
                    max_results = Some(0);
                }
            }

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

    /// Create a new external location using the builder pattern.
    pub fn create(
        &self,
        url: impl IntoUrl,
        credential_name: impl Into<String>,
    ) -> Result<CreateExternalLocationBuilder> {
        let url = url
            .into_url()
            .map(|u| u.to_string())
            .map_err(|e| Error::generic(e.to_string()))?;
        Ok(CreateExternalLocationBuilder::new(
            self.client.clone(),
            &self.name,
            url,
            credential_name.into(),
        ))
    }

    /// Get an external location using the builder pattern.
    pub fn get(&self) -> GetExternalLocationBuilder {
        GetExternalLocationBuilder::new(self.client.clone(), &self.name)
    }

    /// Update this external location using the builder pattern.
    pub fn update(&self) -> UpdateExternalLocationBuilder {
        UpdateExternalLocationBuilder::new(self.client.clone(), &self.name)
    }

    pub async fn delete(&self, force: impl Into<Option<bool>>) -> Result<()> {
        let request = DeleteExternalLocationRequest {
            name: self.name.clone(),
            force: force.into(),
        };
        self.client.delete_external_location(&request).await
    }
}
