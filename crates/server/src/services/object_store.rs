use std::sync::Arc;

use datafusion::common::{DataFusionError, Result as DFResult};
use deltalake_datafusion::ObjectStoreFactory;
use itertools::Itertools;
use object_store::DynObjectStore;
use object_store::azure::MicrosoftAzureBuilder;
use unitycatalog_common::credentials::v1::AzureManagedIdentity;
use unitycatalog_common::models::credentials::v1::{
    AzureServicePrincipal, AzureStorageKey, GetCredentialRequest,
    azure_managed_identity::Identifier as AzureMiIdentifier,
    azure_service_principal::Credential as AzureSpCredential,
};
use unitycatalog_common::models::external_locations::v1::ExternalLocationInfo;
use url::Url;

use super::ServerHandlerInner;
use super::location::{StorageLocationScheme, StorageLocationUrl};
use crate::api::CredentialHandler;
use crate::api::credentials::CredentialHandlerExt;
use crate::store::ResourceStore;
use crate::{Error, Result};

pub(crate) trait RegistryHandler:
    ResourceStore + CredentialHandler + CredentialHandlerExt
{
}
impl<T: ResourceStore + CredentialHandler + CredentialHandlerExt> RegistryHandler for T {}

#[async_trait::async_trait]
impl ObjectStoreFactory for ServerHandlerInner {
    async fn create_object_store(&self, location: &Url) -> DFResult<Arc<DynObjectStore>> {
        tracing::debug!("create_object_store: {:?}", location);
        let location = StorageLocationUrl::try_new(location.clone())
            .map_err(|e| DataFusionError::Execution(e.to_string()))?;
        get_object_store(&location, self)
            .await
            .map_err(|e| DataFusionError::Execution(e.to_string()))
    }
}

pub(crate) async fn get_object_store(
    location: &StorageLocationUrl,
    handler: &dyn RegistryHandler,
) -> Result<Arc<DynObjectStore>> {
    tracing::debug!("get_object_store: {:?}", location.location());
    // TODO(roeap): just listing all external locations could be very inefficient.
    // introduce an endpoint that allows us to query for specific resource properties instead
    let (locations, _) = handler
        .list(
            &unitycatalog_common::ObjectLabel::ExternalLocationInfo,
            None,
            None,
            None,
        )
        .await?;
    let locations: Vec<ExternalLocationInfo> =
        locations.into_iter().map(|l| l.try_into()).try_collect()?;
    // find the longest matching location
    let ext_loc = locations
        .iter()
        .filter(|l| {
            let ext_loc_url = StorageLocationUrl::parse(&l.url).unwrap();
            location
                .raw()
                .as_str()
                .starts_with(ext_loc_url.raw().as_str())
                || location
                    .location()
                    .as_str()
                    .starts_with(ext_loc_url.location().as_str())
        })
        .max_by(|l, r| l.url.len().cmp(&r.url.len()))
        .ok_or_else(|| Error::NotFound)?;
    let credential = handler
        .get_credential_internal(GetCredentialRequest {
            name: ext_loc.credential_name.clone(),
        })
        .await?;
    get_azure_store(
        location,
        credential.azure_managed_identity,
        credential.azure_service_principal,
        credential.azure_storage_key,
    )
}

fn get_azure_store(
    location: &StorageLocationUrl,
    azure_managed_identity: Option<AzureManagedIdentity>,
    azure_service_principal: Option<AzureServicePrincipal>,
    azure_storage_key: Option<AzureStorageKey>,
) -> Result<Arc<DynObjectStore>> {
    tracing::debug!("get_azure_store: {:?}", location.location());
    let url_err = || {
        Error::invalid_argument(
            "emulator URLs must encode the account and container name in the path",
        )
    };
    // check if the location is localhost to determine if we are running the emulator
    let mut builder = if matches!(location.scheme(), StorageLocationScheme::Azurite) {
        let container_name = location
            .location()
            .host_str()
            .ok_or_else(url_err)?
            .to_string();

        MicrosoftAzureBuilder::new()
            .with_use_emulator(true)
            .with_container_name(container_name)
    } else {
        MicrosoftAzureBuilder::new().with_url(location.raw().as_str())
    };

    if let Some(AzureServicePrincipal {
        directory_id,
        application_id,
        credential,
    }) = azure_service_principal
    {
        builder = builder
            .with_tenant_id(directory_id)
            .with_client_id(application_id);
        match credential {
            Some(AzureSpCredential::ClientSecret(client_secret)) => {
                builder = builder.with_client_secret(client_secret);
            }
            Some(AzureSpCredential::FederatedTokenFile(federated_token_file)) => {
                builder = builder.with_federated_token_file(federated_token_file);
            }
            _ => {
                return Err(Error::invalid_argument(
                    "Azure service principal requires a credential.",
                ));
            }
        };
    } else if let Some(AzureStorageKey {
        account_name,
        account_key,
    }) = azure_storage_key
    {
        builder = builder
            .with_account(account_name)
            .with_access_key(account_key);
    } else if let Some(AzureManagedIdentity { identifier }) = azure_managed_identity {
        use AzureMiIdentifier::*;
        match identifier {
            Some(ObjectId(_object_id)) => {
                todo!()
            }
            Some(ApplicationId(application_id)) => {
                builder = builder.with_client_id(application_id);
            }
            Some(MsiResourceId(_msi_resource_id)) => {
                todo!()
            }
            _ => (),
        }
    } else {
        return Err(Error::invalid_argument(
            "Azure service principal requires a credential.",
        ));
    }

    Ok(Arc::new(builder.build()?))
}
