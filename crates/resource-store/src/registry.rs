use std::collections::HashMap;

use crate::Label;

/// The role of a field within a resource type.
///
/// Derived from proto annotations:
/// - `IDENTIFIER` → [`FieldRole::Identifier`]
/// - `debug_redact = true` → [`FieldRole::Sensitive`]
/// - `OUTPUT_ONLY` + known name → [`FieldRole::Managed`]
/// - Everything else → [`FieldRole::Data`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldRole {
    /// Normal data field — stored in properties JSON.
    Data,

    /// Identifier field — managed by store, maps to `Object.id`.
    /// Derived from `google.api.field_behavior = IDENTIFIER`.
    Identifier,

    /// Sensitive field — routed to the secret store.
    /// Derived from `debug_redact = true` on the proto field.
    Sensitive,

    /// Store-managed field — `created_at`, `updated_at`, etc.
    /// Derived from `OUTPUT_ONLY` + known field names.
    Managed,
}

/// Descriptor for a single field within a resource type.
///
/// Generated from proto annotations by the code generation pipeline.
#[derive(Debug, Clone, Copy)]
pub struct ResourceFieldDescriptor {
    /// The proto field name (e.g., `"name"`, `"client_secret"`).
    pub name: &'static str,

    /// The role of this field in the store.
    pub role: FieldRole,
}

/// Descriptor for a resource type, derived from proto annotations.
///
/// Generated as a static constant by the code generation pipeline.
/// Generic over `L` so the label field can be a typed enum (e.g., `ObjectLabel`)
/// rather than a plain string.
#[derive(Debug, Clone)]
pub struct ResourceTypeDescriptor<L: Label> {
    /// The label for this resource type.
    pub label: L,

    /// Field descriptors for this resource type.
    pub fields: &'static [ResourceFieldDescriptor],

    /// Ordered path components for building a [`ResourceName`](crate::ResourceName),
    /// e.g., `["catalog_name", "schema_name", "name"]` for Table.
    pub path_names: &'static [&'static str],

    /// Parent resource type label, inferred from the path_names hierarchy.
    /// `None` for top-level resources (e.g., Catalog).
    pub parent_label: Option<L>,
}

/// Runtime registry for resource type descriptors.
///
/// Provides efficient lookup of field roles, sensitive fields, and hierarchy
/// information by resource label.
#[derive(Debug, Clone)]
pub struct ResourceRegistry<L: Label> {
    by_label: HashMap<L, &'static ResourceTypeDescriptor<L>>,
}

impl<L: Label> ResourceRegistry<L> {
    /// Create a registry from a static slice of descriptors.
    pub fn from_static(descriptors: &'static [ResourceTypeDescriptor<L>]) -> Self {
        let by_label = descriptors.iter().map(|d| (d.label, d)).collect();
        Self { by_label }
    }

    /// Get the descriptor for a resource type by label.
    pub fn get(&self, label: L) -> Option<&&'static ResourceTypeDescriptor<L>> {
        self.by_label.get(&label)
    }

    /// Returns the names of sensitive fields for the given resource label.
    pub fn sensitive_field_names(&self, label: L) -> Vec<&'static str> {
        self.by_label
            .get(&label)
            .map(|d| {
                d.fields
                    .iter()
                    .filter(|f| f.role == FieldRole::Sensitive)
                    .map(|f| f.name)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns true if the resource type has any sensitive fields.
    pub fn has_sensitive_fields(&self, label: L) -> bool {
        self.by_label
            .get(&label)
            .is_some_and(|d| d.fields.iter().any(|f| f.role == FieldRole::Sensitive))
    }

    /// Returns the names of managed fields for the given resource label.
    pub fn managed_field_names(&self, label: L) -> Vec<&'static str> {
        self.by_label
            .get(&label)
            .map(|d| {
                d.fields
                    .iter()
                    .filter(|f| f.role == FieldRole::Managed)
                    .map(|f| f.name)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns the identifier field name for the given resource label, if any.
    pub fn identifier_field_name(&self, label: L) -> Option<&'static str> {
        self.by_label.get(&label).and_then(|d| {
            d.fields
                .iter()
                .find(|f| f.role == FieldRole::Identifier)
                .map(|f| f.name)
        })
    }

    /// Returns the parent label for a resource type, if any.
    pub fn parent_label(&self, label: L) -> Option<L> {
        self.by_label.get(&label).and_then(|d| d.parent_label)
    }

    /// Returns the path names for a resource type.
    pub fn path_names(&self, label: L) -> Option<&'static [&'static str]> {
        self.by_label.get(&label).map(|d| d.path_names)
    }

    /// Returns all registered labels.
    pub fn labels(&self) -> impl Iterator<Item = &L> {
        self.by_label.keys()
    }
}
