use chrono::{DateTime, Utc};
use cloud_client::CloudClient;
use delta_kernel::Version;
use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use itertools::Itertools;

use super::SharingDiscoveryClient;
use super::models::*;
use crate::models::sharing::v1::*;
use crate::utils::stream_paginated;
use crate::{Error, Result};

#[derive(Clone)]
pub struct SharingClient {
    client: CloudClient,
    base_url: url::Url,
    discovery: SharingDiscoveryClient,
}

impl SharingClient {
    pub fn new(client: CloudClient, base_url: url::Url) -> Self {
        let base_url = base_url.join("api/v1/delta-sharing/").unwrap();
        Self {
            discovery: SharingDiscoveryClient::new(client.clone(), base_url.clone()),
            client,
            base_url,
        }
    }

    pub fn new_with_prefix(
        client: CloudClient,
        base_url: url::Url,
        prefix: impl Into<String>,
    ) -> Self {
        let prefix = prefix.into();
        let base_url = if !prefix.ends_with('/') {
            base_url.join(&format!("{}/", prefix)).unwrap()
        } else {
            base_url.join(&prefix).unwrap()
        };
        Self {
            discovery: SharingDiscoveryClient::new(client.clone(), base_url.clone()),
            client,
            base_url,
        }
    }

    pub fn list_shares(&self, max_results: impl Into<Option<i32>>) -> BoxStream<'_, Result<Share>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |max_results, page_token| async move {
            let request = ListSharesRequest {
                max_results,
                page_token,
            };
            let res = self
                .discovery
                .list_shares(&request)
                .await
                .map_err(|e| Error::generic(e.to_string()))?;
            Ok((res.items, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub async fn get_share(&self, name: impl Into<String>) -> Result<Share> {
        let request = GetShareRequest { name: name.into() };
        self.discovery.get_share(&request).await
    }

    async fn list_share_schemas_inner(
        &self,
        request: ListSharingSchemasRequest,
    ) -> Result<ListSharingSchemasResponse> {
        let mut url = self
            .base_url
            .join(&format!("shares/{}/schemas", request.share))?;
        if let Some(page_token) = request.page_token {
            url.query_pairs_mut().append_pair("page_token", &page_token);
        }
        if let Some(max_results) = request.max_results {
            url.query_pairs_mut()
                .append_pair("max_results", &max_results.to_string());
        }
        let result = self.client.get(url).send().await?;
        result.error_for_status_ref()?;
        let result = result.bytes().await?;
        Ok(::serde_json::from_slice(&result)?)
    }

    pub fn list_share_schemas(
        &self,
        share: impl Into<String>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<SharingSchema>> {
        let share = share.into();
        let max_results = max_results.into();
        stream_paginated(
            (share, max_results),
            move |(share, max_results), page_token| async move {
                let request = ListSharingSchemasRequest {
                    share: share.clone(),
                    max_results,
                    page_token,
                };
                let res = self
                    .list_share_schemas_inner(request)
                    .await
                    .map_err(|e| Error::generic(e.to_string()))?;
                Ok((res.items, (share, max_results), res.next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    async fn list_share_tables_inner(
        &self,
        request: ListShareTablesRequest,
    ) -> Result<ListShareTablesResponse> {
        let mut url = self
            .base_url
            .join(&format!("shares/{}/all-tables", request.name))?;
        if let Some(page_token) = request.page_token {
            url.query_pairs_mut().append_pair("page_token", &page_token);
        }
        if let Some(max_results) = request.max_results {
            url.query_pairs_mut()
                .append_pair("max_results", &max_results.to_string());
        }
        let result = self.client.get(url).send().await?;
        result.error_for_status_ref()?;
        let result = result.bytes().await?;
        Ok(::serde_json::from_slice(&result)?)
    }

    pub fn list_share_tables(
        &self,
        share: impl Into<String>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<SharingTable>> {
        let share = share.into();
        let max_results = max_results.into();
        stream_paginated(
            (share, max_results),
            move |(share, max_results), page_token| async move {
                let request = ListShareTablesRequest {
                    name: share.clone(),
                    max_results,
                    page_token,
                };
                let res = self
                    .list_share_tables_inner(request)
                    .await
                    .map_err(|e| Error::generic(e.to_string()))?;
                Ok((res.items, (share, max_results), res.next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    async fn list_schema_tables_inner(
        &self,
        request: ListSchemaTablesRequest,
    ) -> Result<ListSchemaTablesResponse> {
        let mut url = self.base_url.join(&format!(
            "shares/{}/schemas/{}/tables",
            request.share, request.name
        ))?;
        if let Some(page_token) = request.page_token {
            url.query_pairs_mut().append_pair("page_token", &page_token);
        }
        if let Some(max_results) = request.max_results {
            url.query_pairs_mut()
                .append_pair("max_results", &max_results.to_string());
        }
        let result = self.client.get(url).send().await?;
        result.error_for_status_ref()?;
        let result = result.bytes().await?;
        Ok(::serde_json::from_slice(&result)?)
    }

    pub fn list_schema_tables(
        &self,
        share: impl Into<String>,
        schema: impl Into<String>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<SharingTable>> {
        let share = share.into();
        let schema = schema.into();
        let max_results = max_results.into();
        stream_paginated(
            (share, schema, max_results),
            move |(share, schema, max_results), page_token| async move {
                let request = ListSchemaTablesRequest {
                    share: share.clone(),
                    name: schema.clone(),
                    max_results,
                    page_token,
                };
                let res = self
                    .list_schema_tables_inner(request)
                    .await
                    .map_err(|e| Error::generic(e.to_string()))?;
                Ok((res.items, (share, schema, max_results), res.next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub async fn get_table_version(
        &self,
        share: impl Into<String>,
        schema: impl Into<String>,
        table: impl Into<String>,
        starting_timestamp: Option<DateTime<Utc>>,
    ) -> Result<Version> {
        let mut url = self.base_url.join(&format!(
            "shares/{}/schemas/{}/tables/{}/version",
            share.into(),
            schema.into(),
            table.into()
        ))?;
        if let Some(ts) = starting_timestamp {
            url.query_pairs_mut()
                .append_pair("startingTimestamp", &ts.to_rfc3339());
        }
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let headers = response.headers();
        let version = headers
            .get("Delta-Table-Version")
            .ok_or(Error::generic("Delta-Table-Version header not found"))?;
        let version = version
            .to_str()
            .ok()
            .and_then(|v| v.parse::<Version>().ok())
            .ok_or(Error::generic("Invalid version header"))?;
        Ok(version)
    }

    pub async fn get_table_metadata(
        &self,
        share: impl Into<String>,
        schema: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<(ProtocolResponseData, MetadataResponseData)> {
        let url = self.base_url.join(&format!(
            "shares/{}/schemas/{}/tables/{}/metadata",
            share.into(),
            schema.into(),
            name.into()
        ))?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        // split newlines and parse each as json
        let line_data: Vec<MetadataResponse> = result
            .split(|c| *c == b'\n')
            .map(::serde_json::from_slice)
            .try_collect()?;
        let mut protocol = None;
        let mut metadata = None;
        for item in line_data {
            match item {
                MetadataResponse::Protocol(p) => protocol = Some(p),
                MetadataResponse::MetaData(m) => metadata = Some(m),
            }
        }
        if protocol.is_none() {
            return Err(Error::generic("Protocol not found"));
        }
        if metadata.is_none() {
            return Err(Error::generic("Metadata not found"));
        }
        Ok((protocol.unwrap(), metadata.unwrap()))
    }
}
