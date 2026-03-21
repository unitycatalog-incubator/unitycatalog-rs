// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports, dead_code, clippy::all)]
pub mod catalogs;
pub mod credentials;
pub mod external_locations;
pub mod recipients;
pub mod schemas;
pub mod shares;
pub mod tables;
pub mod temporary_credentials;
pub mod volumes;
use crate::codegen::catalogs::NapiCatalogClient;
use crate::codegen::credentials::NapiCredentialClient;
use crate::codegen::external_locations::NapiExternalLocationClient;
use crate::codegen::recipients::NapiRecipientClient;
use crate::codegen::schemas::NapiSchemaClient;
use crate::codegen::shares::NapiShareClient;
use crate::codegen::tables::NapiTableClient;
use crate::codegen::temporary_credentials::NapiTemporaryCredentialClient;
use crate::codegen::volumes::NapiVolumeClient;
use crate::error::NapiErrorExt;
use futures::stream::TryStreamExt;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use prost::Message;
use std::collections::HashMap;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::models::catalogs::v1::*;
use unitycatalog_common::models::credentials::v1::*;
use unitycatalog_common::models::external_locations::v1::*;
use unitycatalog_common::models::recipients::v1::*;
use unitycatalog_common::models::schemas::v1::*;
use unitycatalog_common::models::shares::v1::*;
use unitycatalog_common::models::tables::v1::*;
use unitycatalog_common::models::temporary_credentials::v1::*;
use unitycatalog_common::models::volumes::v1::*;
#[napi]
pub struct NapiUnityCatalogClient {
    client: UnityCatalogClient,
}
#[napi]
impl NapiUnityCatalogClient {
    #[napi(factory)]
    pub fn from_url(base_url: String, token: Option<String>) -> napi::Result<Self> {
        let client = if let Some(token) = token {
            cloud_client::CloudClient::new_with_token(token)
        } else {
            cloud_client::CloudClient::new_unauthenticated()
        };
        let base_url = base_url
            .parse()
            .map_err(|e: url::ParseError| napi::Error::from_reason(e.to_string()))?;
        Ok(Self {
            client: UnityCatalogClient::new(client, base_url),
        })
    }
    #[napi(catch_unwind)]
    pub async fn list_catalogs(&self, max_results: Option<i32>) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_catalogs();
        request = request.with_max_results(max_results);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn create_catalog(
        &self,
        name: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        storage_root: Option<String>,
        provider_name: Option<String>,
        share_name: Option<String>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_catalog(name);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_storage_root(storage_root);
        request = request.with_provider_name(provider_name);
        request = request.with_share_name(share_name);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_credentials(
        &self,
        purpose: Option<i32>,
        max_results: Option<i32>,
    ) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_credentials();
        request = request.with_purpose(purpose.map(|v| v.try_into().ok()).flatten());
        request = request.with_max_results(max_results);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn create_credential(
        &self,
        name: String,
        purpose: i32,
        comment: Option<String>,
        read_only: Option<bool>,
        skip_validation: Option<bool>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_credential(
            name,
            purpose
                .try_into()
                .map_err(|_| napi::Error::from_reason("invalid enum value"))?,
        );
        request = request.with_comment(comment);
        request = request.with_read_only(read_only);
        request = request.with_skip_validation(skip_validation);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_external_locations(
        &self,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_external_locations();
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn create_external_location(
        &self,
        name: String,
        url: String,
        credential_name: String,
        read_only: Option<bool>,
        comment: Option<String>,
        skip_validation: Option<bool>,
    ) -> napi::Result<Buffer> {
        let mut request = self
            .client
            .create_external_location(name, url, credential_name);
        request = request.with_read_only(read_only);
        request = request.with_comment(comment);
        request = request.with_skip_validation(skip_validation);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_recipients(&self, max_results: Option<i32>) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_recipients();
        request = request.with_max_results(max_results);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn create_recipient(
        &self,
        name: String,
        authentication_type: i32,
        owner: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        expiration_time: Option<i64>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_recipient(
            name,
            authentication_type
                .try_into()
                .map_err(|_| napi::Error::from_reason("invalid enum value"))?,
            owner,
        );
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_expiration_time(expiration_time);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_schemas(
        &self,
        catalog_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_schemas(catalog_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn create_schema(
        &self,
        name: String,
        catalog_name: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_schema(name, catalog_name);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_shares(&self, max_results: Option<i32>) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_shares();
        request = request.with_max_results(max_results);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn create_share(
        &self,
        name: String,
        comment: Option<String>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_share(name);
        request = request.with_comment(comment);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_tables(
        &self,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_delta_metadata: Option<bool>,
        omit_columns: Option<bool>,
        omit_properties: Option<bool>,
        omit_username: Option<bool>,
        include_browse: Option<bool>,
        include_manifest_capabilities: Option<bool>,
    ) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_tables(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_delta_metadata(include_delta_metadata);
        request = request.with_omit_columns(omit_columns);
        request = request.with_omit_properties(omit_properties);
        request = request.with_omit_username(omit_username);
        request = request.with_include_browse(include_browse);
        request = request.with_include_manifest_capabilities(include_manifest_capabilities);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn create_table(
        &self,
        name: String,
        schema_name: String,
        catalog_name: String,
        table_type: i32,
        data_source_format: i32,
        storage_location: Option<String>,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_table(
            name,
            schema_name,
            catalog_name,
            table_type
                .try_into()
                .map_err(|_| napi::Error::from_reason("invalid enum value"))?,
            data_source_format
                .try_into()
                .map_err(|_| napi::Error::from_reason("invalid enum value"))?,
        );
        request = request.with_storage_location(storage_location);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_volumes(
        &self,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_volumes(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn create_volume(
        &self,
        catalog_name: String,
        schema_name: String,
        name: String,
        volume_type: i32,
        storage_location: Option<String>,
        comment: Option<String>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_volume(
            catalog_name,
            schema_name,
            name,
            volume_type
                .try_into()
                .map_err(|_| napi::Error::from_reason("invalid enum value"))?,
        );
        request = request.with_storage_location(storage_location);
        request = request.with_comment(comment);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi]
    pub fn catalog(&self, name: String) -> NapiCatalogClient {
        NapiCatalogClient {
            client: self.client.catalog(name),
        }
    }
    #[napi]
    pub fn credential(&self, name: String) -> NapiCredentialClient {
        NapiCredentialClient {
            client: self.client.credential(name),
        }
    }
    #[napi]
    pub fn external_location(&self, name: String) -> NapiExternalLocationClient {
        NapiExternalLocationClient {
            client: self.client.external_location(name),
        }
    }
    #[napi]
    pub fn recipient(&self, name: String) -> NapiRecipientClient {
        NapiRecipientClient {
            client: self.client.recipient(name),
        }
    }
    #[napi]
    pub fn schema(&self, catalog_name: String, schema_name: String) -> NapiSchemaClient {
        NapiSchemaClient {
            client: self.client.schema(catalog_name, schema_name),
        }
    }
    #[napi]
    pub fn share(&self, name: String) -> NapiShareClient {
        NapiShareClient {
            client: self.client.share(name),
        }
    }
    #[napi]
    pub fn table(&self, name: String) -> NapiTableClient {
        NapiTableClient {
            client: self.client.table(name),
        }
    }
    #[napi]
    pub fn volume(
        &self,
        catalog_name: String,
        schema_name: String,
        volume_name: String,
    ) -> NapiVolumeClient {
        NapiVolumeClient {
            client: self.client.volume(catalog_name, schema_name, volume_name),
        }
    }
}
