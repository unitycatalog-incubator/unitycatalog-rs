use std::sync::Arc;

use uuid::Uuid;

use crate::Result;
use crate::label::Label;
use crate::name::ResourceName;
use crate::object::{Association, Object};

/// Read-only interface for the object store.
#[async_trait::async_trait]
pub trait ObjectStoreReader<L: Label>: Send + Sync + 'static {
    /// Get an object by its UUID.
    async fn get(&self, id: &Uuid) -> Result<Object<L>>;

    /// Get an object by its label and name.
    async fn get_by_name(&self, label: L, name: &ResourceName) -> Result<Object<L>>;

    /// List objects of a given label, optionally scoped to a namespace prefix.
    ///
    /// Returns the matching objects and an optional continuation token.
    async fn list(
        &self,
        label: L,
        namespace: Option<&ResourceName>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<Object<L>>, Option<String>)>;
}

/// Read-write interface for the object store.
#[async_trait::async_trait]
pub trait ObjectStore<L: Label>: ObjectStoreReader<L> + Send + Sync + 'static {
    /// Create a new object. The store generates the `id`, `created_at`, and `updated_at` fields.
    async fn create(
        &self,
        label: L,
        name: &ResourceName,
        properties: Option<serde_json::Value>,
    ) -> Result<Object<L>>;

    /// Update an existing object's properties.
    async fn update(&self, id: &Uuid, properties: Option<serde_json::Value>) -> Result<Object<L>>;

    /// Delete an object and all its associations.
    async fn delete(&self, id: &Uuid) -> Result<()>;
}

/// Read-only interface for the association (edge) store.
#[async_trait::async_trait]
pub trait AssociationStoreReader<L: Label>: Send + Sync + 'static {
    /// List associations from a given source object with a specific edge label.
    ///
    /// Optionally filter by the target object's label.
    async fn list(
        &self,
        from_id: Uuid,
        label: &str,
        target_label: Option<L>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<Association<L>>, Option<String>)>;
}

/// Read-write interface for the association (edge) store.
///
/// Associations are directed edges between objects. Implementations should
/// automatically create inverse edges when adding an association.
#[async_trait::async_trait]
pub trait AssociationStore<L: Label>: AssociationStoreReader<L> + Send + Sync + 'static {
    /// Add an association between two objects.
    ///
    /// The implementation should also create the inverse association if the
    /// edge label has one.
    async fn add(
        &self,
        from_id: Uuid,
        to_id: Uuid,
        label: &str,
        properties: Option<serde_json::Value>,
    ) -> Result<()>;

    /// Remove an association between two objects.
    ///
    /// The implementation should also remove the inverse association.
    async fn remove(&self, from_id: Uuid, to_id: Uuid, label: &str) -> Result<()>;
}

// --- Blanket impls for Arc<T> ---

#[async_trait::async_trait]
impl<L: Label, T: ObjectStoreReader<L>> ObjectStoreReader<L> for Arc<T> {
    async fn get(&self, id: &Uuid) -> Result<Object<L>> {
        T::get(self, id).await
    }

    async fn get_by_name(&self, label: L, name: &ResourceName) -> Result<Object<L>> {
        T::get_by_name(self, label, name).await
    }

    async fn list(
        &self,
        label: L,
        namespace: Option<&ResourceName>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<Object<L>>, Option<String>)> {
        T::list(self, label, namespace, max_results, page_token).await
    }
}

#[async_trait::async_trait]
impl<L: Label, T: ObjectStore<L>> ObjectStore<L> for Arc<T> {
    async fn create(
        &self,
        label: L,
        name: &ResourceName,
        properties: Option<serde_json::Value>,
    ) -> Result<Object<L>> {
        T::create(self, label, name, properties).await
    }

    async fn update(&self, id: &Uuid, properties: Option<serde_json::Value>) -> Result<Object<L>> {
        T::update(self, id, properties).await
    }

    async fn delete(&self, id: &Uuid) -> Result<()> {
        T::delete(self, id).await
    }
}

#[async_trait::async_trait]
impl<L: Label, T: AssociationStoreReader<L>> AssociationStoreReader<L> for Arc<T> {
    async fn list(
        &self,
        from_id: Uuid,
        label: &str,
        target_label: Option<L>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<Association<L>>, Option<String>)> {
        T::list(self, from_id, label, target_label, max_results, page_token).await
    }
}

#[async_trait::async_trait]
impl<L: Label, T: AssociationStore<L>> AssociationStore<L> for Arc<T> {
    async fn add(
        &self,
        from_id: Uuid,
        to_id: Uuid,
        label: &str,
        properties: Option<serde_json::Value>,
    ) -> Result<()> {
        T::add(self, from_id, to_id, label, properties).await
    }

    async fn remove(&self, from_id: Uuid, to_id: Uuid, label: &str) -> Result<()> {
        T::remove(self, from_id, to_id, label).await
    }
}
