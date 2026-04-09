use uuid::Uuid;

use crate::ResourceName;

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
