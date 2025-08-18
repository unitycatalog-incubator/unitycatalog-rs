use std::sync::Arc;

use cloud_client::CloudClient;
use object_store::aws::AmazonS3Builder;
use object_store::azure::MicrosoftAzureBuilder;
use object_store::gcp::GoogleCloudStorageBuilder;
use object_store::path::Path;
use object_store::prefix::PrefixStore;
use object_store::{ObjectStore, Result};
use unitycatalog_client::{
    PathOperation, TableOperation, TableReference, TemporaryCredentialClient,
};
use unitycatalog_common::temporary_credentials::v1::TemporaryCredential;
use url::Url;

use crate::credential::{SecurableRef, as_aws, as_azure, as_gcp, new_aws, new_azure, new_gcp};
pub use crate::error::Error;

mod credential;
mod error;

#[derive(Debug, Clone, Default)]
pub struct UnityObjectStoreFactoryBuilder {
    /// URI of the Unity Catalog instance.
    uri: Option<String>,
    /// Token for authentication.
    token: Option<String>,
    /// Allow unauthenticated access.
    allow_unauthenticated: bool,
    /// aws region to assign
    aws_region: Option<String>,
}

/// Builder for creating a UnityObjectStoreFactory.
impl UnityObjectStoreFactoryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the URI of the Unity Catalog instance.
    ///
    /// This is required for creating a UnityObjectStoreFactory.
    pub fn with_uri<T>(mut self, uri: impl Into<Option<T>>) -> Self
    where
        T: ToString,
    {
        self.uri = uri.into().map(|t| t.to_string());
        self
    }

    /// Set the [access token] for the Unity Catalog instance.
    ///
    /// [access token]: https://docs.databricks.com/aws/en/dev-tools/auth/pat
    pub fn with_token<T>(mut self, token: impl Into<Option<T>>) -> Self
    where
        T: ToString,
    {
        self.token = token.into().map(|t| t.to_string());
        self
    }

    /// Allow creating an unauthenticated client
    ///
    /// This should only ever be used fro testing / development as there should
    /// probably not be any unauthenticated UC servers out there.
    pub fn with_allow_unauthenticated(mut self, allow_unauthenticated: bool) -> Self {
        self.allow_unauthenticated = allow_unauthenticated;
        self
    }

    /// Set the AWS region for the factory.
    ///
    /// Right now we cannot infer the AWS region from the vended credential return.
    /// If no region is supplied the default region from object_store will be used.
    /// Currently, the default region is set to "us-east-1".
    pub fn with_aws_region<T>(mut self, aws_region: impl Into<Option<T>>) -> Self
    where
        T: ToString,
    {
        self.aws_region = aws_region.into().map(|t| t.to_string());
        self
    }

    pub async fn build(self) -> Result<UnityObjectStoreFactory> {
        let url = if let Some(uri) = self.uri {
            url::Url::parse(&uri).map_err(Error::from)?
        } else {
            return Err(
                Error::invalid_config("insufficient options to create cloud client").into(),
            );
        };

        let cloud_client = if let Some(token) = self.token {
            CloudClient::new_with_token(token)
        } else if self.allow_unauthenticated {
            CloudClient::new_unauthenticated()
        } else {
            return Err(Error::invalid_config("Failed to find credential for cloud client").into());
        };

        let client = TemporaryCredentialClient::new_with_url(cloud_client, url);
        Ok(UnityObjectStoreFactory {
            client,
            aws_region: self.aws_region,
        })
    }
}

/// Result returned by [`UnityObjectStoreFactory`] methods.
///
/// This wraps the base store with some contextual information.
#[derive(Clone)]
pub struct UCFactoryStore {
    /// The base store pointing at the root of the URL returned by unity catalog
    ///
    /// The credential may not be valid for all paths within that store.
    pub store: Arc<dyn ObjectStore>,
    /// The full url of the credential.
    ///
    /// All paths that are children of this URL accessible by the credential.
    pub url: Url,
    /// The path within the store that is accessible by the credential.
    pub path: Path,
}

impl UCFactoryStore {
    /// The base store pointing at the root of the URL returned by unity catalog
    ///
    /// The credential may not be valid for all paths within that store.
    pub fn store(&self) -> Arc<dyn ObjectStore> {
        self.store.clone()
    }

