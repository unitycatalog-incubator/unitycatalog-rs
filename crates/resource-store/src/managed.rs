//! Registry-aware object store decorator that enforces field roles.
//!
//! [`ManagedObjectStore`] wraps an [`ObjectStore`] and optionally a [`SecretManager`]
//! to automatically:
//!
//! - Strip [`FieldRole::Identifier`] and [`FieldRole::Managed`] fields on create/update
//!   (the store is the source of truth for these)
//! - Route [`FieldRole::Sensitive`] fields to the [`SecretManager`]
//! - Inject Identifier and Managed fields back into properties on read
//! - Redact Sensitive fields on read (unless `get_with_secrets` is used)

use std::marker::PhantomData;
use std::sync::Arc;

use bytes::Bytes;
use uuid::Uuid;

use crate::label::Label;
use crate::name::ResourceName;
use crate::object::Object;
use crate::registry::{FieldRole, ResourceRegistry};
use crate::secrets::SecretManager;
use crate::store::{ObjectStore, ObjectStoreReader};
use crate::{Error, Result};

/// A registry-aware object store that enforces field roles.
///
/// Wraps an inner [`ObjectStore`] and uses a [`ResourceRegistry`] to determine
/// how each field should be handled during CRUD operations.
///
/// When a [`SecretManager`] is provided, sensitive fields (marked with
/// `debug_redact = true` in proto definitions) are automatically separated
/// into encrypted secret storage.
pub struct ManagedObjectStore<L: Label, S, M = NoSecrets> {
    inner: S,
    secrets: M,
    registry: Arc<ResourceRegistry<L>>,
    _label: PhantomData<L>,
}

/// Placeholder type for when no [`SecretManager`] is configured.
pub struct NoSecrets;

impl<L: Label, S: ObjectStore<L>> ManagedObjectStore<L, S, NoSecrets> {
    /// Create a managed store without secret management.
    ///
    /// Sensitive fields will be stripped from properties but not stored anywhere.
    pub fn new(inner: S, registry: ResourceRegistry<L>) -> Self {
        Self {
            inner,
            secrets: NoSecrets,
            registry: Arc::new(registry),
            _label: PhantomData,
        }
    }
}

impl<L: Label, S: ObjectStore<L>, M: SecretManager> ManagedObjectStore<L, S, M> {
    /// Create a managed store with secret management.
    pub fn with_secrets(inner: S, secrets: M, registry: ResourceRegistry<L>) -> Self {
        Self {
            inner,
            secrets,
            registry: Arc::new(registry),
            _label: PhantomData,
        }
    }
}

impl<L: Label, S, M> ManagedObjectStore<L, S, M> {
    /// Strip fields that should not be stored in properties on create/update.
    ///
    /// Returns (stripped_properties, sensitive_fields_map).
    fn strip_fields(
        &self,
        label: L,
        properties: Option<serde_json::Value>,
    ) -> (
        Option<serde_json::Value>,
        Option<serde_json::Map<String, serde_json::Value>>,
    ) {
        let Some(serde_json::Value::Object(mut map)) = properties else {
            return (properties, None);
        };

        let Some(descriptor) = self.registry.get(label) else {
            return (Some(serde_json::Value::Object(map)), None);
        };

        let mut sensitive_map = serde_json::Map::new();

        for field in descriptor.fields.iter() {
            match field.role {
                FieldRole::Identifier | FieldRole::Managed => {
                    // Remove — store manages these
                    map.remove(field.name);
                }
                FieldRole::Sensitive => {
                    // Extract — will be routed to secret store
                    if let Some(value) = map.remove(field.name) {
                        sensitive_map.insert(field.name.to_string(), value);
                    }
                }
                FieldRole::Data => {
                    // Keep as-is
                }
            }
        }

        let sensitive = if sensitive_map.is_empty() {
            None
        } else {
            Some(sensitive_map)
        };

        (Some(serde_json::Value::Object(map)), sensitive)
    }

    /// Inject Identifier and Managed fields from Object metadata into properties.
    fn inject_fields(&self, object: &mut Object<L>) {
        let Some(descriptor) = self.registry.get(object.label) else {
            return;
        };

        let map = object
            .properties
            .get_or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));

        let Some(map) = map.as_object_mut() else {
            return;
        };

        for field in descriptor.fields.iter() {
            match field.role {
                FieldRole::Identifier => {
                    map.insert(
                        field.name.to_string(),
                        serde_json::Value::String(object.id.to_string()),
                    );
                }
                FieldRole::Managed => {
                    match field.name {
                        "created_at" => {
                            map.insert(
                                field.name.to_string(),
                                serde_json::Value::String(object.created_at.to_rfc3339()),
                            );
                        }
                        "updated_at" => {
                            if let Some(updated) = object.updated_at {
                                map.insert(
                                    field.name.to_string(),
                                    serde_json::Value::String(updated.to_rfc3339()),
                                );
                            }
                        }
                        _ => {
                            // Other managed fields (created_by, updated_by) — leave as-is
                            // if already present, don't overwrite
                        }
                    }
                }
                FieldRole::Sensitive => {
                    // Redact: ensure sensitive fields are null in the response
                    map.remove(field.name);
                }
                FieldRole::Data => {
                    // Already in properties
                }
            }
        }
    }
}

// --- ObjectStoreReader impl ---

