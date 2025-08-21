#![allow(unused_mut)]
use super::client::*;
use crate::error::Result;
use futures::future::BoxFuture;
use std::future::IntoFuture;
use unitycatalog_common::models::sharing::v1::*;
/// Builder for creating requests
pub struct QueryTableBuilder {
    client: SharingClient,
    request: QueryTableRequest,
}
impl QueryTableBuilder {
    /// Create a new builder instance
    pub fn new(
        client: SharingClient,
        share: impl Into<String>,
        schema: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        let request = QueryTableRequest {
            share: share.into(),
            schema: schema.into(),
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    #[doc = concat!("Set ", "starting_timestamp")]
    pub fn with_starting_timestamp(mut self, starting_timestamp: impl Into<String>) -> Self {
        self.request.starting_timestamp = Some(starting_timestamp.into());
        self
    }

    #[doc = concat!("Set ", "limit_hint")]
    pub fn with_limit_hint(mut self, limit_hint: i32) -> Self {
        self.request.limit_hint = Some(limit_hint);
        self
    }
    #[doc = concat!("Set ", "version")]
    pub fn with_version(mut self, version: i64) -> Self {
        self.request.version = Some(version);
        self
    }
    #[doc = concat!("Set ", "timestamp")]
    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.request.timestamp = Some(timestamp.into());
        self
    }
    #[doc = concat!("Set ", "starting_version")]
    pub fn with_starting_version(mut self, starting_version: i64) -> Self {
        self.request.starting_version = Some(starting_version);
        self
    }
    #[doc = concat!("Set ", "ending_version")]
    pub fn with_ending_version(mut self, ending_version: i64) -> Self {
        self.request.ending_version = Some(ending_version);
        self
    }
}
impl IntoFuture for QueryTableBuilder {
    type Output = Result<QueryResponse>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.query_table(&request).await })
    }
}
