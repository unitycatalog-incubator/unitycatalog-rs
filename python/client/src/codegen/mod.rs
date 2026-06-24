// @generated — do not edit by hand.
#![allow(dead_code, unused_imports, clippy::too_many_arguments)]
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod agent_skills;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod agents;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod catalogs;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod credentials;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod external_locations;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod functions;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod providers;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod recipients;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod schemas;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod shares;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod staging_tables;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod tables;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod tag_policies;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod volumes;
use crate::codegen::agent_skills::PyAgentSkillClient;
use crate::codegen::agents::PyAgentClient;
use crate::codegen::catalogs::PyCatalogClient;
use crate::codegen::credentials::PyCredentialClient;
use crate::codegen::external_locations::PyExternalLocationClient;
use crate::codegen::functions::PyFunctionClient;
use crate::codegen::providers::PyProviderClient;
use crate::codegen::recipients::PyRecipientClient;
use crate::codegen::schemas::PySchemaClient;
use crate::codegen::shares::PyShareClient;
use crate::codegen::staging_tables::PyStagingTableClient;
use crate::codegen::tables::PyTableClient;
use crate::codegen::tag_policies::PyTagPolicyClient;
use crate::codegen::volumes::PyVolumeClient;
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use futures::stream::TryStreamExt;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::models::agent_skills::v0alpha1::*;
use unitycatalog_common::models::agents::v0alpha1::*;
use unitycatalog_common::models::catalogs::v1::*;
use unitycatalog_common::models::credentials::v1::*;
use unitycatalog_common::models::delta_commits::v1::*;
use unitycatalog_common::models::external_locations::v1::*;
use unitycatalog_common::models::functions::v1::*;
use unitycatalog_common::models::providers::v1::*;
use unitycatalog_common::models::recipients::v1::*;
use unitycatalog_common::models::schemas::v1::*;
use unitycatalog_common::models::shares::v1::*;
use unitycatalog_common::models::staging_tables::v1::*;
use unitycatalog_common::models::tables::v1::*;
use unitycatalog_common::models::tags::v1::*;
use unitycatalog_common::models::tags::v1::*;
use unitycatalog_common::models::temporary_credentials::v1::*;
use unitycatalog_common::models::volumes::v1::*;
#[pyclass(name = "UnityCatalogClient")]
pub struct PyUnityCatalogClient {
    client: UnityCatalogClient,
}
#[pymethods]
impl PyUnityCatalogClient {
    #[new]
    #[pyo3(signature = (base_url, token = None))]
    pub fn new(base_url: String, token: Option<String>) -> PyResult<Self> {
        let client = if let Some(token) = token {
            olai_http::CloudClient::new_with_token(token)
        } else {
            olai_http::CloudClient::new_unauthenticated()
        };
        let base_url = base_url.parse().map_err(PyUnityCatalogError::from)?;
        Ok(Self {
            client: UnityCatalogClient::new(client, base_url),
        })
    }
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            max_results = None,
            include_browse = None
        )
    )]
    pub fn list_agent_skills(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<Vec<AgentSkill>> {
        let mut request = self.client.list_agent_skills(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            name,
            agent_skill_type,
            storage_location = None,
            description = None,
            license = None,
            allowed_tools = None,
            metadata = None,
            comment = None
        )
    )]
    pub fn create_agent_skill(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        name: String,
        agent_skill_type: AgentSkillType,
        storage_location: Option<String>,
        description: Option<String>,
        license: Option<String>,
        allowed_tools: Option<Vec<String>>,
        metadata: Option<HashMap<String, String>>,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<AgentSkill> {
        let mut request =
            self.client
                .create_agent_skill(catalog_name, schema_name, name, agent_skill_type);
        request = request.with_storage_location(storage_location);
        request = request.with_description(description);
        request = request.with_license(license);
        if let Some(allowed_tools) = allowed_tools {
            request = request.with_allowed_tools(allowed_tools);
        }
        if let Some(metadata) = metadata {
            request = request.with_metadata(metadata);
        }
        request = request.with_comment(comment);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            max_results = None,
            include_browse = None
        )
    )]
    pub fn list_agents(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<Vec<Agent>> {
        let mut request = self.client.list_agents(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            name,
            invocation_protocol,
            endpoint,
            description = None,
            capabilities = None,
            input_schema = None,
            comment = None
        )
    )]
    pub fn create_agent(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        name: String,
        invocation_protocol: InvocationProtocol,
        endpoint: String,
        description: Option<String>,
        capabilities: Option<Vec<String>>,
        input_schema: Option<String>,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<Agent> {
        let mut request = self.client.create_agent(
            catalog_name,
            schema_name,
            name,
            invocation_protocol,
            endpoint,
        );
        request = request.with_description(description);
        if let Some(capabilities) = capabilities {
            request = request.with_capabilities(capabilities);
        }
        request = request.with_input_schema(input_schema);
        request = request.with_comment(comment);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(signature = (max_results = None))]
    pub fn list_catalogs(
        &self,
        py: Python,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<Catalog>> {
        let mut request = self.client.list_catalogs();
        request = request.with_max_results(max_results);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(
        signature = (
            name,
            comment = None,
            properties = None,
            storage_root = None,
            provider_name = None,
            share_name = None
        )
    )]
    pub fn create_catalog(
        &self,
        py: Python,
        name: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        storage_root: Option<String>,
        provider_name: Option<String>,
        share_name: Option<String>,
    ) -> PyUnityCatalogResult<Catalog> {
        let mut request = self.client.create_catalog(name);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_storage_root(storage_root);
        request = request.with_provider_name(provider_name);
        request = request.with_share_name(share_name);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(signature = (purpose = None, max_results = None))]
    pub fn list_credentials(
        &self,
        py: Python,
        purpose: Option<Purpose>,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<Credential>> {
        let mut request = self.client.list_credentials();
        request = request.with_purpose(purpose);
        request = request.with_max_results(max_results);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(
        signature = (
            name,
            purpose,
            comment = None,
            read_only = None,
            skip_validation = None,
            azure_service_principal = None,
            azure_managed_identity = None,
            azure_storage_key = None,
            aws_iam_role = None,
            databricks_gcp_service_account = None
        )
    )]
    pub fn create_credential(
        &self,
        py: Python,
        name: String,
        purpose: Purpose,
        comment: Option<String>,
        read_only: Option<bool>,
        skip_validation: Option<bool>,
        azure_service_principal: Option<AzureServicePrincipal>,
        azure_managed_identity: Option<AzureManagedIdentity>,
        azure_storage_key: Option<AzureStorageKey>,
        aws_iam_role: Option<AwsIamRoleConfig>,
        databricks_gcp_service_account: Option<DatabricksGcpServiceAccount>,
    ) -> PyUnityCatalogResult<Credential> {
        let mut request = self.client.create_credential(name, purpose);
        request = request.with_comment(comment);
        request = request.with_read_only(read_only);
        request = request.with_skip_validation(skip_validation);
        request = request.with_azure_service_principal(azure_service_principal);
        request = request.with_azure_managed_identity(azure_managed_identity);
        request = request.with_azure_storage_key(azure_storage_key);
        request = request.with_aws_iam_role(aws_iam_role);
        request = request.with_databricks_gcp_service_account(databricks_gcp_service_account);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(
        signature = (
            table_id,
            table_uri,
            commit_info = None,
            latest_backfilled_version = None,
            metadata = None
        )
    )]
    pub fn commit(
        &self,
        py: Python,
        table_id: String,
        table_uri: String,
        commit_info: Option<CommitInfo>,
        latest_backfilled_version: Option<i64>,
        metadata: Option<Metadata>,
    ) -> PyUnityCatalogResult<()> {
        let mut request = self.client.commit(table_id, table_uri);
        request = request.with_commit_info(commit_info);
        request = request.with_latest_backfilled_version(latest_backfilled_version);
        request = request.with_metadata(metadata);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(signature = (table_id, table_uri, start_version, end_version = None))]
    pub fn get_commits(
        &self,
        py: Python,
        table_id: String,
        table_uri: String,
        start_version: i64,
        end_version: Option<i64>,
    ) -> PyUnityCatalogResult<GetCommitsResponse> {
        let mut request = self.client.get_commits(table_id, table_uri, start_version);
        request = request.with_end_version(end_version);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(
        signature = (entity_type, entity_name, max_results = None, page_token = None)
    )]
    pub fn list_entity_tag_assignments(
        &self,
        py: Python,
        entity_type: String,
        entity_name: String,
        max_results: Option<i32>,
        page_token: Option<String>,
    ) -> PyUnityCatalogResult<ListEntityTagAssignmentsResponse> {
        let mut request = self
            .client
            .list_entity_tag_assignments(entity_type, entity_name);
        request = request.with_max_results(max_results);
        request = request.with_page_token(page_token);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(signature = (tag_assignment))]
    pub fn create_entity_tag_assignment(
        &self,
        py: Python,
        tag_assignment: EntityTagAssignment,
    ) -> PyUnityCatalogResult<EntityTagAssignment> {
        let request = self.client.create_entity_tag_assignment(tag_assignment);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(signature = (entity_type, entity_name, tag_key))]
    pub fn get_entity_tag_assignment(
        &self,
        py: Python,
        entity_type: String,
        entity_name: String,
        tag_key: String,
    ) -> PyUnityCatalogResult<EntityTagAssignment> {
        let request = self
            .client
            .get_entity_tag_assignment(entity_type, entity_name, tag_key);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(
        signature = (
            entity_type,
            entity_name,
            tag_key,
            tag_assignment,
            update_mask = None
        )
    )]
    pub fn update_entity_tag_assignment(
        &self,
        py: Python,
        entity_type: String,
        entity_name: String,
        tag_key: String,
        tag_assignment: EntityTagAssignment,
        update_mask: Option<String>,
    ) -> PyUnityCatalogResult<EntityTagAssignment> {
        let mut request = self.client.update_entity_tag_assignment(
            entity_type,
            entity_name,
            tag_key,
            tag_assignment,
        );
        request = request.with_update_mask(update_mask);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(signature = (entity_type, entity_name, tag_key))]
    pub fn delete_entity_tag_assignment(
        &self,
        py: Python,
        entity_type: String,
        entity_name: String,
        tag_key: String,
    ) -> PyUnityCatalogResult<()> {
        let request = self
            .client
            .delete_entity_tag_assignment(entity_type, entity_name, tag_key);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(signature = (max_results = None, include_browse = None))]
    pub fn list_external_locations(
        &self,
        py: Python,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<Vec<ExternalLocation>> {
        let mut request = self.client.list_external_locations();
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(
        signature = (
            name,
            url,
            credential_name,
            read_only = None,
            comment = None,
            skip_validation = None
        )
    )]
    pub fn create_external_location(
        &self,
        py: Python,
        name: String,
        url: String,
        credential_name: String,
        read_only: Option<bool>,
        comment: Option<String>,
        skip_validation: Option<bool>,
    ) -> PyUnityCatalogResult<ExternalLocation> {
        let mut request = self
            .client
            .create_external_location(name, url, credential_name);
        request = request.with_read_only(read_only);
        request = request.with_comment(comment);
        request = request.with_skip_validation(skip_validation);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            max_results = None,
            include_browse = None
        )
    )]
    pub fn list_functions(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<Vec<Function>> {
        let mut request = self.client.list_functions(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(
        signature = (
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
            input_params = None,
            routine_definition = None,
            routine_body_language = None,
            comment = None,
            properties = None
        )
    )]
    pub fn create_function(
        &self,
        py: Python,
        name: String,
        catalog_name: String,
        schema_name: String,
        data_type: String,
        full_data_type: String,
        parameter_style: ParameterStyle,
        is_deterministic: bool,
        sql_data_access: SqlDataAccess,
        is_null_call: bool,
        security_type: SecurityType,
        routine_body: RoutineBody,
        input_params: Option<FunctionParameterInfos>,
        routine_definition: Option<String>,
        routine_body_language: Option<String>,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> PyUnityCatalogResult<Function> {
        let mut request = self.client.create_function(
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
        );
        request = request.with_input_params(input_params);
        request = request.with_routine_definition(routine_definition);
        request = request.with_routine_body_language(routine_body_language);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(signature = (max_results = None))]
    pub fn list_providers(
        &self,
        py: Python,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<Provider>> {
        let mut request = self.client.list_providers();
        request = request.with_max_results(max_results);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(
        signature = (
            name,
            authentication_type,
            owner = None,
            comment = None,
            recipient_profile_str = None,
            properties = None
        )
    )]
    pub fn create_provider(
        &self,
        py: Python,
        name: String,
        authentication_type: ProviderAuthenticationType,
        owner: Option<String>,
        comment: Option<String>,
        recipient_profile_str: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> PyUnityCatalogResult<Provider> {
        let mut request = self.client.create_provider(name, authentication_type);
        request = request.with_owner(owner);
        request = request.with_comment(comment);
        request = request.with_recipient_profile_str(recipient_profile_str);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(signature = (max_results = None))]
    pub fn list_recipients(
        &self,
        py: Python,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<Recipient>> {
        let mut request = self.client.list_recipients();
        request = request.with_max_results(max_results);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(
        signature = (
            name,
            authentication_type,
            owner,
            comment = None,
            properties = None,
            expiration_time = None
        )
    )]
    pub fn create_recipient(
        &self,
        py: Python,
        name: String,
        authentication_type: AuthenticationType,
        owner: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        expiration_time: Option<i64>,
    ) -> PyUnityCatalogResult<Recipient> {
        let mut request = self
            .client
            .create_recipient(name, authentication_type, owner);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_expiration_time(expiration_time);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(signature = (catalog_name, max_results = None, include_browse = None))]
    pub fn list_schemas(
        &self,
        py: Python,
        catalog_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<Vec<Schema>> {
        let mut request = self.client.list_schemas(catalog_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(
        signature = (
            name,
            catalog_name,
            comment = None,
            properties = None,
            storage_root = None
        )
    )]
    pub fn create_schema(
        &self,
        py: Python,
        name: String,
        catalog_name: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        storage_root: Option<String>,
    ) -> PyUnityCatalogResult<Schema> {
        let mut request = self.client.create_schema(name, catalog_name);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_storage_root(storage_root);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(signature = (max_results = None))]
    pub fn list_shares(
        &self,
        py: Python,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<Share>> {
        let mut request = self.client.list_shares();
        request = request.with_max_results(max_results);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(signature = (name, comment = None))]
    pub fn create_share(
        &self,
        py: Python,
        name: String,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<Share> {
        let mut request = self.client.create_share(name);
        request = request.with_comment(comment);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(signature = (name, catalog_name, schema_name))]
    pub fn create_staging_table(
        &self,
        py: Python,
        name: String,
        catalog_name: String,
        schema_name: String,
    ) -> PyUnityCatalogResult<StagingTable> {
        let request = self
            .client
            .create_staging_table(name, catalog_name, schema_name);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            max_results = None,
            include_delta_metadata = None,
            omit_columns = None,
            omit_properties = None,
            omit_username = None,
            include_browse = None,
            include_manifest_capabilities = None
        )
    )]
    pub fn list_tables(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_delta_metadata: Option<bool>,
        omit_columns: Option<bool>,
        omit_properties: Option<bool>,
        omit_username: Option<bool>,
        include_browse: Option<bool>,
        include_manifest_capabilities: Option<bool>,
    ) -> PyUnityCatalogResult<Vec<Table>> {
        let mut request = self.client.list_tables(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_delta_metadata(include_delta_metadata);
        request = request.with_omit_columns(omit_columns);
        request = request.with_omit_properties(omit_properties);
        request = request.with_omit_username(omit_username);
        request = request.with_include_browse(include_browse);
        request = request.with_include_manifest_capabilities(include_manifest_capabilities);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(
        signature = (
            name,
            schema_name,
            catalog_name,
            table_type,
            data_source_format,
            columns = None,
            storage_location = None,
            comment = None,
            properties = None,
            view_definition = None,
            view_dependencies = None
        )
    )]
    pub fn create_table(
        &self,
        py: Python,
        name: String,
        schema_name: String,
        catalog_name: String,
        table_type: TableType,
        data_source_format: DataSourceFormat,
        columns: Option<Vec<Column>>,
        storage_location: Option<String>,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        view_definition: Option<String>,
        view_dependencies: Option<DependencyList>,
    ) -> PyUnityCatalogResult<Table> {
        let mut request = self.client.create_table(
            name,
            schema_name,
            catalog_name,
            table_type,
            data_source_format,
        );
        if let Some(columns) = columns {
            request = request.with_columns(columns);
        }
        request = request.with_storage_location(storage_location);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_view_definition(view_definition);
        request = request.with_view_dependencies(view_dependencies);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(signature = (max_results = None))]
    pub fn list_tag_policies(
        &self,
        py: Python,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<Vec<TagPolicy>> {
        let mut request = self.client.list_tag_policies();
        request = request.with_max_results(max_results);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(signature = (tag_policy))]
    pub fn create_tag_policy(
        &self,
        py: Python,
        tag_policy: TagPolicy,
    ) -> PyUnityCatalogResult<TagPolicy> {
        let request = self.client.create_tag_policy(tag_policy);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(signature = (table_id, operation))]
    pub fn generate_temporary_table_credentials(
        &self,
        py: Python,
        table_id: String,
        operation: generate_temporary_table_credentials_request::Operation,
    ) -> PyUnityCatalogResult<TemporaryCredential> {
        let request = self
            .client
            .generate_temporary_table_credentials(table_id, operation);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(signature = (url, operation, dry_run = None))]
    pub fn generate_temporary_path_credentials(
        &self,
        py: Python,
        url: String,
        operation: generate_temporary_path_credentials_request::Operation,
        dry_run: Option<bool>,
    ) -> PyUnityCatalogResult<TemporaryCredential> {
        let mut request = self
            .client
            .generate_temporary_path_credentials(url, operation);
        request = request.with_dry_run(dry_run);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(signature = (volume_id, operation))]
    pub fn generate_temporary_volume_credentials(
        &self,
        py: Python,
        volume_id: String,
        operation: generate_temporary_volume_credentials_request::Operation,
    ) -> PyUnityCatalogResult<TemporaryCredential> {
        let request = self
            .client
            .generate_temporary_volume_credentials(volume_id, operation);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            max_results = None,
            include_browse = None
        )
    )]
    pub fn list_volumes(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<Vec<Volume>> {
        let mut request = self.client.list_volumes(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            let result =
                runtime.block_on(async move { request.into_stream().try_collect().await })?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            name,
            volume_type,
            storage_location = None,
            comment = None
        )
    )]
    pub fn create_volume(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        name: String,
        volume_type: VolumeType,
        storage_location: Option<String>,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<Volume> {
        let mut request = self
            .client
            .create_volume(catalog_name, schema_name, name, volume_type);
        request = request.with_storage_location(storage_location);
        request = request.with_comment(comment);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| Ok::<_, PyUnityCatalogError>(runtime.block_on(request.into_future())?))
    }
    pub fn agent_skill(
        &self,
        catalog_name: String,
        schema_name: String,
        agent_skill_name: String,
    ) -> PyAgentSkillClient {
        let full_name = format!("{}.{}.{}", catalog_name, schema_name, agent_skill_name);
        PyAgentSkillClient {
            client: self.client.agent_skill_from_full_name(full_name),
        }
    }
    pub fn agent(
        &self,
        catalog_name: String,
        schema_name: String,
        agent_name: String,
    ) -> PyAgentClient {
        let full_name = format!("{}.{}.{}", catalog_name, schema_name, agent_name);
        PyAgentClient {
            client: self.client.agent_from_full_name(full_name),
        }
    }
    pub fn catalog(&self, catalog_name: String) -> PyCatalogClient {
        PyCatalogClient {
            client: self.client.catalog(catalog_name),
        }
    }
    pub fn credential(&self, credential_name: String) -> PyCredentialClient {
        PyCredentialClient {
            client: self.client.credential(credential_name),
        }
    }
    pub fn external_location(&self, external_location_name: String) -> PyExternalLocationClient {
        PyExternalLocationClient {
            client: self.client.external_location(external_location_name),
        }
    }
    pub fn function(
        &self,
        catalog_name: String,
        schema_name: String,
        function_name: String,
    ) -> PyFunctionClient {
        let full_name = format!("{}.{}.{}", catalog_name, schema_name, function_name);
        PyFunctionClient {
            client: self.client.function_from_full_name(full_name),
        }
    }
    pub fn provider(&self, provider_name: String) -> PyProviderClient {
        PyProviderClient {
            client: self.client.provider(provider_name),
        }
    }
    pub fn recipient(&self, recipient_name: String) -> PyRecipientClient {
        PyRecipientClient {
            client: self.client.recipient(recipient_name),
        }
    }
    pub fn schema(&self, catalog_name: String, schema_name: String) -> PySchemaClient {
        let full_name = format!("{}.{}", catalog_name, schema_name);
        PySchemaClient {
            client: self.client.schema_from_full_name(full_name),
        }
    }
    pub fn share(&self, share_name: String) -> PyShareClient {
        PyShareClient {
            client: self.client.share(share_name),
        }
    }
    pub fn staging_table(&self, staging_table_name: String) -> PyStagingTableClient {
        PyStagingTableClient {
            client: self.client.staging_table(staging_table_name),
        }
    }
    pub fn table(
        &self,
        catalog_name: String,
        schema_name: String,
        table_name: String,
    ) -> PyTableClient {
        let full_name = format!("{}.{}.{}", catalog_name, schema_name, table_name);
        PyTableClient {
            client: self.client.table_from_full_name(full_name),
        }
    }
    pub fn tag_policy(&self, tag_policy_name: String) -> PyTagPolicyClient {
        PyTagPolicyClient {
            client: self.client.tag_policy(tag_policy_name),
        }
    }
    pub fn volume(
        &self,
        catalog_name: String,
        schema_name: String,
        volume_name: String,
    ) -> PyVolumeClient {
        let full_name = format!("{}.{}.{}", catalog_name, schema_name, volume_name);
        PyVolumeClient {
            client: self.client.volume_from_full_name(full_name),
        }
    }
}
