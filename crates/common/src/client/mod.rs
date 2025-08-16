use cloud_client::CloudClient;

pub use catalogs::*;
pub use credentials::*;
pub use external_locations::*;
use futures::stream::BoxStream;
pub use recipients::*;
pub use schemas::*;
pub use shares::*;
pub use sharing::*;
pub use tables::*;
pub use temporary_credentials::*;

use crate::{CatalogInfo, Result};

mod catalogs;
mod credentials;
mod external_locations;
mod recipients;
mod schemas;
mod shares;
mod sharing;
mod tables;
mod temporary_credentials;
mod utils;

#[derive(Clone)]
pub struct UnityCatalogClient {
    catalogs: CatalogClientBase,
    schemas: SchemaClientBase,
    tables: TableClientBase,
    shares: ShareClientBase,
    recipients: RecipientClientBase,
    credentials: CredentialClientBase,
    external_locations: ExternalLocationClientBase,
}

impl UnityCatalogClient {
    pub fn new_unauthenticated(base_url: url::Url) -> Self {
        let client = CloudClient::new_unauthenticated();
        Self::new(client, base_url)
    }

    pub fn new_with_token(base_url: url::Url, token: impl ToString) -> Self {
        let client = CloudClient::new_with_token(token);
        Self::new(client, base_url)
    }

    pub fn new(client: CloudClient, mut base_url: url::Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        let catalogs = CatalogClientBase::new(client.clone(), base_url.clone());
        let schemas = SchemaClientBase::new(client.clone(), base_url.clone());
        let tables = TableClientBase::new(client.clone(), base_url.clone());
        let shares = ShareClientBase::new(client.clone(), base_url.clone());
        let recipients = RecipientClientBase::new(client.clone(), base_url.clone());
        let credentials = CredentialClientBase::new(client.clone(), base_url.clone());
        let external_locations = ExternalLocationClientBase::new(client.clone(), base_url.clone());

        Self {
            catalogs,
            schemas,
            tables,
            shares,
            recipients,
            credentials,
            external_locations,
        }
    }

