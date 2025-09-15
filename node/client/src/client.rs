use std::collections::HashMap;

use futures::TryStreamExt;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use prost::Message;
use unitycatalog_client::{CatalogClient as UCCatalogClient, UnityCatalogClient as UCClient};

use crate::error::NapiErrorExt;

#[napi]
pub struct UnityCatalogClient {
    inner: UCClient,
}

#[napi]
impl UnityCatalogClient {
    pub fn new(inner: UCClient) -> Self {
        Self { inner }
    }

    #[napi(factory)]
    pub fn from_url(base_url: String, token: Option<String>) -> napi::Result<Self> {
        let client = if let Some(token) = token {
            cloud_client::CloudClient::new_with_token(token)
        } else {
            cloud_client::CloudClient::new_unauthenticated()
        };
        let base_url = base_url.parse().unwrap();
        Ok(Self {
            inner: UCClient::new(client, base_url),
        })
    }

    #[napi(catch_unwind)]
    pub async fn list_catalogs(&self, max_results: Option<i32>) -> napi::Result<Vec<Buffer>> {
        self.inner
            .list_catalogs()
            .with_max_results(max_results)
            .into_stream()
            .map_ok(|catalog| Buffer::from(catalog.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }

    #[napi(catch_unwind)]
    pub async fn create_catalog(
        &self,
        name: String,
        storage_root: Option<String>,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> napi::Result<Buffer> {
        let mut request = self
            .inner
            .create_catalog(name)
            .with_storage_root(storage_root)
            .with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request
            .await
            .map(|catalog| Buffer::from(catalog.encode_to_vec()))
            .default_error()
    }

    #[napi(catch_unwind)]
    pub async fn create_sharing_catalog(
        &self,
        name: String,
        provider_name: String,
        share_name: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> napi::Result<Buffer> {
        let mut request = self
            .inner
            .create_catalog(name)
            .with_provider_name(provider_name)
            .with_share_name(share_name)
            .with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request
            .await
            .map(|catalog| Buffer::from(catalog.encode_to_vec()))
            .default_error()
    }

    #[napi]
    pub fn catalog(&self, name: String) -> CatalogClient {
        CatalogClient::new(self.inner.catalog(name))
    }
}

#[napi]
pub struct CatalogClient {
    inner: UCCatalogClient,
}

#[napi]
impl CatalogClient {
    pub fn new(inner: UCCatalogClient) -> Self {
        Self { inner }
    }

    #[napi(catch_unwind)]
    pub async fn get(&self) -> napi::Result<Buffer> {
        self.inner
            .get()
            .await
            .map(|catalog| Buffer::from(catalog.encode_to_vec()))
            .default_error()
    }

    #[napi(catch_unwind)]
    pub async fn update(
        &self,
        new_name: Option<String>,
        comment: Option<String>,
        owner: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> napi::Result<Buffer> {
        let mut request = self
            .inner
            .update()
            .with_new_name(new_name)
            .with_comment(comment)
            .with_owner(owner);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request
            .await
            .map(|catalog| Buffer::from(catalog.encode_to_vec()))
            .default_error()
    }

    /// Deletes the catalog.
    #[napi(catch_unwind)]
    pub async fn delete(&self, force: Option<bool>) -> napi::Result<()> {
        self.inner.delete().with_force(force).await.default_error()
    }
}
