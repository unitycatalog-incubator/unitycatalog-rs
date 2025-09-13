#![allow(unused_mut)]
use futures::future::BoxFuture;
use std::future::IntoFuture;

use crate::models::sharing::v1::*;

use super::client::*;
use crate::error::Result;

/// Builder for creating requests
pub struct GetShareBuilder {
    client: SharingClient,
    request: GetShareRequest,
}

impl GetShareBuilder {
    /// Create a new builder instance
    pub(crate) fn new(client: SharingClient, name: impl Into<String>) -> Self {
        let request = GetShareRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}

impl IntoFuture for GetShareBuilder {
    type Output = Result<Share>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_share(&request).await })
    }
}

/// Builder for creating requests
pub struct GetTableVersionBuilder {
    client: SharingClient,
    request: GetTableVersionRequest,
}

impl GetTableVersionBuilder {
    /// Create a new builder instance
    pub(crate) fn new(
        client: SharingClient,
        name: impl Into<String>,
        schema: impl Into<String>,
        share: impl Into<String>,
    ) -> Self {
        let request = GetTableVersionRequest {
            name: name.into(),
            schema: schema.into(),
            share: share.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /**The startingTimestamp of the query, a string in the  ISO8601 format, in the UTC timezone,
    such as 2022-01-01T00:00:00Z. the server needs to return the earliest table version at
    or after the provided timestamp, can be earlier than the timestamp of table version 0.*/
    pub fn with_starting_timestamp(
        mut self,
        starting_timestamp: impl Into<Option<String>>,
    ) -> Self {
        self.request.starting_timestamp = starting_timestamp.into();
        self
    }
}

impl IntoFuture for GetTableVersionBuilder {
    type Output = Result<GetTableVersionResponse>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_table_version(&request).await })
    }
}

/// Builder for creating requests
pub struct GetTableMetadataBuilder {
    client: SharingClient,
    request: GetTableMetadataRequest,
}

impl GetTableMetadataBuilder {
    /// Create a new builder instance
    pub(crate) fn new(
        client: SharingClient,
        name: impl Into<String>,
        share: impl Into<String>,
        schema: impl Into<String>,
    ) -> Self {
        let request = GetTableMetadataRequest {
            name: name.into(),
            share: share.into(),
            schema: schema.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}

impl IntoFuture for GetTableMetadataBuilder {
    type Output = Result<QueryResponse>;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_table_metadata(&request).await })
    }
}

/// Builder for creating requests
pub struct QueryTableBuilder {
    client: SharingClient,
    request: QueryTableRequest,
}

impl QueryTableBuilder {
    /// Create a new builder instance
    pub(crate) fn new(
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
    ///The starting timestamp to query from.
    pub fn with_starting_timestamp(
        mut self,
        starting_timestamp: impl Into<Option<String>>,
    ) -> Self {
        self.request.starting_timestamp = starting_timestamp.into();
        self
    }
    #[doc = concat!("Set ", "predicate_hints")]
    pub fn with_predicate_hints<I>(mut self, predicate_hints: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        self.request.predicate_hints = predicate_hints.into_iter().collect();
        self
    }
    ///The predicate to apply to the table.
    pub fn with_json_predicate_hints(mut self, json_predicate_hints: JsonPredicate) -> Self {
        self.request.json_predicate_hints = Some(json_predicate_hints);
        self
    }
    #[doc = concat!("Set ", "limit_hint")]
    pub fn with_limit_hint(mut self, limit_hint: impl Into<Option<i32>>) -> Self {
        self.request.limit_hint = limit_hint.into();
        self
    }
    #[doc = concat!("Set ", "version")]
    pub fn with_version(mut self, version: impl Into<Option<i64>>) -> Self {
        self.request.version = version.into();
        self
    }
    #[doc = concat!("Set ", "timestamp")]
    pub fn with_timestamp(mut self, timestamp: impl Into<Option<String>>) -> Self {
        self.request.timestamp = timestamp.into();
        self
    }
    #[doc = concat!("Set ", "starting_version")]
    pub fn with_starting_version(mut self, starting_version: impl Into<Option<i64>>) -> Self {
        self.request.starting_version = starting_version.into();
        self
    }
    #[doc = concat!("Set ", "ending_version")]
    pub fn with_ending_version(mut self, ending_version: impl Into<Option<i64>>) -> Self {
        self.request.ending_version = ending_version.into();
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