    /// Store prefixed with the path of the credential.
    ///
    /// All paths paths within this store are accessible by the credential.
    pub fn prefixed_store(&self) -> Arc<dyn ObjectStore> {
        Arc::new(PrefixStore::new(self.store.clone(), self.path.clone()))
    }
}

/// Factory for creating [`ObjectStore`] instances backed by temporary unity credentials
#[derive(Clone)]
pub struct UnityObjectStoreFactory {
    client: TemporaryCredentialClient,
    aws_region: Option<String>,
}

impl UnityObjectStoreFactory {
    /// Get an [`ObjectStore`] to read from or write to a table.
    ///
    /// The table can be referenced by its name or its id. If the name is provided,
    /// the client will first resolve the table by its name to obtain the table id.
    /// This is a once per table operation.
    ///
    /// [`String`] and [`Uuid`] both can be converted to [`TableReference`].
    /// If a string is provided, it will always be assumed to be the name.
    ///
    /// ## Parameters
    /// - `table`: The table to read from or write to.
    /// - `operation`: The operation to perform on the table.
    ///
    /// ## Returns
    /// A [`Result`] containing an [`Arc<dyn ObjectStore>`] to read from or write to the table.
    pub async fn for_table(
        &self,
        table: impl Into<TableReference>,
        operation: TableOperation,
    ) -> Result<UCFactoryStore> {
        let credential = self
            .client
            .temporary_table_credential(table, operation)
            .await
            .map_err(Error::from)?;
        let securable = SecurableRef::Table(credential.1, operation);
        let url = Url::parse(&credential.0.url).map_err(Error::from)?;
        let path = Path::from_url_path(url.path())?;
        let store = self.to_store(credential.0, securable).await?;
        Ok(UCFactoryStore { store, url, path })
    }

    /// Get an [`ObjectStore`] to read from or write to a path.
    ///
    /// ## Parameters
    /// - `path`: The path to read from or write to.
    /// - `operation`: The operation to perform on the path.
    ///
    /// ## Returns
    /// A [`Result`] containing an [`Arc<dyn ObjectStore>`] to read from or write to the path.
    pub async fn for_path(&self, path: &Url, operation: PathOperation) -> Result<UCFactoryStore> {
        let credential = self
            .client
            .temporary_path_credential(path.clone(), operation, false)
            .await
            .map_err(Error::from)?;
        let securable = SecurableRef::Path(credential.1, operation);
        let url = Url::parse(&credential.0.url).map_err(Error::from)?;
        let path = Path::from_url_path(url.path())?;
        let store = self.to_store(credential.0, securable).await?;
        Ok(UCFactoryStore { store, url, path })
    }

    async fn to_store(
        &self,
        credential: TemporaryCredential,
        securable: SecurableRef,
    ) -> Result<Arc<dyn ObjectStore>> {
        if as_azure(&credential).is_ok() {
            let provider = new_azure(self.client.clone(), &credential, securable).await?;
            let url = Url::parse(&credential.url).map_err(Error::from)?;
            let store = MicrosoftAzureBuilder::new()
                .with_url(url.to_string())
                .with_credentials(Arc::new(provider))
                .build()?;
            return Ok(Arc::new(store));
        }

        if as_aws(&credential).is_ok() {
            let provider = new_aws(self.client.clone(), &credential, securable).await?;
            let url = Url::parse(&credential.url).map_err(Error::from)?;
            let mut store = AmazonS3Builder::new()
                .with_url(url.to_string())
                .with_credentials(Arc::new(provider));
            if let Some(region) = &self.aws_region {
                store = store.with_region(region);
            }
            let store = store.build()?;
            return Ok(Arc::new(store));
        }

        if as_gcp(&credential).is_ok() {
            let provider = new_gcp(self.client.clone(), &credential, securable).await?;
            let url = Url::parse(&credential.url).map_err(Error::from)?;
            let store = GoogleCloudStorageBuilder::new()
                .with_url(url.to_string())
                .with_credentials(Arc::new(provider))
                .build()?;
            return Ok(Arc::new(store));
        }

        Err(
            Error::InvalidCredential("Failed to match credential with storage type".to_string())
                .into(),
        )
    }
}