#[async_trait::async_trait]
impl<L: Label, S: ObjectStoreReader<L>, M: Send + Sync + 'static> ObjectStoreReader<L>
    for ManagedObjectStore<L, S, M>
{
    async fn get(&self, id: &Uuid) -> Result<Object<L>> {
        let mut object = self.inner.get(id).await?;
        self.inject_fields(&mut object);
        Ok(object)
    }

    async fn get_by_name(&self, label: L, name: &ResourceName) -> Result<Object<L>> {
        let mut object = self.inner.get_by_name(label, name).await?;
        self.inject_fields(&mut object);
        Ok(object)
    }

    async fn list(
        &self,
        label: L,
        namespace: Option<&ResourceName>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<Object<L>>, Option<String>)> {
        let (mut objects, token) = self
            .inner
            .list(label, namespace, max_results, page_token)
            .await?;
        for object in &mut objects {
            self.inject_fields(object);
        }
        Ok((objects, token))
    }
}

// --- ObjectStore impl (without secrets) ---

#[async_trait::async_trait]
impl<L: Label, S: ObjectStore<L>> ObjectStore<L> for ManagedObjectStore<L, S, NoSecrets> {
    async fn create(
        &self,
        label: L,
        name: &ResourceName,
        properties: Option<serde_json::Value>,
    ) -> Result<Object<L>> {
        let (stripped, _sensitive) = self.strip_fields(label, properties);
        let mut object = self.inner.create(label, name, stripped).await?;
        self.inject_fields(&mut object);
        Ok(object)
    }

    async fn update(&self, id: &Uuid, properties: Option<serde_json::Value>) -> Result<Object<L>> {
        // We need the label to look up the descriptor. Fetch the object first.
        let existing = self.inner.get(id).await?;
        let (stripped, _sensitive) = self.strip_fields(existing.label, properties);
        let mut object = self.inner.update(id, stripped).await?;
        self.inject_fields(&mut object);
        Ok(object)
    }

    async fn delete(&self, id: &Uuid) -> Result<()> {
        self.inner.delete(id).await
    }
}

// --- ObjectStore impl (with secrets) ---

#[async_trait::async_trait]
impl<L: Label, S: ObjectStore<L>, M: SecretManager> ObjectStore<L> for ManagedObjectStore<L, S, M> {
    async fn create(
        &self,
        label: L,
        name: &ResourceName,
        properties: Option<serde_json::Value>,
    ) -> Result<Object<L>> {
        let (stripped, sensitive) = self.strip_fields(label, properties);

        // Store sensitive fields in secret manager
        if let Some(sensitive_map) = sensitive {
            let secret_bytes = Bytes::from(serde_json::to_vec(&serde_json::Value::Object(
                sensitive_map,
            ))?);
            self.secrets
                .create_secret(&name.to_string(), secret_bytes)
                .await?;
        }

        let mut object = self.inner.create(label, name, stripped).await?;
        self.inject_fields(&mut object);
        Ok(object)
    }

    async fn update(&self, id: &Uuid, properties: Option<serde_json::Value>) -> Result<Object<L>> {
        let existing = self.inner.get(id).await?;
        let (stripped, sensitive) = self.strip_fields(existing.label, properties);

        // Update sensitive fields in secret manager
        if let Some(sensitive_map) = sensitive {
            let secret_bytes = Bytes::from(serde_json::to_vec(&serde_json::Value::Object(
                sensitive_map,
            ))?);
            let secret_name = existing.name.to_string();
            // Try update; if the secret doesn't exist yet, create it
            match self
                .secrets
                .update_secret(&secret_name, secret_bytes.clone())
                .await
            {
                Ok(_) => {}
                Err(Error::NotFound) => {
                    self.secrets
                        .create_secret(&secret_name, secret_bytes)
                        .await?;
                }
                Err(e) => return Err(e),
            }
        }

        let mut object = self.inner.update(id, stripped).await?;
        self.inject_fields(&mut object);
        Ok(object)
    }

    async fn delete(&self, id: &Uuid) -> Result<()> {
        // Delete secret first (best-effort — may not exist)
        let object = self.inner.get(id).await?;
        if self.registry.has_sensitive_fields(object.label) {
            let secret_name = object.name.to_string();
            match self.secrets.delete_secret(&secret_name).await {
                Ok(()) | Err(Error::NotFound) => {}
                Err(e) => return Err(e),
            }
        }
        self.inner.delete(id).await
    }
}

impl<L: Label, S: ObjectStore<L>, M: SecretManager> ManagedObjectStore<L, S, M> {
    /// Get an object with its sensitive fields populated from the secret store.
    ///
    /// This is intended for internal use (e.g., credential vending) where the
    /// caller needs access to the full credential data.
    pub async fn get_with_secrets(&self, id: &Uuid) -> Result<Object<L>> {
        let mut object = self.inner.get(id).await?;
        self.inject_fields(&mut object);

        // Join sensitive fields from secret store
        if self.registry.has_sensitive_fields(object.label) {
            let secret_name = object.name.to_string();
            match self.secrets.get_secret(&secret_name).await {
                Ok((_version, secret_bytes)) => {
                    let sensitive: serde_json::Value = serde_json::from_slice(&secret_bytes)?;
                    if let (Some(props), serde_json::Value::Object(secret_map)) =
                        (object.properties.as_mut(), sensitive)
                    {
                        if let Some(props_map) = props.as_object_mut() {
                            for (key, value) in secret_map {
                                props_map.insert(key, value);
                            }
                        }
                    }
                }
                Err(Error::NotFound) => {
                    // No secrets stored — that's fine
                }
                Err(e) => return Err(e),
            }
        }

        Ok(object)
    }
}
