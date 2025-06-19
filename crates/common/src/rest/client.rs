use chrono::{DateTime, Utc};
use cloud_client::CloudClient;
use delta_kernel::Version;
use delta_kernel::actions::{Metadata, Protocol};
use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use itertools::Itertools;
use reqwest::IntoUrl;
use serde::Deserialize;

pub use crate::api::catalogs::CatalogClient;
pub use crate::api::credentials::CredentialsClient;
pub use crate::api::external_locations::ExternalLocationsClient;
pub use crate::api::recipients::RecipientsClient;
pub use crate::api::schemas::SchemasClient;
pub use crate::api::shares::SharesClient;
pub use crate::api::tables::TablesClient;
use crate::models::catalogs::v1 as catalog;
use crate::models::credentials::v1 as cred;
use crate::models::credentials::v1::Purpose;
use crate::models::external_locations::v1 as loc;
use crate::models::recipients::v1 as rec;
use crate::models::schemas::v1 as schema;
use crate::models::shares::v1 as share;
use crate::models::sharing::v1 as sharing;
use crate::models::tables::v1 as tbl;
pub use crate::sharing::SharingDiscoveryClient;
use crate::sharing::{MetadataResponse, MetadataResponseData, ProtocolResponseData};
use crate::utils::stream_paginated;
use crate::{Error, Result};

#[derive(Clone)]
pub struct UnityCatalogClient {
    client: CloudClient,
    base_url: url::Url,
}

impl UnityCatalogClient {
    pub fn new(client: CloudClient, base_url: url::Url) -> Self {
        Self { client, base_url }
    }

    pub fn catalogs(&self) -> CatalogClient {
        CatalogClient::new(self.client.clone(), self.base_url.clone())
    }

    pub fn credentials(&self) -> CredentialsClient {
        CredentialsClient::new(self.client.clone(), self.base_url.clone())
    }

    pub fn external_locations(&self) -> ExternalLocationsClient {
        ExternalLocationsClient::new(self.client.clone(), self.base_url.clone())
    }

    pub fn recipients(&self) -> RecipientsClient {
        RecipientsClient::new(self.client.clone(), self.base_url.clone())
    }

    pub fn schemas(&self) -> SchemasClient {
        SchemasClient::new(self.client.clone(), self.base_url.clone())
    }

    pub fn tables(&self) -> TablesClient {
        TablesClient::new(self.client.clone(), self.base_url.clone())
    }

    pub fn shares(&self) -> SharesClient {
        SharesClient::new(self.client.clone(), self.base_url.clone())
    }
}

