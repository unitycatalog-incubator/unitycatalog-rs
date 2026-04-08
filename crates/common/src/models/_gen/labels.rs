// @generated — do not edit by hand.
/// All resource types managed by Unity Catalog.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Debug, PartialEq)]
pub enum Resource {
    Catalog(super::catalogs::v1::Catalog),
    Column(super::tables::v1::Column),
    Credential(super::credentials::v1::Credential),
    ExternalLocation(super::external_locations::v1::ExternalLocation),
    Function(super::functions::v1::Function),
    Recipient(super::recipients::v1::Recipient),
    Schema(super::schemas::v1::Schema),
    Share(super::shares::v1::Share),
    Table(super::tables::v1::Table),
    Volume(super::volumes::v1::Volume),
}
/// Discriminant label for each resource type.
#[derive(
    ::strum::AsRefStr,
    ::strum::Display,
    ::strum::EnumIter,
    ::strum::EnumString,
    ::serde::Serialize,
    ::serde::Deserialize,
    Hash,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "sqlx", derive(::sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "object_label", rename_all = "snake_case")
)]
pub enum ObjectLabel {
    Catalog,
    Column,
    Credential,
    ExternalLocation,
    Function,
    Recipient,
    Schema,
    Share,
    Table,
    Volume,
}
impl Resource {
    /// Return the discriminant label for this resource.
    pub fn resource_label(&self) -> &ObjectLabel {
        match self {
            Resource::Catalog(_) => &ObjectLabel::Catalog,
            Resource::Column(_) => &ObjectLabel::Column,
            Resource::Credential(_) => &ObjectLabel::Credential,
            Resource::ExternalLocation(_) => &ObjectLabel::ExternalLocation,
            Resource::Function(_) => &ObjectLabel::Function,
            Resource::Recipient(_) => &ObjectLabel::Recipient,
            Resource::Schema(_) => &ObjectLabel::Schema,
            Resource::Share(_) => &ObjectLabel::Share,
            Resource::Table(_) => &ObjectLabel::Table,
            Resource::Volume(_) => &ObjectLabel::Volume,
        }
    }
}
impl From<super::catalogs::v1::Catalog> for Resource {
    fn from(v: super::catalogs::v1::Catalog) -> Self {
        Resource::Catalog(v)
    }
}
impl TryFrom<Resource> for super::catalogs::v1::Catalog {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Catalog(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Catalog)
            ))),
        }
    }
}
impl From<super::tables::v1::Column> for Resource {
    fn from(v: super::tables::v1::Column) -> Self {
        Resource::Column(v)
    }
}
impl TryFrom<Resource> for super::tables::v1::Column {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Column(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Column)
            ))),
        }
    }
}
impl From<super::credentials::v1::Credential> for Resource {
    fn from(v: super::credentials::v1::Credential) -> Self {
        Resource::Credential(v)
    }
}
impl TryFrom<Resource> for super::credentials::v1::Credential {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Credential(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Credential)
            ))),
        }
    }
}
impl From<super::external_locations::v1::ExternalLocation> for Resource {
    fn from(v: super::external_locations::v1::ExternalLocation) -> Self {
        Resource::ExternalLocation(v)
    }
}
impl TryFrom<Resource> for super::external_locations::v1::ExternalLocation {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::ExternalLocation(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(ExternalLocation)
            ))),
        }
    }
}
impl From<super::functions::v1::Function> for Resource {
    fn from(v: super::functions::v1::Function) -> Self {
        Resource::Function(v)
    }
}
impl TryFrom<Resource> for super::functions::v1::Function {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Function(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Function)
            ))),
        }
    }
}
impl From<super::recipients::v1::Recipient> for Resource {
    fn from(v: super::recipients::v1::Recipient) -> Self {
        Resource::Recipient(v)
    }
}
impl TryFrom<Resource> for super::recipients::v1::Recipient {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Recipient(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Recipient)
            ))),
        }
    }
}
impl From<super::schemas::v1::Schema> for Resource {
    fn from(v: super::schemas::v1::Schema) -> Self {
        Resource::Schema(v)
    }
}
impl TryFrom<Resource> for super::schemas::v1::Schema {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Schema(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Schema)
            ))),
        }
    }
}
impl From<super::shares::v1::Share> for Resource {
    fn from(v: super::shares::v1::Share) -> Self {
        Resource::Share(v)
    }
}
impl TryFrom<Resource> for super::shares::v1::Share {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Share(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Share)
            ))),
        }
    }
}
impl From<super::tables::v1::Table> for Resource {
    fn from(v: super::tables::v1::Table) -> Self {
        Resource::Table(v)
    }
}
impl TryFrom<Resource> for super::tables::v1::Table {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Table(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Table)
            ))),
        }
    }
}
impl From<super::volumes::v1::Volume> for Resource {
    fn from(v: super::volumes::v1::Volume) -> Self {
        Resource::Volume(v)
    }
}
impl TryFrom<Resource> for super::volumes::v1::Volume {
    type Error = crate::Error;
    fn try_from(r: Resource) -> Result<Self, Self::Error> {
        match r {
            Resource::Volume(v) => Ok(v),
            _ => Err(<crate::Error>::generic(concat!(
                "Resource is not a ",
                stringify!(Volume)
            ))),
        }
    }
}
