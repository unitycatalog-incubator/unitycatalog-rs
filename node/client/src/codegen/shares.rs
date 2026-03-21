// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports, dead_code, clippy::all)]
use crate::error::NapiErrorExt;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use prost::Message;
use std::collections::HashMap;
use unitycatalog_client::ShareClient;
use unitycatalog_common::models::shares::v1::*;
#[napi]
pub struct NapiShareClient {
    pub(crate) client: ShareClient,
}
#[napi]
impl NapiShareClient {
    #[napi(catch_unwind)]
    pub async fn get(&self, include_shared_data: Option<bool>) -> napi::Result<Buffer> {
        let mut request = self.client.get();
        request = request.with_include_shared_data(include_shared_data);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn update(
        &self,
        new_name: Option<String>,
        owner: Option<String>,
        comment: Option<String>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.update();
        request = request.with_new_name(new_name);
        request = request.with_owner(owner);
        request = request.with_comment(comment);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn delete(&self) -> napi::Result<()> {
        let mut request = self.client.delete();
        request.await.default_error()
    }
}
impl NapiShareClient {
    pub fn new(client: ShareClient) -> Self {
        Self { client }
    }
}
