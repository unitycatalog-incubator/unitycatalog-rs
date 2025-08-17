use std::collections::HashMap;

use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::SchemaInfo;
use unitycatalog_common::models::catalogs::v1::*;
use unitycatalog_common::schemas::v1::CreateSchemaRequest;

use super::schemas::{SchemaClient, SchemaClientBase};
use super::utils::stream_paginated;
use crate::Result;
pub(super) use crate::codegen::catalogs::CatalogClient as CatalogClientBase;

impl CatalogClientBase {
    pub fn list(&self, max_results: impl Into<Option<i32>>) -> BoxStream<'_, Result<CatalogInfo>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |max_results, page_token| async move {
            let request = ListCatalogsRequest {
                max_results,
                page_token,
            };
            let res = self.list_catalogs(&request).await?;
            Ok((res.catalogs, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}

#[derive(Clone)]
pub struct CatalogClient {
    name: String,
    client: CatalogClientBase,
}

impl CatalogClient {
    pub fn new(name: impl ToString, client: CatalogClientBase) -> Self {
        Self {
            name: name.to_string(),
            client,
        }
    }

    pub fn schema(&self, name: impl Into<String>) -> SchemaClient {
        SchemaClient::new(
            self.name.clone(),
            name.into(),
            SchemaClientBase::new(self.client.client.clone(), self.client.base_url.clone()),
        )
    }

    /// Create a new managed catalog.
    pub(super) async fn create(
        &self,
        storage_root: Option<impl ToString>,
        comment: Option<impl ToString>,
        properties: impl Into<Option<HashMap<String, String>>>,
    ) -> Result<CatalogInfo> {
        let request = CreateCatalogRequest {
            name: self.name.clone(),
            comment: comment.map(|s| s.to_string()),
            properties: properties.into().unwrap_or_default(),
            storage_root: storage_root.map(|s| s.to_string()),
            ..Default::default()
        };
        self.client.create_catalog(&request).await
    }

    pub(super) async fn create_sharing(
        &self,
        provider_name: impl Into<String>,
        share_name: impl Into<String>,
        comment: Option<impl ToString>,
        properties: impl Into<Option<HashMap<String, String>>>,
    ) -> Result<CatalogInfo> {
        let request = CreateCatalogRequest {
            name: self.name.clone(),
            comment: comment.map(|s| s.to_string()),
            properties: properties.into().unwrap_or_default(),
            share_name: Some(share_name.into()),
            provider_name: Some(provider_name.into()),
            ..Default::default()
        };
        self.client.create_catalog(&request).await
    }

    /// Create a new schema in this catalog.
    pub async fn create_schema(
        &self,
        name: impl ToString,
        comment: Option<impl ToString>,
    ) -> Result<SchemaInfo> {
        let request = CreateSchemaRequest {
            catalog_name: self.name.clone(),
            name: name.to_string(),
            comment: comment.map(|s| s.to_string()),
            ..Default::default()
        };
        let schemas_client = super::schemas::SchemaClientBase::new(
            self.client.client.clone(),
            self.client.base_url.clone(),
        );
        schemas_client.create_schema(&request).await
    }

    pub async fn get(&self) -> Result<CatalogInfo> {
        let request = GetCatalogRequest {
            name: self.name.clone(),
            include_browse: None,
        };
        self.client.get_catalog(&request).await
    }

    pub async fn update(
        &self,
        new_name: Option<impl ToString>,
        comment: Option<impl ToString>,
        owner: Option<impl ToString>,
        properties: impl Into<Option<HashMap<String, String>>>,
    ) -> Result<CatalogInfo> {
        let request = UpdateCatalogRequest {
            name: self.name.clone(),
            new_name: new_name
                .map(|s| s.to_string())
                .unwrap_or_else(|| self.name.clone()),
            comment: comment.map(|s| s.to_string()),
            owner: owner.map(|s| s.to_string()),
            properties: properties.into().unwrap_or_default(),
        };
        self.client.update_catalog(&request).await
    }

    pub async fn delete(&self, force: impl Into<Option<bool>>) -> Result<()> {
        let request = DeleteCatalogRequest {
            name: self.name.clone(),
            force: force.into(),
        };
        self.client.delete_catalog(&request).await
    }
}
