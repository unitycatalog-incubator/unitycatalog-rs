use std::sync::Arc;

use bytes::Bytes;

use crate::Result;

/// A trait for managing secrets.
///
/// All sensitive data that needs to be stored in the system should be stored as a secret.
///
/// The secret manager is responsible for persisting and retrieving the secret value. Values are
/// expected to be encrypted at rest by the implementation (e.g. via envelope encryption); callers
/// pass and receive plaintext.
///
/// Secrets are identified by a unique name and hold a single current value. Writing a secret is
/// idempotent: [`put_secret`](SecretManager::put_secret) creates the secret if it does not exist
/// and replaces the value otherwise.
#[async_trait::async_trait]
pub trait SecretManager: Send + Sync + 'static {
    /// Store the secret value for the given name, replacing any existing value.
    async fn put_secret(&self, secret_name: &str, secret_value: Bytes) -> Result<()>;

    /// Get the current secret value for the given name.
    ///
    /// Returns an error if the secret does not exist.
    async fn get_secret(&self, secret_name: &str) -> Result<Bytes>;

    /// Delete the secret with the given name.
    ///
    /// Returns an error if the secret does not exist.
    async fn delete_secret(&self, secret_name: &str) -> Result<()>;
}

/// Auxiliary trait for implementing [`SecretManager`] for structs that contain a [`SecretManager`].
pub trait ProvidesSecretManager: Send + Sync + 'static {
    fn secret_manager(&self) -> &dyn SecretManager;
}

#[async_trait::async_trait]
impl<T: SecretManager> SecretManager for Arc<T> {
    async fn put_secret(&self, secret_name: &str, secret_value: Bytes) -> Result<()> {
        T::put_secret(self, secret_name, secret_value).await
    }

    async fn get_secret(&self, secret_name: &str) -> Result<Bytes> {
        T::get_secret(self, secret_name).await
    }

    async fn delete_secret(&self, secret_name: &str) -> Result<()> {
        T::delete_secret(self, secret_name).await
    }
}

#[async_trait::async_trait]
impl<T: ProvidesSecretManager> SecretManager for T {
    async fn put_secret(&self, secret_name: &str, secret_value: Bytes) -> Result<()> {
        self.secret_manager()
            .put_secret(secret_name, secret_value)
            .await
    }

    async fn get_secret(&self, secret_name: &str) -> Result<Bytes> {
        self.secret_manager().get_secret(secret_name).await
    }

    async fn delete_secret(&self, secret_name: &str) -> Result<()> {
        self.secret_manager().delete_secret(secret_name).await
    }
}
