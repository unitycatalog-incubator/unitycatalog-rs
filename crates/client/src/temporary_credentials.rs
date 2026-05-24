use olai_http::CloudClient;
use reqwest::IntoUrl;
use unitycatalog_common::models::temporary_credentials::v1::TemporaryCredential;
use unitycatalog_common::{
    models::temporary_credentials::v1::{
        GenerateTemporaryPathCredentialsRequest, GenerateTemporaryTableCredentialsRequest,
        GenerateTemporaryVolumeCredentialsRequest,
        generate_temporary_path_credentials_request::Operation as PthOperation,
        generate_temporary_table_credentials_request::Operation as TblOperation,
        generate_temporary_volume_credentials_request::Operation as VolOperation,
    },
    tables::v1::GetTableRequest,
    volumes::v1::GetVolumeRequest,
};
use url::Url;
use uuid::Uuid;

use crate::Result;
use crate::codegen::tables::TableClient as TableClientBase;
pub(super) use crate::codegen::temporary_credentials::TemporaryCredentialClient as TemporaryCredentialClientBase;
use crate::codegen::volumes::client::VolumeClient as VolumeClientBaseFromCodegen;

/// A reference to a table in unity catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableReference {
    /// The unique identifier of the table.
    Id(Uuid),
    /// The fully qualified name of the table.
    Name(String),
}

impl From<String> for TableReference {
    fn from(name: String) -> Self {
        TableReference::Name(name)
    }
}

impl From<&str> for TableReference {
    fn from(name: &str) -> Self {
        TableReference::Name(name.to_string())
    }
}

impl From<Uuid> for TableReference {
    fn from(id: Uuid) -> Self {
        TableReference::Id(id)
    }
}

/// A reference to a volume in unity catalog.
///
/// Use [`VolumeReference::Name`] for the three-level
/// `<catalog>.<schema>.<volume>` form and [`VolumeReference::Id`] when you
/// already hold the volume's UUID. The client resolves names to IDs
/// transparently on first use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumeReference {
    /// The unique identifier of the volume.
    Id(Uuid),
    /// The fully qualified `<catalog>.<schema>.<volume>` name.
    Name(String),
}

impl From<String> for VolumeReference {
    fn from(name: String) -> Self {
        VolumeReference::Name(name)
    }
}

impl From<&str> for VolumeReference {
    fn from(name: &str) -> Self {
        VolumeReference::Name(name.to_string())
    }
}

impl From<Uuid> for VolumeReference {
    fn from(id: Uuid) -> Self {
        VolumeReference::Id(id)
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

impl From<TableOperation> for TblOperation {
    fn from(operation: TableOperation) -> Self {
        match operation {
            TableOperation::Read => TblOperation::Read,
            TableOperation::ReadWrite => TblOperation::ReadWrite,
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

impl From<PathOperation> for PthOperation {
    fn from(operation: PathOperation) -> Self {
        match operation {
            PathOperation::Read => PthOperation::PathRead,
            PathOperation::ReadWrite => PthOperation::PathReadWrite,
            PathOperation::CreateTable => PthOperation::PathCreateTable,
        }
    }
}

/// The kind of access requested for a volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeOperation {
    /// Read-only access.
    Read,
    /// Read and write access.
    ReadWrite,
}

impl From<VolumeOperation> for i32 {
    fn from(operation: VolumeOperation) -> Self {
        match operation {
            VolumeOperation::Read => VolOperation::ReadVolume as i32,
            VolumeOperation::ReadWrite => VolOperation::WriteVolume as i32,
        }
    }
}

impl From<VolumeOperation> for VolOperation {
    fn from(operation: VolumeOperation) -> Self {
        match operation {
            VolumeOperation::Read => VolOperation::ReadVolume,
            VolumeOperation::ReadWrite => VolOperation::WriteVolume,
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

    /// Get a temporary credential for reading or writing to a volume.
    ///
    /// ## Parameters
    /// * `volume`: The volume to get a temporary credential for. May be either
    ///   a [`VolumeReference::Id`] (UUID, preferred when known) or a
    ///   [`VolumeReference::Name`] in three-level dotted form
    ///   (`catalog.schema.volume`). Names are resolved to IDs by issuing a
    ///   `GetVolume` request.
    /// * `operation`: Whether the credentials should grant read-only or
    ///   read-write access.
    ///
    /// ## Returns
    /// A tuple containing the temporary credential and the resolved volume ID.
    ///
    /// ## Server requirements
    /// The Unity Catalog metastore must have `external_access_enabled = true`
    /// and the caller must hold `EXTERNAL_USE_SCHEMA` on the parent schema.
    pub async fn temporary_volume_credential(
        &self,
        volume: impl Into<VolumeReference>,
        operation: VolumeOperation,
    ) -> Result<(TemporaryCredential, Uuid)> {
        let volume_id = match volume.into() {
            VolumeReference::Id(id) => id.as_hyphenated().to_string(),
            VolumeReference::Name(name) => {
                let volume_client = VolumeClientBaseFromCodegen::new(
                    self.client.client.clone(),
                    self.client.base_url.clone(),
                );
                let info = volume_client
                    .get_volume(&GetVolumeRequest {
                        name,
                        include_browse: Some(false),
                    })
                    .await?;
                info.volume_id
            }
        };
        let uuid =
            Uuid::parse_str(&volume_id).map_err(unitycatalog_common::Error::InvalidIdentifier)?;
        Ok((
            self.client
                .generate_temporary_volume_credentials(&GenerateTemporaryVolumeCredentialsRequest {
                    volume_id,
                    operation: operation.into(),
                })
                .await?,
            uuid,
        ))
    }
}
