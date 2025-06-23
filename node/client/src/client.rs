use std::collections::HashMap;

use futures::TryStreamExt;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use prost::Message;
use unitycatalog_common::catalogs::v1::{CreateCatalogRequest, UpdateCatalogRequest};
use unitycatalog_common::google::protobuf::{Struct, Value, value::Kind as ValueKind};
use unitycatalog_common::rest::client::UnityCatalogClient as UCClient;
use unitycatalog_common::schemas::v1::{CreateSchemaRequest, UpdateSchemaRequest};

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
            .catalogs()
            .list(max_results)
            .map_ok(|catalog| Buffer::from(catalog.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }

    #[napi]
    pub fn catalog(&self, name: String) -> CatalogClient {
        CatalogClient::new(name, self.inner.clone())
    }
}

#[napi]
pub struct CatalogClient {
    name: String,
    inner: UCClient,
}

#[napi]
impl CatalogClient {
    pub fn new(name: String, inner: UCClient) -> Self {
        Self { name, inner }
    }

    #[napi(getter)]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[napi(setter)]
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    #[napi]
    pub fn schema(&self, name: String) -> SchemaClient {
        SchemaClient::new(name, self.name.clone(), self.inner.clone())
    }

    #[napi(catch_unwind)]
    pub async fn get(&self) -> napi::Result<Buffer> {
        self.inner
            .catalogs()
            .get(&self.name)
            .await
            .map(|catalog| Buffer::from(catalog.encode_to_vec()))
            .default_error()
    }

    #[napi(catch_unwind)]
    pub async fn create(
        &self,
        comment: Option<String>,
        storage_root: Option<String>,
        provider_name: Option<String>,
        share_name: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> napi::Result<Buffer> {
        let request = CreateCatalogRequest {
            name: self.name.clone(),
            comment,
            properties: properties.map(hash_map_to_struct),
            storage_root,
            provider_name,
            share_name,
        };
        self.inner
            .catalogs()
            .create_catalog(&request)
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
        let request = UpdateCatalogRequest {
            name: self.name.clone(),
            comment,
            new_name: new_name.unwrap_or_else(|| self.name.clone()),
            owner,
            properties: properties.map(hash_map_to_struct),
        };
        self.inner
            .catalogs()
            .update_catalog(&request)
            .await
            .map(|catalog| Buffer::from(catalog.encode_to_vec()))
            .default_error()
    }

    #[napi(catch_unwind)]
    pub async fn list_schemas(&self, max_results: Option<i32>) -> napi::Result<Vec<Buffer>> {
        self.inner
            .schemas()
            .list(&self.name, max_results)
            .map_ok(|schema| Buffer::from(schema.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
}

#[napi]
pub struct SchemaClient {
    name: String,
    catalog_name: String,
    inner: UCClient,
}

#[napi]
impl SchemaClient {
    pub fn new(name: String, catalog_name: String, inner: UCClient) -> Self {
        Self {
            name,
            catalog_name,
            inner,
        }
    }

    #[napi(getter)]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[napi(setter)]
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    #[napi(getter)]
    pub fn catalog_name(&self) -> &str {
        &self.catalog_name
    }

    #[napi(setter)]
    pub fn set_catalog_name(&mut self, catalog_name: String) {
        self.catalog_name = catalog_name;
    }

    #[napi(catch_unwind)]
    pub async fn get(&self) -> napi::Result<Buffer> {
        self.inner
            .schemas()
            .get(&self.catalog_name, &self.name)
            .await
            .map(|schema| Buffer::from(schema.encode_to_vec()))
            .default_error()
    }

    #[napi(catch_unwind)]
    pub async fn create(
        &self,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> napi::Result<Buffer> {
        let request = CreateSchemaRequest {
            name: self.name.clone(),
            catalog_name: self.catalog_name.clone(),
            comment,
            properties: properties.map(hash_map_to_struct),
        };
        self.inner
            .schemas()
            .create_schema(&request)
            .await
            .map(|schema| Buffer::from(schema.encode_to_vec()))
            .default_error()
    }

    #[napi(catch_unwind)]
    pub async fn update(
        &self,
        new_name: Option<String>,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> napi::Result<Buffer> {
        let request = UpdateSchemaRequest {
            full_name: format!("{}.{}", self.catalog_name, self.name),
            comment,
            properties: properties.map(hash_map_to_struct),
            new_name: new_name.unwrap_or_else(|| self.name.clone()),
        };
        self.inner
            .schemas()
            .update_schema(&request)
            .await
            .map(|schema| Buffer::from(schema.encode_to_vec()))
            .default_error()
    }

    #[napi(catch_unwind)]
    pub async fn list_tables(
        &self,
        max_results: Option<i32>,
        include_delta_metadata: Option<bool>,
        omit_columns: Option<bool>,
        omit_properties: Option<bool>,
        omit_username: Option<bool>,
    ) -> napi::Result<Vec<Buffer>> {
        self.inner
            .tables()
            .list(
                &self.catalog_name,
                &self.name,
                max_results,
                include_delta_metadata,
                omit_columns,
                omit_properties,
                omit_username,
            )
            .map_ok(|table| Buffer::from(table.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
}

fn hash_map_to_struct(map: HashMap<String, String>) -> Struct {
    Struct {
        fields: map
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    Value {
                        kind: Some(ValueKind::StringValue(v.clone())),
                    },
                )
            })
            .collect(),
    }
}
