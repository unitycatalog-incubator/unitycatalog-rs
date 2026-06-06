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
use unitycatalog_common::models::tables::v1::Table;
use unitycatalog_common::models::volumes::v1::Volume;
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
    handler: &(impl ResourceStore + ?Sized),
) -> Result<ExternalLocation> {
    let locations = list_external_locations(handler).await?;
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

/// All registered external locations paired with their parsed URLs.
///
/// Mirrors the listing/tolerance behaviour of [`find_external_location_for_url`]:
/// entries whose `url` fails to parse are silently skipped. Returned alongside
/// each location's name so callers can exclude a specific record (e.g. the one
/// being updated).
///
/// TODO(roeap): just listing all external locations could be very inefficient.
/// introduce an endpoint that allows us to query for specific resource properties instead
pub(crate) async fn list_external_locations(
    handler: &(impl ResourceStore + ?Sized),
) -> Result<Vec<ExternalLocation>> {
    let (locations, _) = handler
        .list(
            &unitycatalog_common::ObjectLabel::ExternalLocation,
            None,
            None,
            None,
        )
        .await?;
    Ok(locations.into_iter().map(|l| l.try_into()).try_collect()?)
}

/// All non-empty `storage_location`s currently registered for tables and
/// volumes, parsed into [`StorageLocationUrl`]s.
///
/// Used to enforce that external tables/volumes do not overlap any existing
/// table or volume. Entries without a storage location, or whose location fails
/// to parse, are skipped — matching the tolerance of the other helpers here.
///
/// TODO(roeap): just listing all tables and volumes could be very inefficient.
/// introduce an endpoint that allows us to query for specific resource properties instead
pub(crate) async fn list_table_volume_locations(
    handler: &(impl ResourceStore + ?Sized),
) -> Result<Vec<StorageLocationUrl>> {
    let mut out = Vec::new();

    let (tables, _) = handler
        .list(&unitycatalog_common::ObjectLabel::Table, None, None, None)
        .await?;
    for resource in tables {
        let table: Table = resource.try_into()?;
        if let Some(loc) = table.storage_location.filter(|s| !s.is_empty()) {
            if let Ok(url) = StorageLocationUrl::parse(&loc) {
                out.push(url);
            }
        }
    }

    let (volumes, _) = handler
        .list(&unitycatalog_common::ObjectLabel::Volume, None, None, None)
        .await?;
    for resource in volumes {
        let volume: Volume = resource.try_into()?;
        if !volume.storage_location.is_empty() {
            if let Ok(url) = StorageLocationUrl::parse(&volume.storage_location) {
                out.push(url);
            }
        }
    }

    Ok(out)
}

/// Validate the `storage_location` of an external table or volume.
///
/// Enforces the two Unity Catalog rules for external securables:
/// 1. the path must be contained within a registered external location; and
/// 2. the path must not overlap any existing table or volume.
///
/// Returns [`Error::invalid_argument`] on a violation. The containment check
/// reuses [`find_external_location_for_url`]; a missing enclosing location
/// surfaces as a clear argument error rather than the raw `NotFound`.
pub(crate) async fn validate_external_storage_location(
    handler: &(impl ResourceStore + ?Sized),
    location: &StorageLocationUrl,
) -> Result<()> {
    // 1. Must reside within a registered external location.
    if let Err(Error::NotFound) = find_external_location_for_url(location, handler).await {
        return Err(Error::invalid_argument(format!(
            "storage location '{}' is not contained within any registered external location",
            location.raw()
        )));
    }

    // 2. Must not overlap any existing table or volume.
    for existing in list_table_volume_locations(handler).await? {
        if locations_overlap(location, &existing) {
            return Err(Error::invalid_argument(format!(
                "storage location '{}' overlaps existing table or volume location '{}'",
                location.raw(),
                existing.raw()
            )));
        }
    }

    Ok(())
}

/// Whether two storage locations overlap: one is equal to, or nested within,
/// the other. Unity Catalog forbids overlapping governed storage regions
/// (external locations, tables, volumes), so this is the core guard used when
/// registering them. The check is symmetric and respects path-segment
/// boundaries (see [`is_path_prefix`]), comparing on both the raw and
/// normalized URL forms so that, e.g., `abfss://` and `https://` spellings of
/// the same Azure location are caught.
pub(crate) fn locations_overlap(a: &StorageLocationUrl, b: &StorageLocationUrl) -> bool {
    paths_overlap(a.raw().as_str(), b.raw().as_str())
        || paths_overlap(a.location().as_str(), b.location().as_str())
}

/// Whether either path is a path-segment prefix of the other (symmetric).
pub(crate) fn paths_overlap(a: &str, b: &str) -> bool {
    is_path_prefix(a, b) || is_path_prefix(b, a)
}

/// Whether `prefix` is a path-segment prefix of `url`.
///
/// Delegates to [`object_store::path::Path::prefix_matches`], which compares on
/// path-segment boundaries so that a location registered for `s3://bucket/data`
/// does **not** match `s3://bucket/data-secret/x` (which a raw `starts_with`
/// would wrongly accept). Both inputs are full URL strings; `Path` parses them
/// the same way (splitting on `/`), so the comparison is consistent. A trailing
/// `/` is collapsed by `Path` parsing, so no separate normalization is needed.
pub(crate) fn is_path_prefix(url: &str, prefix: &str) -> bool {
    object_store::path::Path::from(url).prefix_matches(&object_store::path::Path::from(prefix))
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
    use super::{is_path_prefix, locations_overlap, paths_overlap};
    use crate::services::location::StorageLocationUrl;

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

    #[test]
    fn paths_overlap_is_symmetric_on_nesting() {
        // Descendant overlaps ancestor, in both argument orders.
        assert!(paths_overlap("s3://b/data", "s3://b/data/sub"));
        assert!(paths_overlap("s3://b/data/sub", "s3://b/data"));
        // Exact-equal paths overlap.
        assert!(paths_overlap("s3://b/data", "s3://b/data"));
    }

    #[test]
    fn paths_overlap_rejects_non_nested() {
        // Sibling that merely shares a textual prefix does not overlap.
        assert!(!paths_overlap("s3://b/data", "s3://b/data-secret"));
        // Disjoint paths do not overlap.
        assert!(!paths_overlap("s3://b/data", "s3://b/other"));
        // Different buckets never overlap.
        assert!(!paths_overlap("s3://b/data", "s3://c/data"));
    }

    #[test]
    fn locations_overlap_via_storage_location_url() {
        let parse = |s| StorageLocationUrl::parse(s).unwrap();
        assert!(locations_overlap(
            &parse("s3://b/data"),
            &parse("s3://b/data/sub")
        ));
        assert!(!locations_overlap(
            &parse("s3://b/data"),
            &parse("s3://b/data-secret")
        ));
    }
}
