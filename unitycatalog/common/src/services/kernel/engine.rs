use std::sync::Arc;

use itertools::Itertools;
use object_store::DynObjectStore;
use object_store::azure::MicrosoftAzureBuilder;

use crate::api::CredentialsHandler;
use crate::models::credentials::v1::credential_info::Credential;
use crate::models::credentials::v1::{
    AzureServicePrincipal, AzureStorageKey, GetCredentialRequest,
};
use crate::models::external_locations::v1::ExternalLocationInfo;
use crate::resources::ResourceStore;
use crate::services::location::{StorageLocationScheme, StorageLocationUrl};
use crate::{Error, Result};

pub(crate) trait RegistryHandler: ResourceStore + CredentialsHandler {}
impl<T: ResourceStore + CredentialsHandler> RegistryHandler for T {}

pub(crate) async fn get_object_store(
    location: &StorageLocationUrl,
    handler: &dyn RegistryHandler,
) -> Result<Arc<DynObjectStore>> {
    tracing::debug!("get_object_store: {:?}", location.location());
    // TODO(roeap): just listing all external locations could be very inefficient.
    // introduce an endpoint that allows us to query for specific resource properties instead
    let (locations, _) = handler
        .list(&crate::ObjectLabel::ExternalLocationInfo, None, None, None)
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
    let Some(cred) = credential.credential else {
        return Err(Error::NotFound);
    };
    match cred {
        Credential::AzureStorageKey(_)
        | Credential::AzureServicePrincipal(_)
        | Credential::AzureManagedIdentity(_) => get_azure_store(location, cred),
    }
}

fn get_azure_store(
    location: &StorageLocationUrl,
    credential: Credential,
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
    match credential {
        Credential::AzureStorageKey(AzureStorageKey {
            account_name,
            account_key,
        }) => {
            builder = builder
                .with_account(account_name)
                .with_access_key(account_key);
        }
        Credential::AzureServicePrincipal(AzureServicePrincipal {
            directory_id,
            application_id,
            credential,
        }) => {
            builder = builder
                .with_tenant_id(directory_id)
                .with_client_id(application_id);
        }
        _ => {
            return Err(Error::invalid_argument(
                "Invalid credential for Azure Blob Storage.",
            ));
        }
    }
    Ok(Arc::new(builder.build()?))
}
