//! A generic, TAO-inspired resource store.
//!
//! This crate provides the core abstractions for a graph-based resource store:
//!
//! - **`Object<L>`** — A generic resource node identified by UUID, label, and hierarchical name,
//!   with properties stored as JSON.
//! - **`Association<L>`** — A directed edge between two objects with a label and optional properties.
//! - **`ObjectStore<L>`** / **`AssociationStore<L>`** — Async traits for CRUD + graph operations.
//! - **`SecretManager`** — Trait for encrypted storage of sensitive field values.
//! - **`ResourceRegistry`** — Runtime metadata registry describing field roles (data, identifier,
//!   sensitive, managed) per resource type, derived from proto annotations.
//!
//! The store is generic over `L: Label`, a type-safe discriminant for resource types
//! (typically generated from protobuf `google.api.resource` annotations).
//!
//! ## Architecture
//!
//! ```text
//! Proto definitions
//!     │  google.api.resource, field_behavior, debug_redact
//!     ▼
//! proto-gen → ObjectLabel (impl Label), RESOURCE_DESCRIPTORS
//!     │
//!     ▼
//! ObjectStore<L> + AssociationStore<L>
//!     │
//!     ├── ManagedObjectStore<L, S, M>  ← field role enforcement (Phase 3)
//!     ├── AssociationManager<L, S, A>  ← auto parent-child (Phase 4)
//!     └── RoutingStore<L>              ← per-label backend dispatch (Phase 4)
//! ```

pub mod error;
pub mod label;
pub mod name;
pub mod object;
pub mod reference;
pub mod registry;
pub mod secrets;
pub mod store;

// Re-exports for convenience.
pub use error::{Error, Result};
pub use label::Label;
pub use name::{EMPTY_RESOURCE_NAME, ResourceName};
pub use object::{Association, Object};
pub use reference::ResourceRef;
pub use registry::{FieldRole, ResourceFieldDescriptor, ResourceRegistry, ResourceTypeDescriptor};
pub use secrets::{ProvidesSecretManager, SecretManager};
pub use store::{AssociationStore, AssociationStoreReader, ObjectStore, ObjectStoreReader};
