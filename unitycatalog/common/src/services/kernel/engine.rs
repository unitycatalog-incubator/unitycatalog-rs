use std::sync::Arc;

use dashmap::DashMap;
use delta_kernel::DeltaResult;
use delta_kernel::engine::default::executor::TaskExecutor;
use delta_kernel::engine::default::{DefaultEngine, ObjectStoreRegistry};
use itertools::Itertools;
use object_store::DynObjectStore;
use object_store::azure::MicrosoftAzureBuilder;
use url::Url;

use crate::api::CredentialsHandler;
use crate::models::credentials::v1::credential_info::Credential;
use crate::models::credentials::v1::{
    AzureServicePrincipal, AzureStorageKey, GetCredentialRequest,
};
use crate::models::external_locations::v1::ExternalLocationInfo;
use crate::resources::ResourceStore;
use crate::{Error, Result};

pub trait RegistryHandler: ResourceStore + CredentialsHandler {}
impl<T: ResourceStore + CredentialsHandler> RegistryHandler for T {}

pub fn get_engine<E: TaskExecutor>(
    handler: Arc<dyn RegistryHandler>,
    task_executor: Arc<E>,
) -> DeltaResult<Arc<DefaultEngine<E>>> {
    let registry = Arc::new(UnityStorageRegistry::new(task_executor.clone(), handler));
    Ok(Arc::new(DefaultEngine::new(registry, task_executor)))
}

pub struct UnityStorageRegistry<E: TaskExecutor> {
    executor: Arc<E>,
    handler: Arc<dyn RegistryHandler>,
    registry: Arc<DashMap<String, Arc<DynObjectStore>>>,
}

impl<E: TaskExecutor> std::fmt::Debug for UnityStorageRegistry<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("UnityStorageRegistry")
            .field("registry", &self.registry)
            .finish()
    }
}

impl<E: TaskExecutor> UnityStorageRegistry<E> {
    pub fn new(executor: Arc<E>, handler: Arc<dyn RegistryHandler>) -> Self {
        Self {
            executor,
            handler,
            registry: Arc::new(DashMap::new()),
        }
    }
}

impl<E: TaskExecutor> ObjectStoreRegistry for UnityStorageRegistry<E> {
    fn get_store(&self, url: &Url) -> DeltaResult<(Arc<DynObjectStore>, bool)> {
        self.registry
            // TODO(packre): using this url kei may be too simplistic since UC allows
            // registering nested external locations. We need to find a way to find
            // the most specific location while still being able to cache based on the url.
            .entry(get_url_key(url))
            .or_try_insert_with(move || {
                let url = url.clone();
                self.executor
                    .block_on(get_object_store(url, self.handler.clone()))
            })
            .map(|e| (e.value().clone(), true))
            .map_err(delta_kernel::Error::generic)
    }
}

/// Get the key of a url for object store registration.
/// The credential info will be removed
fn get_url_key(url: &Url) -> String {
    format!(
        "{}://{}",
        url.scheme(),
        &url[url::Position::BeforeHost..url::Position::AfterPort],
    )
}

async fn get_object_store(
    location: Url,
    handler: Arc<dyn RegistryHandler>,
) -> Result<Arc<DynObjectStore>> {
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
        .filter(|l| location.as_str().starts_with(&l.url))
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
        | Credential::AzureManagedIdentity(_) => get_azure_store(&location, cred),
    }
}

fn get_azure_store(location: &Url, credential: Credential) -> Result<Arc<DynObjectStore>> {
    let mut builder = MicrosoftAzureBuilder::new().with_url(location.as_str());
    // check if the location is localhost to determine if we are running the emulator
    if matches!(location.host_str(), Some("localhost") | Some("127.0.0.1")) {
        let url_err = || {
            Error::invalid_argument(
                "emulator URLs must encode the account and container name in the path",
            )
        };
        let mut segments = location.path_segments().ok_or_else(url_err)?;
        let account_name = segments.next().ok_or_else(url_err)?.to_string();
        let container_name = segments.next().ok_or_else(url_err)?.to_string();
        builder = builder
            .with_use_emulator(true)
            .with_account(account_name)
            .with_container_name(container_name);
    }
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
