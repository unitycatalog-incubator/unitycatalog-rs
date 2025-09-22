use uuid::Uuid;

use crate::models::ObjectLabel;

pub use name::*;

mod name;

/// Unique identifier for a resource.
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum ResourceRef {
    Uuid(Uuid),
    Name(ResourceName),
    /// Not referencing a specific resource.
    ///
    /// This is used to represent a wildcard in a policy
    /// which can be useful to check if a user can create
    /// or manage resources at a specific level.
    Undefined,
}

impl ResourceRef {
    pub fn is_undefined(&self) -> bool {
        matches!(self, Self::Undefined)
    }

    pub fn name(name: impl Into<ResourceName>) -> Self {
        Self::Name(name.into())
    }
}

impl std::fmt::Display for ResourceRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uuid(u) => write!(f, "{}", u.hyphenated()),
            Self::Name(name) => {
                write!(f, "{}", name)
            }
            Self::Undefined => write!(f, "*"),
        }
    }
}

impl From<Uuid> for ResourceRef {
    fn from(val: Uuid) -> Self {
        Self::Uuid(val)
    }
}

impl From<&Uuid> for ResourceRef {
    fn from(val: &Uuid) -> Self {
        Self::Uuid(*val)
    }
}

impl From<ResourceName> for ResourceRef {
    fn from(val: ResourceName) -> Self {
        Self::Name(val)
    }
}

/// Resource that a policy can authorize.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceIdent {
    Share(ResourceRef),
    Credential(ResourceRef),
    ExternalLocation(ResourceRef),
    Catalog(ResourceRef),
    Schema(ResourceRef),
    Table(ResourceRef),
    Recipient(ResourceRef),
    Column(ResourceRef),
}

impl ResourceIdent {
    pub fn label(&self) -> &ObjectLabel {
        self.as_ref()
    }

    pub fn reference(&self) -> &ResourceRef {
        self.as_ref()
    }

    pub fn share(name: impl Into<ResourceRef>) -> Self {
        Self::Share(name.into())
    }

    pub fn credential(name: impl Into<ResourceRef>) -> Self {
        Self::Credential(name.into())
    }

    pub fn catalog(name: impl Into<ResourceRef>) -> Self {
        Self::Catalog(name.into())
    }

    pub fn schema(name: impl Into<ResourceRef>) -> Self {
        Self::Schema(name.into())
    }

    pub fn table(name: impl Into<ResourceRef>) -> Self {
        Self::Table(name.into())
    }

    pub fn column(name: impl Into<ResourceRef>) -> Self {
        Self::Column(name.into())
    }

    pub fn external_location(name: impl Into<ResourceRef>) -> Self {
        Self::ExternalLocation(name.into())
    }

    pub fn recipient(name: impl Into<ResourceRef>) -> Self {
        Self::Recipient(name.into())
    }
}

impl std::fmt::Display for ResourceIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceIdent::Share(r) => write!(f, "share:{}", r),
            ResourceIdent::Credential(r) => write!(f, "credential:{}", r),
            ResourceIdent::ExternalLocation(r) => write!(f, "external_location:{}", r),
            ResourceIdent::Catalog(r) => write!(f, "catalog:{}", r),
            ResourceIdent::Schema(r) => write!(f, "schema:{}", r),
            ResourceIdent::Table(r) => write!(f, "table:{}", r),
            ResourceIdent::Recipient(r) => write!(f, "recipient:{}", r),
            ResourceIdent::Column(r) => write!(f, "column:{}", r),
        }
    }
}

impl AsRef<ResourceRef> for ResourceIdent {
    fn as_ref(&self) -> &ResourceRef {
        match self {
            ResourceIdent::Share(r) => r,
            ResourceIdent::Credential(r) => r,
            ResourceIdent::ExternalLocation(r) => r,
            ResourceIdent::Catalog(r) => r,
            ResourceIdent::Schema(r) => r,
            ResourceIdent::Table(r) => r,
            ResourceIdent::Recipient(r) => r,
            ResourceIdent::Column(r) => r,
        }
    }
}

impl AsRef<ObjectLabel> for ResourceIdent {
    fn as_ref(&self) -> &ObjectLabel {
        match self {
            ResourceIdent::Share(_) => &ObjectLabel::Share,
            ResourceIdent::Credential(_) => &ObjectLabel::CredentialInfo,
            ResourceIdent::ExternalLocation(_) => &ObjectLabel::ExternalLocationInfo,
            ResourceIdent::Catalog(_) => &ObjectLabel::CatalogInfo,
            ResourceIdent::Schema(_) => &ObjectLabel::SchemaInfo,
            ResourceIdent::Table(_) => &ObjectLabel::TableInfo,
            ResourceIdent::Recipient(_) => &ObjectLabel::Recipient,
            ResourceIdent::Column(_) => &ObjectLabel::ColumnInfo,
        }
    }
}

impl From<ResourceIdent> for ResourceRef {
    fn from(ident: ResourceIdent) -> Self {
        match ident {
            ResourceIdent::Share(r) => r,
            ResourceIdent::Credential(r) => r,
            ResourceIdent::ExternalLocation(r) => r,
            ResourceIdent::Catalog(r) => r,
            ResourceIdent::Schema(r) => r,
            ResourceIdent::Table(r) => r,
            ResourceIdent::Recipient(r) => r,
            ResourceIdent::Column(r) => r,
        }
    }
}

impl From<&ResourceIdent> for ResourceRef {
    fn from(ident: &ResourceIdent) -> Self {
        (ident as &dyn AsRef<ResourceRef>).as_ref().clone()
    }
}

impl From<&ResourceIdent> for ObjectLabel {
    fn from(ident: &ResourceIdent) -> Self {
        *(ident as &dyn AsRef<ObjectLabel>).as_ref()
    }
}

impl From<ResourceIdent> for ObjectLabel {
    fn from(ident: ResourceIdent) -> Self {
        (&ident).into()
    }
}

pub trait ResourceExt {
    /// Get the label for the resource
    fn resource_label(&self) -> &ObjectLabel;

    /// Get the name of the resource
    fn resource_name(&self) -> ResourceName;

    /// Get the reference for the resource
    ///
    /// Depending on the resource type, this may be a UUID or a name.
    /// If possible, implementations should prefer to use the UUID
    /// as it is globally unique. However not all resource-like objects
    /// have a UUID field, or the UUID field may be optional.
    fn resource_ref(&self) -> ResourceRef;

    /// Get the ident for the resource
    fn resource_ident(&self) -> ResourceIdent {
        self.resource_label().to_ident(self.resource_ref())
    }
}

impl<T: ResourceExt> From<&T> for ResourceIdent {
    fn from(resource: &T) -> Self {
        resource.resource_ident()
    }
}
