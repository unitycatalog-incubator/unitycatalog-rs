use unitycatalog_common::models::credentials::v1::GetCredentialRequest;
use unitycatalog_common::models::tables::v1::Table;
use unitycatalog_common::models::temporary_credentials::v1::*;
use unitycatalog_common::models::{ResourceIdent, ResourceRef};

use super::{RequestContext, SecuredAction};
use crate::api::CredentialHandler;
use crate::api::credentials::CredentialHandlerExt;
pub use crate::codegen::temporary_credentials::TemporaryCredentialHandler;
use crate::policy::{Permission, Policy};
use crate::services::credential_vending::vend_credential;
use crate::services::location::StorageLocationUrl;
use crate::services::object_store::find_external_location_for_url;
use crate::store::ResourceStore;
use crate::{Error, Result};

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
        self.check_required(&request, &context).await?;
        let storage_url = StorageLocationUrl::parse(&request.url)?;
        let ext_loc = find_external_location_for_url(&storage_url, self).await?;
        let credential = self
            .get_credential_internal(GetCredentialRequest {
                name: ext_loc.credential_name.clone(),
            })
            .await?;
        vend_credential(&credential, &request.url).await
    }

    #[tracing::instrument(skip(self, context))]
    async fn generate_temporary_table_credentials(
        &self,
        request: GenerateTemporaryTableCredentialsRequest,
        context: RequestContext,
    ) -> Result<TemporaryCredential> {
        self.check_required(&request, &context).await?;
        let table_id = uuid::Uuid::parse_str(&request.table_id)
            .map_err(|_| Error::invalid_argument("table_id is not a valid UUID"))?;
        let (resource, _) = self
            .get(&ResourceIdent::Table(ResourceRef::Uuid(table_id)))
            .await?;
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
        vend_credential(&credential, &location).await
    }
}

impl SecuredAction for GenerateTemporaryPathCredentialsRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::external_location(ResourceRef::Undefined)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GenerateTemporaryTableCredentialsRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::table(ResourceRef::Undefined)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}
