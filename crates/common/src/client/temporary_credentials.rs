use cloud_client::CloudClient;
use reqwest::IntoUrl;
use url::Url;
use uuid::Uuid;

pub(super) use crate::codegen::temporary_credentials::TemporaryCredentialClient as TemporaryCredentialClientBase;
use crate::models::temporary_credentials::v1::TemporaryCredential;
use crate::temporary_credentials::v1::{
    GenerateTemporaryPathCredentialsRequest, GenerateTemporaryTableCredentialsRequest,
};
use crate::{
    Result,
    client::TableClientBase,
    models::temporary_credentials::v1::{
        generate_temporary_path_credentials_request::Operation as PthOperation,
        generate_temporary_table_credentials_request::Operation as TblOperation,
    },
    tables::v1::GetTableRequest,
};

/// A reference to a table in unity catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableReference {
    /// The unique identifier of the table.
    Id(uuid::Uuid),
    /// The fully qualified name of the table.
    Name(String),
}

impl From<String> for TableReference {
    fn from(name: String) -> Self {
        TableReference::Name(name)
    }
}

impl From<Uuid> for TableReference {
    fn from(id: Uuid) -> Self {
        TableReference::Id(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableOperation {
    Read,
    ReadWrite,
}

impl From<TableOperation> for i32 {
    fn from(operation: TableOperation) -> Self {
        match operation {
            TableOperation::Read => TblOperation::Read as i32,
            TableOperation::ReadWrite => TblOperation::ReadWrite as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathOperation {
    Read,
    ReadWrite,
    CreateTable,
}

impl From<PathOperation> for i32 {
    fn from(operation: PathOperation) -> Self {
        match operation {
            PathOperation::Read => PthOperation::PathRead as i32,
            PathOperation::ReadWrite => PthOperation::PathReadWrite as i32,
            PathOperation::CreateTable => PthOperation::PathCreateTable as i32,
        }
    }
}

#[derive(Clone)]
pub struct TemporaryCredentialClient {
    client: TemporaryCredentialClientBase,
}

impl TemporaryCredentialClient {
    pub fn new_with_url(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self {
            client: TemporaryCredentialClientBase::new(client, base_url),
        }
    }

    pub fn new(client: TemporaryCredentialClientBase) -> Self {
        Self { client }
    }

    /// Get a temporary credential for reading or writing to a table.
    ///
    /// ## Parameters
    ///
    /// * `table`: The table to get a temporary credential for.
    /// * `operation`: The operation to perform on the table.
    ///
    /// ## Returns
    ///
    /// A tuple containing the temporary credential and the resolved table ID.
    pub async fn temporary_table_credential(
        &self,
        table: impl Into<TableReference>,
        operation: TableOperation,
    ) -> Result<(TemporaryCredential, Uuid)> {
        let table_id = match table.into() {
            TableReference::Id(id) => id.as_hyphenated().to_string(),
            TableReference::Name(name) => {
                let table_client =
                    TableClientBase::new(self.client.client.clone(), self.client.base_url.clone());
                let table_info = table_client
                    .get_table(&GetTableRequest {
                        full_name: name,
                        include_browse: Some(false),
                        include_delta_metadata: Some(false),
                        include_manifest_capabilities: Some(false),
                    })
                    .await?;
                table_info.table_id().to_string()
            }
        };
        let uuid = Uuid::parse_str(&table_id).unwrap();
        Ok((
            self.client
                .generate_temporary_table_credentials(&GenerateTemporaryTableCredentialsRequest {
                    table_id,
                    operation: operation.into(),
                })
                .await?,
            uuid,
        ))
    }

    pub async fn temporary_path_credential(
        &self,
        path: impl IntoUrl,
        operation: PathOperation,
        dry_run: impl Into<Option<bool>>,
    ) -> Result<(TemporaryCredential, Url)> {
        let url = path.into_url()?;
        Ok((
            self.client
                .generate_temporary_path_credentials(&GenerateTemporaryPathCredentialsRequest {
                    url: url.to_string(),
                    operation: operation.into(),
                    dry_run: dry_run.into(),
                })
                .await?,
            url,
        ))
    }
}
