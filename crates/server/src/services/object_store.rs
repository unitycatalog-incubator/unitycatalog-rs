use std::sync::Arc;

use super::kernel::ObjectStoreFactory;
use datafusion::common::{DataFusionError, Result as DFResult};
use itertools::Itertools;
use object_store::DynObjectStore;
use object_store::azure::MicrosoftAzureBuilder;
use unitycatalog_common::credentials::v1::AzureManagedIdentity;
use unitycatalog_common::models::credentials::v1::{
    AzureServicePrincipal, AzureStorageKey, GetCredentialRequest,
    azure_service_principal::Credential as AzureSpCredential,
};
use unitycatalog_common::models::external_locations::v1::ExternalLocation;
use url::Url;

use super::ServerHandlerInner;
use super::location::{StorageLocationScheme, StorageLocationUrl};
use crate::api::CredentialHandler;
use crate::api::credentials::CredentialHandlerExt;
use crate::store::ResourceStore;
use crate::{Error, Result};

pub(crate) trait RegistryHandler:
    ResourceStore + CredentialHandler<crate::api::RequestContext> + CredentialHandlerExt
{
}
impl<T: ResourceStore + CredentialHandler<crate::api::RequestContext> + CredentialHandlerExt>
    RegistryHandler for T
{
}

#[async_trait::async_trait]
impl ObjectStoreFactory for ServerHandlerInner<crate::api::RequestContext> {
    async fn create_object_store(&self, location: &Url) -> DFResult<Arc<DynObjectStore>> {
        tracing::debug!("create_object_store: {:?}", location);
        let location = StorageLocationUrl::try_new(location.clone())
            .map_err(|e| DataFusionError::Execution(e.to_string()))?;
        get_object_store(&location, self)
            .await
            .map_err(|e| DataFusionError::Execution(e.to_string()))
    }
}

/// Find the most specific external location whose URL is a prefix of `location`.
///
/// Returns the external location with the longest matching URL prefix, which
/// provides the most specific credential for the requested storage path.
pub(crate) async fn find_external_location_for_url(
    location: &StorageLocationUrl,
    handler: &dyn RegistryHandler,
) -> Result<ExternalLocation> {
    // TODO(roeap): just listing all external locations could be very inefficient.
    // introduce an endpoint that allows us to query for specific resource properties instead
    let (locations, _) = handler
        .list(
            &unitycatalog_common::ObjectLabel::ExternalLocation,
            None,
            None,
            None,
        )
        .await?;
    let locations: Vec<ExternalLocation> =
        locations.into_iter().map(|l| l.try_into()).try_collect()?;
    // find the longest matching location
    locations
        .into_iter()
        .filter(|l| {
            let Ok(ext_loc_url) = StorageLocationUrl::parse(&l.url) else {
                return false;
            };
            is_path_prefix(location.raw().as_str(), ext_loc_url.raw().as_str())
                || is_path_prefix(
                    location.location().as_str(),
                    ext_loc_url.location().as_str(),
                )
        })
        .max_by(|l, r| l.url.len().cmp(&r.url.len()))
        .ok_or(Error::NotFound)
}

/// Whether `prefix` is a path-segment prefix of `url`.
///
/// Unlike a raw `starts_with`, this respects path boundaries so that a location
/// registered for `s3://bucket/data` does **not** match `s3://bucket/data-secret/x`
/// (which `starts_with` alone would wrongly accept). A match requires either an
/// exact equality or that the character immediately after `prefix` is a `/`.
/// A trailing `/` on `prefix` is normalized away first.
fn is_path_prefix(url: &str, prefix: &str) -> bool {
    let prefix = prefix.strip_suffix('/').unwrap_or(prefix);
    match url.strip_prefix(prefix) {
        Some("") => true,
        Some(rest) => rest.starts_with('/'),
        None => false,
    }
}

pub(crate) async fn get_object_store(
    location: &StorageLocationUrl,
    handler: &dyn RegistryHandler,
) -> Result<Arc<DynObjectStore>> {
    tracing::debug!("get_object_store: {:?}", location.location());
    let ext_loc = find_external_location_for_url(location, handler).await?;
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
    } else if let Some(msi) = azure_managed_identity {
        // managed_identity_id is the ARM resource ID of a user-assigned identity.
        // Pass it as the client_id to the object store builder when present.
        // When absent, the system-assigned identity of the Access Connector is used.
        if let Some(managed_identity_id) = msi.managed_identity_id {
            builder = builder.with_client_id(managed_identity_id);
        }
    } else {
        return Err(Error::invalid_argument(
            "Azure service principal requires a credential.",
        ));
    }

    Ok(Arc::new(builder.build()?))
}

#[cfg(test)]
mod tests {
    use super::is_path_prefix;

    #[test]
    fn path_prefix_matches_exact_and_subpaths() {
        assert!(is_path_prefix("s3://bucket/data", "s3://bucket/data"));
        assert!(is_path_prefix("s3://bucket/data/file", "s3://bucket/data"));
        assert!(is_path_prefix("s3://bucket/data/x/y", "s3://bucket/data/"));
    }

    #[test]
    fn path_prefix_rejects_sibling_prefix() {
        // The classic over-match: `data` must not match `data-secret`.
        assert!(!is_path_prefix(
            "s3://bucket/data-secret/file",
            "s3://bucket/data"
        ));
        assert!(!is_path_prefix("s3://bucket/database", "s3://bucket/data"));
    }
}
