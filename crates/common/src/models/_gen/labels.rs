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
use crate::Error;
use crate::models::object::Object;
use crate::models::resources::{ResourceExt, ResourceIdent, ResourceName, ResourceRef};
::unitycatalog_derive::object_conversions!(
    super::catalogs::v1::Catalog, ObjectLabel::Catalog, id, [name], true;
    super::tables::v1::Column, ObjectLabel::Column, column_id, [name], true;
    super::credentials::v1::Credential, ObjectLabel::Credential, id, [name], true;
    super::external_locations::v1::ExternalLocation, ObjectLabel::ExternalLocation,
    external_location_id, [name], true; super::functions::v1::Function,
    ObjectLabel::Function, function_id, [catalog_name, schema_name, name], true;
    super::recipients::v1::Recipient, ObjectLabel::Recipient, id, [name], true;
    super::schemas::v1::Schema, ObjectLabel::Schema, schema_id, [catalog_name, name],
    true; super::shares::v1::Share, ObjectLabel::Share, id, [name], true;
    super::tables::v1::Table, ObjectLabel::Table, table_id, [catalog_name, schema_name,
    name], true; super::volumes::v1::Volume, ObjectLabel::Volume, volume_id,
    [catalog_name, schema_name, name], false
);
impl super::catalogs::v1::Catalog {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        self.name.clone()
    }
}
impl super::tables::v1::Column {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        self.name.clone()
    }
}
impl super::credentials::v1::Credential {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        self.name.clone()
    }
}
impl super::external_locations::v1::ExternalLocation {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        self.name.clone()
    }
}
impl super::functions::v1::Function {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        format!("{}.{}.{}", self.catalog_name, self.schema_name, self.name)
    }
}
impl super::recipients::v1::Recipient {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        self.name.clone()
    }
}
impl super::schemas::v1::Schema {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.catalog_name, self.name)
    }
}
impl super::shares::v1::Share {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        self.name.clone()
    }
}
impl super::tables::v1::Table {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        format!("{}.{}.{}", self.catalog_name, self.schema_name, self.name)
    }
}
impl super::volumes::v1::Volume {
    /// Returns the fully-qualified dot-separated name computed from component fields.
    pub fn qualified_name(&self) -> String {
        format!("{}.{}.{}", self.catalog_name, self.schema_name, self.name)
    }
}
impl ::unitycatalog_resource_store::Label for ObjectLabel {
    fn as_str(&self) -> &str {
        self.as_ref()
    }
}
/// Static resource type descriptors derived from proto annotations.
///
/// Each entry describes a resource type's fields (with roles: data, identifier,
/// sensitive, managed), hierarchical name components, and parent relationship.
///
/// Use [`::unitycatalog_resource_store::ResourceRegistry::from_static`] to build
/// a runtime registry from this data.
pub static RESOURCE_DESCRIPTORS: &[::unitycatalog_resource_store::ResourceTypeDescriptor<
    ObjectLabel,
>] = &[
    ::unitycatalog_resource_store::ResourceTypeDescriptor {
        label: ObjectLabel::Catalog,
        fields: &[
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "id",
                role: ::unitycatalog_resource_store::FieldRole::Identifier,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "properties",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "storage_root",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "provider_name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "share_name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "catalog_type",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "browse_only",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
        ],
        path_names: &["name"],
        parent_label: None,
    },
    ::unitycatalog_resource_store::ResourceTypeDescriptor {
        label: ObjectLabel::Column,
        fields: &[
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "type_text",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "type_json",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "position",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "type_name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "type_precision",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "type_scale",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "type_interval_type",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "nullable",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "partition_index",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "column_id",
                role: ::unitycatalog_resource_store::FieldRole::Identifier,
            },
        ],
        path_names: &["name"],
        parent_label: None,
    },
    ::unitycatalog_resource_store::ResourceTypeDescriptor {
        label: ObjectLabel::Credential,
        fields: &[
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "id",
                role: ::unitycatalog_resource_store::FieldRole::Identifier,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "purpose",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "read_only",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "used_for_managed_storage",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "full_name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "azure_service_principal",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "azure_managed_identity",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "azure_storage_key",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "aws_iam_role",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "databricks_gcp_service_account",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
        ],
        path_names: &["name"],
        parent_label: None,
    },
    ::unitycatalog_resource_store::ResourceTypeDescriptor {
        label: ObjectLabel::ExternalLocation,
        fields: &[
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "url",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "credential_name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "read_only",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "credential_id",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "browse_only",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "external_location_id",
                role: ::unitycatalog_resource_store::FieldRole::Identifier,
            },
        ],
        path_names: &["name"],
        parent_label: None,
    },
    ::unitycatalog_resource_store::ResourceTypeDescriptor {
        label: ObjectLabel::Function,
        fields: &[
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "catalog_name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "schema_name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "full_name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "data_type",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "full_data_type",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "input_params",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "return_params",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "routine_body_language",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "routine_definition",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "routine_dependencies",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "parameter_style",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "is_deterministic",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "sql_data_access",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "is_null_call",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "security_type",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "specific_name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "routine_body",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "properties",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "function_id",
                role: ::unitycatalog_resource_store::FieldRole::Identifier,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
        ],
        path_names: &["catalog_name", "schema_name", "name"],
        parent_label: Some(ObjectLabel::Schema),
    },
    ::unitycatalog_resource_store::ResourceTypeDescriptor {
        label: ObjectLabel::Recipient,
        fields: &[
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "id",
                role: ::unitycatalog_resource_store::FieldRole::Identifier,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "authentication_type",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "properties",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "tokens",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
        ],
        path_names: &["name"],
        parent_label: None,
    },
    ::unitycatalog_resource_store::ResourceTypeDescriptor {
        label: ObjectLabel::Schema,
        fields: &[
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "catalog_name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "full_name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "properties",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "schema_id",
                role: ::unitycatalog_resource_store::FieldRole::Identifier,
            },
        ],
        path_names: &["catalog_name", "name"],
        parent_label: Some(ObjectLabel::Catalog),
    },
    ::unitycatalog_resource_store::ResourceTypeDescriptor {
        label: ObjectLabel::Share,
        fields: &[
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "id",
                role: ::unitycatalog_resource_store::FieldRole::Identifier,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "objects",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "storage_location",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "storage_root",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
        ],
        path_names: &["name"],
        parent_label: None,
    },
    ::unitycatalog_resource_store::ResourceTypeDescriptor {
        label: ObjectLabel::Table,
        fields: &[
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "catalog_name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "schema_name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "table_type",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "data_source_format",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "columns",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "storage_location",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "properties",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "storage_credential_name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "full_name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "deleted_at",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "table_id",
                role: ::unitycatalog_resource_store::FieldRole::Identifier,
            },
        ],
        path_names: &["catalog_name", "schema_name", "name"],
        parent_label: Some(ObjectLabel::Schema),
    },
    ::unitycatalog_resource_store::ResourceTypeDescriptor {
        label: ObjectLabel::Volume,
        fields: &[
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "catalog_name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "schema_name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "full_name",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "storage_location",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "volume_id",
                role: ::unitycatalog_resource_store::FieldRole::Identifier,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "volume_type",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "owner",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "comment",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "created_at",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "created_by",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "updated_at",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "updated_by",
                role: ::unitycatalog_resource_store::FieldRole::Managed,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "browse_only",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
            ::unitycatalog_resource_store::ResourceFieldDescriptor {
                name: "metastore_id",
                role: ::unitycatalog_resource_store::FieldRole::Data,
            },
        ],
        path_names: &["catalog_name", "schema_name", "name"],
        parent_label: Some(ObjectLabel::Schema),
    },
];