impl CatalogClient {
    pub fn list(
        &self,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<catalog::CatalogInfo>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |max_results, page_token| async move {
            let request = catalog::ListCatalogsRequest {
                max_results,
                page_token,
            };
            let res = self
                .list_catalogs(&request)
                .await
                .map_err(|e| Error::generic(e.to_string()))?;
            Ok((res.catalogs, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub async fn create(
        &self,
        name: impl Into<String>,
        comment: impl Into<Option<String>>,
    ) -> Result<catalog::CatalogInfo> {
        let request = catalog::CreateCatalogRequest {
            name: name.into(),
            comment: comment.into(),
            ..Default::default()
        };
        self.create_catalog(&request).await
    }

    pub async fn get(&self, name: impl Into<String>) -> Result<catalog::CatalogInfo> {
        let request = catalog::GetCatalogRequest {
            name: name.into(),
            include_browse: None,
        };
        self.get_catalog(&request).await
    }

    pub async fn delete(
        &self,
        name: impl Into<String>,
        force: impl Into<Option<bool>>,
    ) -> Result<()> {
        let request = catalog::DeleteCatalogRequest {
            name: name.into(),
            force: force.into(),
        };
        self.delete_catalog(&request).await
    }
}

impl SchemasClient {
    pub fn list(
        &self,
        catalog_name: impl Into<String>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<schema::SchemaInfo>> {
        let max_results = max_results.into();
        let catalog_name = catalog_name.into();
        stream_paginated(
            (catalog_name, max_results),
            move |(catalog_name, max_results), page_token| async move {
                let request = schema::ListSchemasRequest {
                    catalog_name: catalog_name.clone(),
                    max_results,
                    page_token,
                    include_browse: None,
                };
                let res = self
                    .list_schemas(&request)
                    .await
                    .map_err(|e| Error::generic(e.to_string()))?;
                Ok((
                    res.schemas,
                    (catalog_name, max_results),
                    res.next_page_token,
                ))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub async fn create(
        &self,
        catalog_name: impl Into<String>,
        name: impl Into<String>,
        comment: impl Into<Option<String>>,
    ) -> Result<schema::SchemaInfo> {
        let request = schema::CreateSchemaRequest {
            catalog_name: catalog_name.into(),
            name: name.into(),
            comment: comment.into(),
            ..Default::default()
        };
        self.create_schema(&request).await
    }

    pub async fn get(
        &self,
        catalog_name: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<schema::SchemaInfo> {
        let request = schema::GetSchemaRequest {
            full_name: format!("{}.{}", catalog_name.into(), name.into()),
        };
        self.get_schema(&request).await
    }

    pub async fn delete(
        &self,
        catalog_name: impl Into<String>,
        name: impl Into<String>,
        force: impl Into<Option<bool>>,
    ) -> Result<()> {
        let request = schema::DeleteSchemaRequest {
            full_name: format!("{}.{}", catalog_name.into(), name.into()),
            force: force.into(),
        };
        tracing::info!("deleting schema {}", request.full_name);
        self.delete_schema(&request).await
    }
}

impl TablesClient {
    pub fn list_summaries(
        &self,
        catalog_name: impl Into<String>,
        schema_name_pattern: impl Into<Option<String>>,
        table_name_pattern: impl Into<Option<String>>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<tbl::TableSummary>> {
        let max_results = max_results.into();
        let catalog_name = catalog_name.into();
        let schema_name_pattern = schema_name_pattern.into();
        let table_name_pattern = table_name_pattern.into();
        stream_paginated(
            (
                catalog_name,
                schema_name_pattern,
                table_name_pattern,
                max_results,
            ),
            move |(catalog_name, schema_name_pattern, table_name_pattern, max_results),
                  page_token| async move {
                let request = tbl::ListTableSummariesRequest {
                    catalog_name: catalog_name.clone(),
                    schema_name_pattern: schema_name_pattern.clone(),
                    table_name_pattern: table_name_pattern.clone(),
                    page_token,
                    max_results: None,
                    include_manifest_capabilities: None,
                };
                let res = self
                    .list_table_summaries(&request)
                    .await
                    .map_err(|e| Error::generic(e.to_string()))?;
                Ok((
                    res.tables,
                    (
                        catalog_name,
                        schema_name_pattern,
                        table_name_pattern,
                        max_results,
                    ),
                    res.next_page_token,
                ))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub fn list(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        max_results: impl Into<Option<i32>>,
        include_delta_metadata: impl Into<Option<bool>>,
        omit_columns: impl Into<Option<bool>>,
        omit_properties: impl Into<Option<bool>>,
        omit_username: impl Into<Option<bool>>,
    ) -> BoxStream<'_, Result<tbl::TableInfo>> {
        let max_results = max_results.into();
        let catalog_name = catalog_name.into();
        let schema_name = schema_name.into();
        let include_delta_metadata = include_delta_metadata.into();
        let omit_columns = omit_columns.into();
        let omit_properties = omit_properties.into();
        let omit_username = omit_username.into();
        stream_paginated(
            (catalog_name, schema_name, max_results),
            move |(catalog_name, schema_name, max_results), page_token| async move {
                let request = tbl::ListTablesRequest {
                    catalog_name: catalog_name.clone(),
                    schema_name: schema_name.clone(),
                    include_delta_metadata,
                    omit_columns,
                    omit_properties,
                    omit_username,
                    max_results,
                    page_token,
                    include_browse: None,
                    include_manifest_capabilities: None,
                };
                let res = self
                    .list_tables(&request)
                    .await
                    .map_err(|e| Error::generic(e.to_string()))?;
                Ok((
                    res.tables,
                    (catalog_name, schema_name, max_results),
                    res.next_page_token,
                ))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub async fn create(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        name: impl Into<String>,
        comment: impl Into<Option<String>>,
    ) -> Result<tbl::TableInfo> {
        let request = tbl::CreateTableRequest {
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            name: name.into(),
            comment: comment.into(),
            ..Default::default()
        };
        self.create_table(&request).await
    }

    pub async fn get(
        &self,
        full_name: impl Into<String>,
        include_delta_metadata: impl Into<Option<bool>>,
    ) -> Result<tbl::TableInfo> {
        let request = tbl::GetTableRequest {
            full_name: full_name.into(),
            include_delta_metadata: include_delta_metadata.into(),
            include_browse: None,
            include_manifest_capabilities: None,
        };
        self.get_table(&request).await
    }

    pub async fn delete(&self, full_name: impl Into<String>) -> Result<()> {
        let request = tbl::DeleteTableRequest {
            full_name: full_name.into(),
        };
        self.delete_table(&request).await
    }
}

impl CredentialsClient {
    pub fn list(
        &self,
        purpose: Option<Purpose>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<cred::CredentialInfo>> {
        let max_results = max_results.into();
        let purpose = purpose.map(|p| p as i32);
        stream_paginated(max_results, move |max_results, page_token| async move {
            let request = cred::ListCredentialsRequest {
                max_results,
                page_token,
                purpose,
            };
            let res = self
                .list_credentials(&request)
                .await
                .map_err(|e| Error::generic(e.to_string()))?;
            Ok((res.credentials, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub async fn create(
        &self,
        name: impl Into<String>,
        purpose: Purpose,
        comment: impl Into<Option<String>>,
    ) -> Result<cred::CredentialInfo> {
        let request = cred::CreateCredentialRequest {
            name: name.into(),
            purpose: purpose.into(),
            comment: comment.into(),
            ..Default::default()
        };
        self.create_credential(&request).await
    }

    pub async fn get(&self, name: impl Into<String>) -> Result<cred::CredentialInfo> {
        let request = cred::GetCredentialRequest { name: name.into() };
        self.get_credential(&request).await
    }

    pub async fn delete(&self, name: impl Into<String>) -> Result<()> {
        let request = cred::DeleteCredentialRequest { name: name.into() };
        self.delete_credential(&request).await
    }
}

impl ExternalLocationsClient {
    pub fn list(
        &self,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<loc::ExternalLocationInfo>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |max_results, page_token| async move {
            let request = loc::ListExternalLocationsRequest {
                max_results,
                page_token,
                include_browse: None,
            };
            let res = self
                .list_external_locations(&request)
                .await
                .map_err(|e| Error::generic(e.to_string()))?;
            Ok((res.external_locations, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub async fn create(
        &self,
        name: impl Into<String>,
        url: impl IntoUrl,
        credential_name: impl Into<String>,
        comment: impl Into<Option<String>>,
    ) -> Result<loc::ExternalLocationInfo> {
        let request = loc::CreateExternalLocationRequest {
            name: name.into(),
            url: url
                .into_url()
                .map(|u| u.to_string())
                .map_err(|e| Error::generic(e.to_string()))?,
            credential_name: credential_name.into(),
            comment: comment.into(),
            ..Default::default()
        };
        self.create_external_location(&request).await
    }

    pub async fn get(&self, name: impl Into<String>) -> Result<loc::ExternalLocationInfo> {
        let request = loc::GetExternalLocationRequest { name: name.into() };
        self.get_external_location(&request).await
    }

    pub async fn delete(
        &self,
        name: impl Into<String>,
        force: impl Into<Option<bool>>,
    ) -> Result<()> {
        let request = loc::DeleteExternalLocationRequest {
            name: name.into(),
            force: force.into(),
        };
        self.delete_external_location(&request).await
    }
}

impl RecipientsClient {
    pub fn list(
        &self,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<rec::RecipientInfo>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |max_results, page_token| async move {
            let request = rec::ListRecipientsRequest {
                max_results,
                page_token,
            };
            let res = self
                .list_recipients(&request)
                .await
                .map_err(|e| Error::generic(e.to_string()))?;
            Ok((res.recipients, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub async fn create(
        &self,
        name: impl Into<String>,
        authentication_type: rec::AuthenticationType,
        comment: impl Into<Option<String>>,
    ) -> Result<rec::RecipientInfo> {
        let request = rec::CreateRecipientRequest {
            name: name.into(),
            authentication_type: authentication_type.into(),
            comment: comment.into(),
            ..Default::default()
        };
        self.create_recipient(&request).await
    }

    pub async fn get(&self, name: impl Into<String>) -> Result<rec::RecipientInfo> {
        let request = rec::GetRecipientRequest { name: name.into() };
        self.get_recipient(&request).await
    }

    pub async fn delete(&self, name: impl Into<String>) -> Result<()> {
        let request = rec::DeleteRecipientRequest { name: name.into() };
        self.delete_recipient(&request).await
    }
}

impl SharesClient {
    pub fn list(
        &self,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<share::ShareInfo>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |max_results, page_token| async move {
            let request = share::ListSharesRequest {
                max_results,
                page_token,
            };
            let res = self
                .list_shares(&request)
                .await
                .map_err(|e| Error::generic(e.to_string()))?;
            Ok((res.shares, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub async fn create(
        &self,
        name: impl Into<String>,
        comment: impl Into<Option<String>>,
    ) -> Result<share::ShareInfo> {
        let request = share::CreateShareRequest {
            name: name.into(),
            comment: comment.into(),
        };
        self.create_share(&request).await
    }

    pub async fn get(
        &self,
        name: impl Into<String>,
        include_shared_data: impl Into<Option<bool>>,
    ) -> Result<share::ShareInfo> {
        let request = share::GetShareRequest {
            name: name.into(),
            include_shared_data: include_shared_data.into(),
        };
        self.get_share(&request).await
    }

    pub async fn delete(&self, name: impl Into<String>) -> Result<()> {
        let request = share::DeleteShareRequest { name: name.into() };
        self.delete_share(&request).await
    }

    pub async fn update(
        &self,
        name: impl Into<String>,
        new_name: impl Into<Option<String>>,
        updates: Vec<share::DataObjectUpdate>,
        comment: impl Into<Option<String>>,
        owner: impl Into<Option<String>>,
    ) -> Result<share::ShareInfo> {
        let request = share::UpdateShareRequest {
            name: name.into(),
            new_name: new_name.into().and_then(|s| (!s.is_empty()).then_some(s)),
            comment: comment.into(),
            owner: owner.into(),
            updates,
        };
        self.update_share(&request).await
    }
}

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

    pub fn list_shares(
        &self,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<sharing::Share>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |max_results, page_token| async move {
            let request = sharing::ListSharesRequest {
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

    pub async fn get_share(&self, name: impl Into<String>) -> Result<sharing::Share> {
        let request = sharing::GetShareRequest { name: name.into() };
        self.discovery.get_share(&request).await
    }

    async fn list_share_schemas_inner(
        &self,
        request: sharing::ListSharingSchemasRequest,
    ) -> Result<sharing::ListSharingSchemasResponse> {
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
    ) -> BoxStream<'_, Result<sharing::SharingSchema>> {
        let share = share.into();
        let max_results = max_results.into();
        stream_paginated(
            (share, max_results),
            move |(share, max_results), page_token| async move {
                let request = sharing::ListSharingSchemasRequest {
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
        request: sharing::ListShareTablesRequest,
    ) -> Result<sharing::ListShareTablesResponse> {
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
    ) -> BoxStream<'_, Result<sharing::SharingTable>> {
        let share = share.into();
        let max_results = max_results.into();
        stream_paginated(
            (share, max_results),
            move |(share, max_results), page_token| async move {
                let request = sharing::ListShareTablesRequest {
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
        request: sharing::ListSchemaTablesRequest,
    ) -> Result<sharing::ListSchemaTablesResponse> {
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
    ) -> BoxStream<'_, Result<sharing::SharingTable>> {
        let share = share.into();
        let schema = schema.into();
        let max_results = max_results.into();
        stream_paginated(
            (share, schema, max_results),
            move |(share, schema, max_results), page_token| async move {
                let request = sharing::ListSchemaTablesRequest {
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
            .map(|line| ::serde_json::from_slice(line))
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaFileResponse {
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
