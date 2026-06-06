// @generated — do not edit by hand.
use crate::CatalogClient;
use crate::CredentialClient;
use crate::ExternalLocationClient;
use crate::FunctionClient;
use crate::ProviderClient;
use crate::RecipientClient;
use crate::SchemaClient;
use crate::ShareClient;
use crate::TableClient;
use crate::TagPolicyClient;
use crate::VolumeClient;
use crate::codegen::catalogs::*;
use crate::codegen::credentials::*;
use crate::codegen::delta_commits::*;
use crate::codegen::entity_tag_assignments::*;
use crate::codegen::external_locations::*;
use crate::codegen::functions::*;
use crate::codegen::providers::*;
use crate::codegen::recipients::*;
use crate::codegen::schemas::*;
use crate::codegen::shares::*;
use crate::codegen::tables::*;
use crate::codegen::tag_policies::*;
use crate::codegen::temporary_credentials::*;
use crate::codegen::volumes::*;
use olai_http::CloudClient;
use unitycatalog_common::models::credentials::v1::*;
use unitycatalog_common::models::functions::v1::*;
use unitycatalog_common::models::providers::v1::*;
use unitycatalog_common::models::recipients::v1::*;
use unitycatalog_common::models::tables::v1::*;
use unitycatalog_common::models::tags::v1::*;
use unitycatalog_common::models::temporary_credentials::v1::*;
use unitycatalog_common::models::volumes::v1::*;
use url::Url;
#[derive(Clone)]
pub struct UnityCatalogClient {
    catalogs: crate::codegen::catalogs::CatalogClient,
    credentials: crate::codegen::credentials::CredentialClient,
    delta_commits: crate::codegen::delta_commits::DeltaCommitClient,
    entity_tag_assignments: crate::codegen::entity_tag_assignments::EntityTagAssignmentClient,
    external_locations: crate::codegen::external_locations::ExternalLocationClient,
    functions: crate::codegen::functions::FunctionClient,
    providers: crate::codegen::providers::ProviderClient,
    recipients: crate::codegen::recipients::RecipientClient,
    schemas: crate::codegen::schemas::SchemaClient,
    shares: crate::codegen::shares::ShareClient,
    tables: crate::codegen::tables::TableClient,
    tag_policies: crate::codegen::tag_policies::TagPolicyClient,
    temporary_credentials: crate::codegen::temporary_credentials::TemporaryCredentialClient,
    volumes: crate::codegen::volumes::VolumeClient,
}
impl UnityCatalogClient {
    /// Create a new aggregate client from a cloud client and base URL.
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        let catalogs =
            crate::codegen::catalogs::CatalogClient::new(client.clone(), base_url.clone());
        let credentials =
            crate::codegen::credentials::CredentialClient::new(client.clone(), base_url.clone());
        let delta_commits =
            crate::codegen::delta_commits::DeltaCommitClient::new(client.clone(), base_url.clone());
        let entity_tag_assignments =
            crate::codegen::entity_tag_assignments::EntityTagAssignmentClient::new(
                client.clone(),
                base_url.clone(),
            );
        let external_locations = crate::codegen::external_locations::ExternalLocationClient::new(
            client.clone(),
            base_url.clone(),
        );
        let functions =
            crate::codegen::functions::FunctionClient::new(client.clone(), base_url.clone());
        let providers =
            crate::codegen::providers::ProviderClient::new(client.clone(), base_url.clone());
        let recipients =
            crate::codegen::recipients::RecipientClient::new(client.clone(), base_url.clone());
        let schemas = crate::codegen::schemas::SchemaClient::new(client.clone(), base_url.clone());
        let shares = crate::codegen::shares::ShareClient::new(client.clone(), base_url.clone());
        let tables = crate::codegen::tables::TableClient::new(client.clone(), base_url.clone());
        let tag_policies =
            crate::codegen::tag_policies::TagPolicyClient::new(client.clone(), base_url.clone());
        let temporary_credentials =
            crate::codegen::temporary_credentials::TemporaryCredentialClient::new(
                client.clone(),
                base_url.clone(),
            );
        let volumes = crate::codegen::volumes::VolumeClient::new(client.clone(), base_url.clone());
        Self {
            catalogs,
            credentials,
            delta_commits,
            entity_tag_assignments,
            external_locations,
            functions,
            providers,
            recipients,
            schemas,
            shares,
            tables,
            tag_policies,
            temporary_credentials,
            volumes,
        }
    }
    /// Create a new aggregate client with no authentication.
    pub fn new_unauthenticated(base_url: Url) -> Self {
        Self::new(CloudClient::new_unauthenticated(), base_url)
    }
    /// Create a new aggregate client authenticating with a bearer token.
    pub fn new_with_token(base_url: Url, token: impl ToString) -> Self {
        Self::new(CloudClient::new_with_token(token), base_url)
    }
    ///Low-level `catalogs` client exposing request/response passthrough methods.
    pub fn catalogs_client(&self) -> crate::codegen::catalogs::CatalogClient {
        self.catalogs.clone()
    }
    ///Low-level `credentials` client exposing request/response passthrough methods.
    pub fn credentials_client(&self) -> crate::codegen::credentials::CredentialClient {
        self.credentials.clone()
    }
    ///Low-level `delta_commits` client exposing request/response passthrough methods.
    pub fn delta_commits_client(&self) -> crate::codegen::delta_commits::DeltaCommitClient {
        self.delta_commits.clone()
    }
    ///Low-level `entity_tag_assignments` client exposing request/response passthrough methods.
    pub fn entity_tag_assignments_client(
        &self,
    ) -> crate::codegen::entity_tag_assignments::EntityTagAssignmentClient {
        self.entity_tag_assignments.clone()
    }
    ///Low-level `external_locations` client exposing request/response passthrough methods.
    pub fn external_locations_client(
        &self,
    ) -> crate::codegen::external_locations::ExternalLocationClient {
        self.external_locations.clone()
    }
    ///Low-level `functions` client exposing request/response passthrough methods.
    pub fn functions_client(&self) -> crate::codegen::functions::FunctionClient {
        self.functions.clone()
    }
    ///Low-level `providers` client exposing request/response passthrough methods.
    pub fn providers_client(&self) -> crate::codegen::providers::ProviderClient {
        self.providers.clone()
    }
    ///Low-level `recipients` client exposing request/response passthrough methods.
    pub fn recipients_client(&self) -> crate::codegen::recipients::RecipientClient {
        self.recipients.clone()
    }
    ///Low-level `schemas` client exposing request/response passthrough methods.
    pub fn schemas_client(&self) -> crate::codegen::schemas::SchemaClient {
        self.schemas.clone()
    }
    ///Low-level `shares` client exposing request/response passthrough methods.
    pub fn shares_client(&self) -> crate::codegen::shares::ShareClient {
        self.shares.clone()
    }
    ///Low-level `tables` client exposing request/response passthrough methods.
    pub fn tables_client(&self) -> crate::codegen::tables::TableClient {
        self.tables.clone()
    }
    ///Low-level `tag_policies` client exposing request/response passthrough methods.
    pub fn tag_policies_client(&self) -> crate::codegen::tag_policies::TagPolicyClient {
        self.tag_policies.clone()
    }
    ///Low-level `temporary_credentials` client exposing request/response passthrough methods.
    pub fn temporary_credentials_client(
        &self,
    ) -> crate::codegen::temporary_credentials::TemporaryCredentialClient {
        self.temporary_credentials.clone()
    }
    ///Low-level `volumes` client exposing request/response passthrough methods.
    pub fn volumes_client(&self) -> crate::codegen::volumes::VolumeClient {
        self.volumes.clone()
    }
    pub fn list_catalogs(&self) -> ListCatalogsBuilder {
        ListCatalogsBuilder::new(self.catalogs.clone())
    }
    pub fn create_catalog(&self, name: impl Into<String>) -> CreateCatalogBuilder {
        CreateCatalogBuilder::new(self.catalogs.clone(), name)
    }
    pub fn catalog(&self, name: impl ToString) -> CatalogClient {
        CatalogClient::new(name, self.catalogs.clone())
    }
    pub fn list_credentials(&self) -> ListCredentialsBuilder {
        ListCredentialsBuilder::new(self.credentials.clone())
    }
    pub fn create_credential(
        &self,
        name: impl Into<String>,
        purpose: Purpose,
    ) -> CreateCredentialBuilder {
        CreateCredentialBuilder::new(self.credentials.clone(), name, purpose)
    }
    pub fn credential(&self, name: impl ToString) -> CredentialClient {
        CredentialClient::new(name, self.credentials.clone())
    }
    pub fn commit(
        &self,
        table_id: impl Into<String>,
        table_uri: impl Into<String>,
    ) -> CommitBuilder {
        CommitBuilder::new(self.delta_commits.clone(), table_id, table_uri)
    }
    pub fn get_commits(
        &self,
        table_id: impl Into<String>,
        table_uri: impl Into<String>,
        start_version: i64,
    ) -> GetCommitsBuilder {
        GetCommitsBuilder::new(
            self.delta_commits.clone(),
            table_id,
            table_uri,
            start_version,
        )
    }
    pub fn list_entity_tag_assignments(
        &self,
        entity_type: impl Into<String>,
        entity_name: impl Into<String>,
    ) -> ListEntityTagAssignmentsBuilder {
        ListEntityTagAssignmentsBuilder::new(
            self.entity_tag_assignments.clone(),
            entity_type,
            entity_name,
        )
    }
    pub fn create_entity_tag_assignment(
        &self,
        tag_assignment: EntityTagAssignment,
    ) -> CreateEntityTagAssignmentBuilder {
        CreateEntityTagAssignmentBuilder::new(self.entity_tag_assignments.clone(), tag_assignment)
    }
    pub fn get_entity_tag_assignment(
        &self,
        entity_type: impl Into<String>,
        entity_name: impl Into<String>,
        tag_key: impl Into<String>,
    ) -> GetEntityTagAssignmentBuilder {
        GetEntityTagAssignmentBuilder::new(
            self.entity_tag_assignments.clone(),
            entity_type,
            entity_name,
            tag_key,
        )
    }
    pub fn update_entity_tag_assignment(
        &self,
        entity_type: impl Into<String>,
        entity_name: impl Into<String>,
        tag_key: impl Into<String>,
        tag_assignment: EntityTagAssignment,
    ) -> UpdateEntityTagAssignmentBuilder {
        UpdateEntityTagAssignmentBuilder::new(
            self.entity_tag_assignments.clone(),
            entity_type,
            entity_name,
            tag_key,
            tag_assignment,
        )
    }
    pub fn delete_entity_tag_assignment(
        &self,
        entity_type: impl Into<String>,
        entity_name: impl Into<String>,
        tag_key: impl Into<String>,
    ) -> DeleteEntityTagAssignmentBuilder {
        DeleteEntityTagAssignmentBuilder::new(
            self.entity_tag_assignments.clone(),
            entity_type,
            entity_name,
            tag_key,
        )
    }
    pub fn list_external_locations(&self) -> ListExternalLocationsBuilder {
        ListExternalLocationsBuilder::new(self.external_locations.clone())
    }
    pub fn create_external_location(
        &self,
        name: impl Into<String>,
        url: impl Into<String>,
        credential_name: impl Into<String>,
    ) -> CreateExternalLocationBuilder {
        CreateExternalLocationBuilder::new(
            self.external_locations.clone(),
            name,
            url,
            credential_name,
        )
    }
    pub fn external_location(&self, name: impl ToString) -> ExternalLocationClient {
        ExternalLocationClient::new(name, self.external_locations.clone())
    }
    pub fn list_functions(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
    ) -> ListFunctionsBuilder {
        ListFunctionsBuilder::new(self.functions.clone(), catalog_name, schema_name)
    }
    pub fn create_function(
        &self,
        name: impl Into<String>,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        data_type: impl Into<String>,
        full_data_type: impl Into<String>,
        parameter_style: ParameterStyle,
        is_deterministic: bool,
        sql_data_access: SqlDataAccess,
        is_null_call: bool,
        security_type: SecurityType,
        routine_body: RoutineBody,
    ) -> CreateFunctionBuilder {
        CreateFunctionBuilder::new(
            self.functions.clone(),
            name,
            catalog_name,
            schema_name,
            data_type,
            full_data_type,
            parameter_style,
            is_deterministic,
            sql_data_access,
            is_null_call,
            security_type,
            routine_body,
        )
    }
    pub fn function(
        &self,
        catalog_name: impl ToString,
        schema_name: impl ToString,
        function_name: impl ToString,
    ) -> FunctionClient {
        let full_name = format!(
            "{}.{}.{}",
            catalog_name.to_string(),
            schema_name.to_string(),
            function_name.to_string()
        );
        self.function_from_full_name(full_name)
    }
    pub fn function_from_full_name(&self, full_name: impl ToString) -> FunctionClient {
        FunctionClient::new_from_full_name(full_name, self.functions.clone())
    }
    pub fn list_providers(&self) -> ListProvidersBuilder {
        ListProvidersBuilder::new(self.providers.clone())
    }
    pub fn create_provider(
        &self,
        name: impl Into<String>,
        authentication_type: ProviderAuthenticationType,
    ) -> CreateProviderBuilder {
        CreateProviderBuilder::new(self.providers.clone(), name, authentication_type)
    }
    pub fn provider(&self, name: impl ToString) -> ProviderClient {
        ProviderClient::new(name, self.providers.clone())
    }
    pub fn list_recipients(&self) -> ListRecipientsBuilder {
        ListRecipientsBuilder::new(self.recipients.clone())
    }
    pub fn create_recipient(
        &self,
        name: impl Into<String>,
        authentication_type: AuthenticationType,
        owner: impl Into<String>,
    ) -> CreateRecipientBuilder {
        CreateRecipientBuilder::new(self.recipients.clone(), name, authentication_type, owner)
    }
    pub fn recipient(&self, name: impl ToString) -> RecipientClient {
        RecipientClient::new(name, self.recipients.clone())
    }
    pub fn list_schemas(&self, catalog_name: impl Into<String>) -> ListSchemasBuilder {
        ListSchemasBuilder::new(self.schemas.clone(), catalog_name)
    }
    pub fn create_schema(
        &self,
        name: impl Into<String>,
        catalog_name: impl Into<String>,
    ) -> CreateSchemaBuilder {
        CreateSchemaBuilder::new(self.schemas.clone(), name, catalog_name)
    }
    pub fn schema(&self, catalog_name: impl ToString, schema_name: impl ToString) -> SchemaClient {
        let full_name = format!("{}.{}", catalog_name.to_string(), schema_name.to_string());
        self.schema_from_full_name(full_name)
    }
    pub fn schema_from_full_name(&self, full_name: impl ToString) -> SchemaClient {
        SchemaClient::new_from_full_name(full_name, self.schemas.clone())
    }
    pub fn list_shares(&self) -> ListSharesBuilder {
        ListSharesBuilder::new(self.shares.clone())
    }
    pub fn create_share(&self, name: impl Into<String>) -> CreateShareBuilder {
        CreateShareBuilder::new(self.shares.clone(), name)
    }
    pub fn share(&self, name: impl ToString) -> ShareClient {
        ShareClient::new(name, self.shares.clone())
    }
    pub fn list_table_summaries(
        &self,
        catalog_name: impl Into<String>,
    ) -> ListTableSummariesBuilder {
        ListTableSummariesBuilder::new(self.tables.clone(), catalog_name)
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
        name: impl Into<String>,
        schema_name: impl Into<String>,
        catalog_name: impl Into<String>,
        table_type: TableType,
        data_source_format: DataSourceFormat,
    ) -> CreateTableBuilder {
        CreateTableBuilder::new(
            self.tables.clone(),
            name,
            schema_name,
            catalog_name,
            table_type,
            data_source_format,
        )
    }
    pub fn table(
        &self,
        catalog_name: impl ToString,
        schema_name: impl ToString,
        table_name: impl ToString,
    ) -> TableClient {
        let full_name = format!(
            "{}.{}.{}",
            catalog_name.to_string(),
            schema_name.to_string(),
            table_name.to_string()
        );
        self.table_from_full_name(full_name)
    }
    pub fn table_from_full_name(&self, full_name: impl ToString) -> TableClient {
        TableClient::new_from_full_name(full_name, self.tables.clone())
    }
    pub fn list_tag_policies(&self) -> ListTagPoliciesBuilder {
        ListTagPoliciesBuilder::new(self.tag_policies.clone())
    }
    pub fn create_tag_policy(&self, tag_policy: TagPolicy) -> CreateTagPolicyBuilder {
        CreateTagPolicyBuilder::new(self.tag_policies.clone(), tag_policy)
    }
    pub fn tag_policy(&self, tag_policy_name: impl ToString) -> TagPolicyClient {
        TagPolicyClient::new(tag_policy_name, self.tag_policies.clone())
    }
    pub fn generate_temporary_table_credentials(
        &self,
        table_id: impl Into<String>,
        operation: generate_temporary_table_credentials_request::Operation,
    ) -> GenerateTemporaryTableCredentialsBuilder {
        GenerateTemporaryTableCredentialsBuilder::new(
            self.temporary_credentials.clone(),
            table_id,
            operation,
        )
    }
    pub fn generate_temporary_path_credentials(
        &self,
        url: impl Into<String>,
        operation: generate_temporary_path_credentials_request::Operation,
    ) -> GenerateTemporaryPathCredentialsBuilder {
        GenerateTemporaryPathCredentialsBuilder::new(
            self.temporary_credentials.clone(),
            url,
            operation,
        )
    }
    pub fn generate_temporary_volume_credentials(
        &self,
        volume_id: impl Into<String>,
        operation: generate_temporary_volume_credentials_request::Operation,
    ) -> GenerateTemporaryVolumeCredentialsBuilder {
        GenerateTemporaryVolumeCredentialsBuilder::new(
            self.temporary_credentials.clone(),
            volume_id,
            operation,
        )
    }
    pub fn list_volumes(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
    ) -> ListVolumesBuilder {
        ListVolumesBuilder::new(self.volumes.clone(), catalog_name, schema_name)
    }
    pub fn create_volume(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        name: impl Into<String>,
        volume_type: VolumeType,
    ) -> CreateVolumeBuilder {
        CreateVolumeBuilder::new(
            self.volumes.clone(),
            catalog_name,
            schema_name,
            name,
            volume_type,
        )
    }
    pub fn volume(
        &self,
        catalog_name: impl ToString,
        schema_name: impl ToString,
        volume_name: impl ToString,
    ) -> VolumeClient {
        let full_name = format!(
            "{}.{}.{}",
            catalog_name.to_string(),
            schema_name.to_string(),
            volume_name.to_string()
        );
        self.volume_from_full_name(full_name)
    }
    pub fn volume_from_full_name(&self, full_name: impl ToString) -> VolumeClient {
        VolumeClient::new_from_full_name(full_name, self.volumes.clone())
    }
}