    // Catalog methods
    pub fn list_catalogs(
        &self,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<CatalogInfo>> {
        self.catalogs.list(max_results)
    }

    pub fn catalog(&self, name: impl ToString) -> CatalogClient {
        CatalogClient::new(name, self.catalogs.clone())
    }

    // Schema methods
    pub fn list_schemas(
        &self,
        catalog_name: impl Into<String>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<crate::models::schemas::v1::SchemaInfo>> {
        self.schemas.list(catalog_name, max_results)
    }

    pub fn schema(&self, catalog_name: impl ToString, schema_name: impl ToString) -> SchemaClient {
        SchemaClient::new(catalog_name, schema_name, self.schemas.clone())
    }

    // Table methods
    pub fn list_table_summaries(
        &self,
        catalog_name: impl Into<String>,
        schema_name_pattern: Option<impl ToString>,
        table_name_pattern: Option<impl ToString>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<crate::models::tables::v1::TableSummary>> {
        self.tables.list_summaries(
            catalog_name,
            schema_name_pattern,
            table_name_pattern,
            max_results,
        )
    }

    pub fn list_tables(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        max_results: impl Into<Option<i32>>,
        include_delta_metadata: impl Into<Option<bool>>,
        omit_columns: impl Into<Option<bool>>,
        omit_properties: impl Into<Option<bool>>,
        omit_username: impl Into<Option<bool>>,
    ) -> BoxStream<'_, Result<crate::models::tables::v1::TableInfo>> {
        self.tables.list(
            catalog_name,
            schema_name,
            max_results,
            include_delta_metadata,
            omit_columns,
            omit_properties,
            omit_username,
        )
    }

    pub fn table(&self, full_name: impl ToString) -> TableClient {
        TableClient::new(full_name, self.tables.clone())
    }

    // Share methods
    pub fn list_shares(
        &self,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<crate::models::shares::v1::ShareInfo>> {
        self.shares.list(max_results)
    }

    pub fn share(&self, name: impl ToString) -> ShareClient {
        ShareClient::new(name, self.shares.clone())
    }

    // Recipient methods
    pub fn list_recipients(
        &self,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<crate::models::recipients::v1::RecipientInfo>> {
        self.recipients.list(max_results)
    }

    pub fn recipient(&self, name: impl ToString) -> RecipientClient {
        RecipientClient::new(name, self.recipients.clone())
    }

    // Credential methods
    pub fn list_credentials(
        &self,
        purpose: Option<crate::models::credentials::v1::Purpose>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<crate::models::credentials::v1::CredentialInfo>> {
        self.credentials.list(purpose, max_results)
    }

    pub fn credential(&self, name: impl ToString) -> CredentialClient {
        CredentialClient::new(name, self.credentials.clone())
    }

    // External location methods
    pub fn list_external_locations(
        &self,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<crate::models::external_locations::v1::ExternalLocationInfo>> {
        self.external_locations.list(max_results)
    }

    pub fn external_location(&self, name: impl ToString) -> ExternalLocationClient {
        ExternalLocationClient::new(name, self.external_locations.clone())
    }

    // Create methods
    pub async fn create_catalog(
        &self,
        name: impl ToString,
        storage_root: Option<impl ToString>,
        comment: Option<impl ToString>,
        properties: impl Into<Option<std::collections::HashMap<String, String>>>,
    ) -> Result<CatalogInfo> {
        let catalog = CatalogClient::new(name, self.catalogs.clone());
        catalog.create(storage_root, comment, properties).await
    }

    pub async fn create_sharing_catalog(
        &self,
        name: impl ToString,
        provider_name: impl Into<String>,
        share_name: impl Into<String>,
        comment: Option<impl ToString>,
        properties: impl Into<Option<std::collections::HashMap<String, String>>>,
    ) -> Result<CatalogInfo> {
        let catalog = CatalogClient::new(name, self.catalogs.clone());
        catalog
            .create_sharing(provider_name, share_name, comment, properties)
            .await
    }

    pub async fn create_schema(
        &self,
        catalog_name: impl ToString,
        schema_name: impl ToString,
        comment: Option<impl ToString>,
    ) -> Result<crate::models::schemas::v1::SchemaInfo> {
        let schema = SchemaClient::new(catalog_name, schema_name, self.schemas.clone());
        schema.create(comment).await
    }

    pub async fn create_share(
        &self,
        name: impl ToString,
        comment: Option<impl ToString>,
    ) -> Result<crate::models::shares::v1::ShareInfo> {
        let share = ShareClient::new(name, self.shares.clone());
        share.create(comment).await
    }

    pub async fn create_recipient(
        &self,
        name: impl ToString,
        authentication_type: crate::models::recipients::v1::AuthenticationType,
        comment: Option<impl ToString>,
    ) -> Result<crate::models::recipients::v1::RecipientInfo> {
        let recipient = RecipientClient::new(name, self.recipients.clone());
        recipient.create(authentication_type, comment).await
    }

    pub async fn create_credential(
        &self,
        name: impl ToString,
        purpose: crate::models::credentials::v1::Purpose,
        comment: Option<impl ToString>,
    ) -> Result<crate::models::credentials::v1::CredentialInfo> {
        let credential = CredentialClient::new(name, self.credentials.clone());
        credential.create(purpose, comment).await
    }

    pub async fn create_external_location(
        &self,
        name: impl ToString,
        url: impl reqwest::IntoUrl,
        credential_name: impl Into<String>,
        comment: Option<impl ToString>,
    ) -> Result<crate::models::external_locations::v1::ExternalLocationInfo> {
        let external_location = ExternalLocationClient::new(name, self.external_locations.clone());
        external_location
            .create(url, credential_name, comment)
            .await
    }
}
