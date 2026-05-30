use unitycatalog_common::models::credentials::v1::GetCredentialRequest;
use unitycatalog_common::models::tables::v1::Table;
use unitycatalog_common::models::temporary_credentials::v1::*;
use unitycatalog_common::models::{ResourceIdent, ResourceRef};

use super::RequestContext;
use crate::api::CredentialHandler;
use crate::api::credentials::CredentialHandlerExt;
pub use crate::codegen::temporary_credentials::TemporaryCredentialHandler;
use crate::policy::{Permission, Policy};
use crate::services::credential_vending::{VendOperation, vend_credential};
use crate::services::location::StorageLocationUrl;
use crate::services::object_store::find_external_location_for_url;
use crate::store::ResourceStore;
use crate::{Error, Result};

/// The permission a vend operation requires from the policy.
///
/// Read-only vending requires [`Permission::Read`]; read-write vending requires
/// [`Permission::Write`] so the policy can deny write access independently.
fn required_permission(operation: VendOperation) -> Permission {
    match operation {
        VendOperation::Read => Permission::Read,
        VendOperation::ReadWrite => Permission::Write,
    }
}

/// Map the proto `operation` integer to a `VendOperation`.
///
/// - `PATH_CREATE_TABLE` and `READ_WRITE` are treated as `ReadWrite`.
/// - `Unspecified` defaults to `Read` (least privilege).
fn to_vend_operation(operation: i32) -> VendOperation {
    use generate_temporary_path_credentials_request::Operation as PathOp;
    use generate_temporary_table_credentials_request::Operation as TableOp;

    // Check path operations first (values 0–3 are defined for both enums,
    // but semantically we just need to distinguish read from read-write).
    match PathOp::try_from(operation) {
        Ok(PathOp::PathReadWrite | PathOp::PathCreateTable) => return VendOperation::ReadWrite,
        Ok(PathOp::PathRead | PathOp::Unspecified) | Err(_) => {}
    }
    // Also check the table operation enum (READ = 1, READ_WRITE = 2).
    match TableOp::try_from(operation) {
        Ok(TableOp::ReadWrite) => VendOperation::ReadWrite,
        _ => VendOperation::Read,
    }
}

#[async_trait::async_trait]
impl<
    T: ResourceStore
        + Policy<RequestContext>
        + CredentialHandler<RequestContext>
        + CredentialHandlerExt,
> TemporaryCredentialHandler<RequestContext> for T
{
    #[tracing::instrument(skip(self, context))]
    async fn generate_temporary_path_credentials(
        &self,
        request: GenerateTemporaryPathCredentialsRequest,
        context: RequestContext,
    ) -> Result<TemporaryCredential> {
        let operation = to_vend_operation(request.operation);
        let storage_url = StorageLocationUrl::parse(&request.url)?;
        let ext_loc = find_external_location_for_url(&storage_url, self).await?;
        // Authorize against the concrete external location and the operation
        // actually requested, rather than the unscoped `SecuredAction` default.
        self.authorize_checked(
            &(&ext_loc).into(),
            &required_permission(operation),
            &context,
        )
        .await?;
        let credential = self
            .get_credential_internal(GetCredentialRequest {
                name: ext_loc.credential_name.clone(),
            })
            .await?;
        vend_credential(&credential, &request.url, operation).await
    }

    /// Generate temporary credentials for a volume.
    ///
    /// **Not yet implemented.** Tracking issue:
    /// <https://github.com/unitycatalog-incubator/unitycatalog-rs/issues/119>.
    ///
    /// The endpoint is defined in proto and exposed through the client
    /// SDKs so callers targeting Databricks or any other compliant
    /// server can use it today. This server returns
    /// [`Error::NotImplemented`] until the handler lands.
    ///
    /// See: <https://docs.databricks.com/api/workspace/temporaryvolumecredentials/generatetemporaryvolumecredentials>
    #[tracing::instrument(skip(self, _context))]
    async fn generate_temporary_volume_credentials(
        &self,
        _request: GenerateTemporaryVolumeCredentialsRequest,
        _context: RequestContext,
    ) -> Result<TemporaryCredential> {
        Err(Error::NotImplemented("temporary-volume-credentials"))
    }

    #[tracing::instrument(skip(self, context))]
    async fn generate_temporary_table_credentials(
        &self,
        request: GenerateTemporaryTableCredentialsRequest,
        context: RequestContext,
    ) -> Result<TemporaryCredential> {
        let operation = to_vend_operation(request.operation);
        let table_id = uuid::Uuid::parse_str(&request.table_id)
            .map_err(|_| Error::invalid_argument("table_id is not a valid UUID"))?;
        // Authorize against the concrete table and the operation actually
        // requested, rather than the unscoped `SecuredAction` default.
        let table_ident = ResourceIdent::Table(ResourceRef::Uuid(table_id));
        self.authorize_checked(&table_ident, &required_permission(operation), &context)
            .await?;
        let (resource, _) = self.get(&table_ident).await?;
        let table: Table = resource.try_into()?;
        let location = table
            .storage_location
            .ok_or_else(|| Error::invalid_argument("table does not have a storage location"))?;
        let storage_url = StorageLocationUrl::parse(&location)?;
        let ext_loc = find_external_location_for_url(&storage_url, self).await?;
        let credential = self
            .get_credential_internal(GetCredentialRequest {
                name: ext_loc.credential_name.clone(),
            })
            .await?;
        vend_credential(&credential, &location, operation).await
    }
}
// NOTE: These request types intentionally do not implement `SecuredAction`.
// Authorization is performed inside the handlers against the *concrete* resolved
// resource (external location / table) and the *requested* operation
// (read vs. read-write), which a static `SecuredAction` impl cannot express.
// See `generate_temporary_path_credentials` / `generate_temporary_table_credentials`.
