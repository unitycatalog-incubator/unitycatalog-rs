use datafusion::datasource::object_store::ObjectStoreUrl;
use delta_kernel::object_store::ObjectStoreScheme;
use itertools::Itertools;
use unitycatalog_common::{Error, Result};
use url::Url;

pub enum StorageLocationScheme {
    ObjectStore(ObjectStoreScheme),
    Azurite,
}

impl AsRef<str> for StorageLocationScheme {
    fn as_ref(&self) -> &str {
        match self {
            Self::ObjectStore(ObjectStoreScheme::Local) => "file",
            Self::ObjectStore(ObjectStoreScheme::Memory) => "memory",
            Self::ObjectStore(ObjectStoreScheme::AmazonS3) => "s3",
            Self::ObjectStore(ObjectStoreScheme::GoogleCloudStorage) => "gs",
            Self::ObjectStore(ObjectStoreScheme::MicrosoftAzure) => "az",
            Self::ObjectStore(ObjectStoreScheme::Http) => "http",
            // NB: ObjectStoreScheme is non exhaustive, so we need to handle the unknown case.
            Self::ObjectStore(_) => "unknown",

            // Custom schemes
            Self::Azurite => "azurite",
        }
    }
}

impl StorageLocationScheme {
    pub fn parse(url: &Url) -> Result<Self> {
        match ObjectStoreScheme::parse(url) {
            Ok((ObjectStoreScheme::Http, _)) if is_azurite(url) => Ok(Self::Azurite),
            Ok((scheme, _)) => Ok(Self::ObjectStore(scheme)),
            Err(e) => {
                if is_azurite(url) {
                    Ok(Self::Azurite)
                } else {
                    Err(Error::invalid_argument(e.to_string()))
                }
            }
        }
    }
}

/// A URL representing a storage location.
///
/// This struct provides a cerntalized place to parse various URLs into more specific
/// service references along with the semantic information that can be extracted from
/// the URL.
pub struct StorageLocationUrl {
    /// The raw, unaltered URL.
    url: Url,
    store_url: ObjectStoreUrl,
    scheme: StorageLocationScheme,
    location: Url,
}

impl StorageLocationUrl {
    pub fn try_new(url: Url) -> Result<Self> {
        let (store_url, scheme, location) = get_store_url(&url)?;
        Ok(Self {
            url,
            store_url,
            scheme,
            location,
        })
    }

    pub fn parse(url: impl AsRef<str>) -> Result<Self> {
        let url = Url::parse(url.as_ref())?;
        Self::try_new(url)
    }

    pub fn raw(&self) -> &Url {
        &self.url
    }

    pub fn location(&self) -> &Url {
        &self.location
    }

    pub fn store_url(&self) -> &ObjectStoreUrl {
        &self.store_url
    }

    pub fn scheme(&self) -> &StorageLocationScheme {
        &self.scheme
    }
}

fn is_azurite(url: &Url) -> bool {
    // for now we assume that azurite is using default values.
    // since this is only for local development, this may be indefinitely the case.
    url.scheme() == "azurite"
        || (url.scheme() == "http"
            && matches!(url.host_str(), Some("localhost") | Some("127.0.0.1"))
            && url.port() == Some(10000))
}

fn get_store_url(url: &url::Url) -> Result<(ObjectStoreUrl, StorageLocationScheme, Url)> {
    let scheme = StorageLocationScheme::parse(url)?;
    let store_url = match &scheme {
        StorageLocationScheme::ObjectStore(_) => ObjectStoreUrl::parse(format!(
            "{}://{}",
            scheme.as_ref(),
            &url[url::Position::BeforeHost..url::Position::AfterPort]
        ))
        .map_err(|e| Error::Generic(e.to_string()))?,
        StorageLocationScheme::Azurite => {
            if url.scheme() != "azurite" {
                // split the path into 3 parts:
                // 1. account name
                // 2. container name
                // 3. path
                let parts = url.path().splitn(4, "/").collect::<Vec<_>>();
                if parts.len() != 4 {
                    return Err(Error::invalid_argument(format!(
                        "Invalid azurite path: {}",
                        url.path()
                    )));
                }
                ObjectStoreUrl::parse(format!("azurite://{}", parts[2]))
                    .map_err(|e| Error::Generic(e.to_string()))?
            } else {
                ObjectStoreUrl::parse(format!(
                    "{}://{}/",
                    url.scheme(),
                    &url[url::Position::BeforeHost..url::Position::AfterPort]
                ))
                .map_err(|e| Error::Generic(e.to_string()))?
            }
        }
    };
    let location = match &scheme {
        StorageLocationScheme::ObjectStore(_) => {
            let store: &Url = store_url.as_ref();
            let store = store.clone();

            store.join(url.path())?
        }
        StorageLocationScheme::Azurite if url.scheme() != "azurite" => {
            let path = url
                .path_segments()
                .ok_or_else(|| Error::invalid_argument("Invalid azurite path"))?
                .skip(2)
                .join("/");
            url::Url::parse(&format!("{}{}", store_url.as_str(), path))?
        }
        StorageLocationScheme::Azurite => url.clone(),
    };
    Ok((store_url, scheme, location))
}
