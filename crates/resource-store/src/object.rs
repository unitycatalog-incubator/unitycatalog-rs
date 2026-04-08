use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::label::Label;
use crate::name::ResourceName;

/// A generic resource object stored in the graph.
///
/// This is the untyped interchange format between the store backend and typed
/// resource wrappers. The `label` discriminant identifies the resource type,
/// and `properties` holds the serialized resource fields as JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Object<L: Label> {
    /// The globally unique identifier of the object.
    pub id: Uuid,

    /// The label / type of the object.
    pub label: L,

    /// The namespaced name of the object.
    pub name: ResourceName,

    /// The properties of the object (serialized resource fields).
    pub properties: Option<serde_json::Value>,

    /// The time when the object was created.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// The time when the object was last updated.
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// An association (edge) between two objects in the graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Association<L: Label> {
    /// Unique identifier for this association.
    pub id: Uuid,

    /// The source object identifier.
    pub from_id: Uuid,

    /// The label of this association edge.
    pub label: String,

    /// The target object identifier.
    pub to_id: Uuid,

    /// The label of the target object (denormalized for efficient queries).
    pub to_label: L,

    /// Optional properties on the association.
    pub properties: Option<serde_json::Value>,

    /// The time when the association was created.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// The time when the association was last updated.
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
