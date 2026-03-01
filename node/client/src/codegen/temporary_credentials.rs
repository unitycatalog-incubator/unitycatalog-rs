#![allow(unused_mut, unused_imports, dead_code, clippy::all)]
use crate::error::NapiErrorExt;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use prost::Message;
use std::collections::HashMap;
use unitycatalog_client::TemporaryCredentialClient;
use unitycatalog_common::models::temporary_credentials::v1::*;
#[napi]
pub struct NapiTemporaryCredentialClient {
    pub(crate) client: TemporaryCredentialClient,
}
#[napi]
impl NapiTemporaryCredentialClient {}
impl NapiTemporaryCredentialClient {
    pub fn new(client: TemporaryCredentialClient) -> Self {
        Self { client }
    }
}
