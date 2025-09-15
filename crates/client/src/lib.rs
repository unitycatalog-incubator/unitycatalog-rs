use cloud_client::CloudClient;

pub use catalogs::*;
pub use credentials::*;
pub use error::*;
pub use external_locations::*;
use futures::stream::BoxStream;
pub use recipients::*;
pub use schemas::*;
pub use shares::*;
pub use tables::*;
pub use temporary_credentials::*;
use unitycatalog_common::models::volumes::v1::VolumeType;
use unitycatalog_common::tables::v1::{DataSourceFormat, TableType};
use unitycatalog_common::{
    credentials::v1::Purpose, recipients::v1::AuthenticationType, tables::v1::TableSummary,
};
pub use volumes::*;

// Re-export all builders for public API
use crate::codegen::catalogs::ListCatalogsBuilder;
pub use crate::codegen::catalogs::builders::{CreateCatalogBuilder, UpdateCatalogBuilder};
use crate::codegen::credentials::ListCredentialsBuilder;
pub use crate::codegen::credentials::builders::{CreateCredentialBuilder, UpdateCredentialBuilder};
use crate::codegen::external_locations::ListExternalLocationsBuilder;
pub use crate::codegen::external_locations::builders::{
    CreateExternalLocationBuilder, UpdateExternalLocationBuilder,
};
use crate::codegen::recipients::ListRecipientsBuilder;
pub use crate::codegen::recipients::builders::{CreateRecipientBuilder, UpdateRecipientBuilder};
use crate::codegen::schemas::ListSchemasBuilder;
pub use crate::codegen::schemas::builders::{CreateSchemaBuilder, UpdateSchemaBuilder};
use crate::codegen::shares::ListSharesBuilder;
pub use crate::codegen::shares::builders::{CreateShareBuilder, UpdateShareBuilder};
use crate::codegen::tables::ListTablesBuilder;
pub use crate::codegen::tables::builders::CreateTableBuilder;
pub use crate::codegen::temporary_credentials::builders::{
    GenerateTemporaryPathCredentialsBuilder, GenerateTemporaryTableCredentialsBuilder,
};
use crate::codegen::volumes::ListVolumesBuilder;
pub use crate::codegen::volumes::builders::{CreateVolumeBuilder, UpdateVolumeBuilder};

mod catalogs;
mod codegen;
mod credentials;
pub mod error;
mod external_locations;
mod recipients;
mod schemas;
mod shares;
mod tables;
mod temporary_credentials;
mod utils;
mod volumes;

#[derive(Clone)]
pub struct UnityCatalogClient {
    catalogs: CatalogClientBase,
    schemas: SchemaClientBase,
    tables: TableClientBase,
    shares: ShareClientBase,
    recipients: RecipientClientBase,
    credentials: CredentialClientBase,
    external_locations: ExternalLocationClientBase,
    temporary_credentials: TemporaryCredentialClientBase,
    volumes: VolumeClientBase,
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
        let temporary_credentials =
            TemporaryCredentialClientBase::new(client.clone(), base_url.clone());
        let volumes = VolumeClientBase::new(client.clone(), base_url.clone());

        Self {
            catalogs,
            schemas,
            tables,
            shares,
            recipients,
            credentials,
            external_locations,
            temporary_credentials,
            volumes,
        }
    }

    // Catalog methods
    pub fn list_catalogs(&self) -> ListCatalogsBuilder {
        ListCatalogsBuilder::new(self.catalogs.clone())
    }

    pub fn create_catalog(&self, name: impl ToString) -> CreateCatalogBuilder {
        CreateCatalogBuilder::new(self.catalogs.clone(), name.to_string())
    }

    pub fn catalog(&self, name: impl ToString) -> CatalogClient {
        CatalogClient::new(name, self.catalogs.clone())
    }

    // Credential methods
    pub fn list_credentials(&self) -> ListCredentialsBuilder {
        ListCredentialsBuilder::new(self.credentials.clone())
    }

    pub fn create_credential(
        &self,
        name: impl ToString,
        purpose: Purpose,
    ) -> CreateCredentialBuilder {
        let credential = CredentialClient::new(name, self.credentials.clone());
        credential.create(purpose)
    }

    pub fn credential(&self, name: impl ToString) -> CredentialClient {
        CredentialClient::new(name, self.credentials.clone())
    }

    // Schema methods
    pub fn list_schemas(&self, catalog_name: impl Into<String>) -> ListSchemasBuilder {
        ListSchemasBuilder::new(self.schemas.clone(), catalog_name)
    }

    pub fn create_schema(
        &self,
        catalog_name: impl ToString,
        schema_name: impl ToString,
    ) -> CreateSchemaBuilder {
        let schema = SchemaClient::new(catalog_name, schema_name, self.schemas.clone());
        schema.create()
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
    ) -> BoxStream<'_, Result<TableSummary>> {
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
    ) -> ListTablesBuilder {
        ListTablesBuilder::new(self.tables.clone(), catalog_name, schema_name)
    }

    pub fn create_table(
        &self,
        name: impl ToString,
        schema_name: impl ToString,
        catalog_name: impl ToString,
        table_type: TableType,
        data_source_format: DataSourceFormat,
    ) -> CreateTableBuilder {
        CreateTableBuilder::new(
            self.tables.clone(),
            name.to_string(),
            schema_name.to_string(),
            catalog_name.to_string(),
            table_type,
            data_source_format,
        )
    }

    pub fn table(&self, full_name: impl ToString) -> TableClient {
        TableClient::new(full_name, self.tables.clone())
    }

    pub fn create_share(&self, name: impl ToString) -> CreateShareBuilder {
        let share = ShareClient::new(name, self.shares.clone());
        share.create()
    }

    // Share methods
    pub fn list_shares(&self) -> ListSharesBuilder {
        ListSharesBuilder::new(self.shares.clone())
    }

    pub fn share(&self, name: impl ToString) -> ShareClient {
        ShareClient::new(name, self.shares.clone())
    }

    pub fn create_recipient(
        &self,
        name: impl ToString,
        authentication_type: AuthenticationType,
        owner: impl Into<String>,
    ) -> CreateRecipientBuilder {
        let recipient = RecipientClient::new(name, self.recipients.clone());
        recipient.create(authentication_type, owner)
    }

    // Recipient methods
    pub fn list_recipients(&self) -> ListRecipientsBuilder {
        ListRecipientsBuilder::new(self.recipients.clone())
    }

    pub fn recipient(&self, name: impl ToString) -> RecipientClient {
        RecipientClient::new(name, self.recipients.clone())
    }

    // External location methods
    pub fn list_external_locations(&self) -> ListExternalLocationsBuilder {
        ListExternalLocationsBuilder::new(self.external_locations.clone())
    }

    pub fn create_external_location(
        &self,
        name: impl ToString,
        url: impl ToString,
        credential_name: impl ToString,
    ) -> CreateExternalLocationBuilder {
        let external_location = ExternalLocationClient::new(name, self.external_locations.clone());
        external_location.create(url, credential_name)
    }

    pub fn external_location(&self, name: impl ToString) -> ExternalLocationClient {
        ExternalLocationClient::new(name, self.external_locations.clone())
    }

    pub fn temporary_credentials(&self) -> TemporaryCredentialClient {
        TemporaryCredentialClient::new(self.temporary_credentials.clone())
    }

    // Volume methods
    pub fn list_volumes(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
    ) -> ListVolumesBuilder {
        ListVolumesBuilder::new(self.volumes.clone(), catalog_name, schema_name)
    }

    pub fn create_volume(
        &self,
        catalog_name: impl ToString,
        schema_name: impl ToString,
        volume_name: impl ToString,
        volume_type: VolumeType,
    ) -> CreateVolumeBuilder {
        let volume =
            VolumeClient::new(catalog_name, schema_name, volume_name, self.volumes.clone());
        volume.create(volume_type)
    }

    pub fn volume(
        &self,
        catalog_name: impl ToString,
        schema_name: impl ToString,
        volume_name: impl ToString,
    ) -> VolumeClient {
        VolumeClient::new(catalog_name, schema_name, volume_name, self.volumes.clone())
    }

    pub fn volume_from_full_name(&self, full_name: impl ToString) -> VolumeClient {
        VolumeClient::new_from_full_name(full_name, self.volumes.clone())
    }
}
