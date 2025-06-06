use serde::{Deserialize, Serialize};
use unitycatalog_derive::object_conversions;
use uuid::Uuid;

use super::ExternalLocationInfo;
use super::tables::v1::TableSummary;
use crate::Error;
use crate::models::{
    CatalogInfo, ColumnInfo, CredentialInfo, ObjectLabel, RecipientInfo, Resource, SchemaInfo,
    ShareInfo, SharingSchemaInfo, SharingTable, TableInfo,
};
use crate::resources::{ResourceExt, ResourceName, ResourceRef};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Object {
    /// The globally unique identifier of the object.
    pub id: Uuid,

    /// The label / type of the object.
    pub label: ObjectLabel,

    /// The namespaced name of the object.
    pub name: ResourceName,

    /// The properties of the object.
    pub properties: Option<serde_json::Value>,

    /// The time when the object was created.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// The time when the object was last updated.
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl ResourceExt for Object {
    fn resource_label(&self) -> &ObjectLabel {
        &self.label
    }
    fn resource_name(&self) -> ResourceName {
        self.name.clone()
    }

    fn resource_ref(&self) -> ResourceRef {
        ResourceRef::Uuid(self.id)
    }
}

impl ResourceExt for Resource {
    fn resource_label(&self) -> &ObjectLabel {
        match self {
            Resource::ShareInfo(_) => &ObjectLabel::ShareInfo,
            Resource::SharingSchemaInfo(_) => &ObjectLabel::SharingSchemaInfo,
            Resource::SharingTable(_) => &ObjectLabel::SharingTable,
            Resource::CredentialInfo(_) => &ObjectLabel::CredentialInfo,
            Resource::CatalogInfo(_) => &ObjectLabel::CatalogInfo,
            Resource::SchemaInfo(_) => &ObjectLabel::SchemaInfo,
            Resource::TableInfo(_) => &ObjectLabel::TableInfo,
            Resource::ExternalLocationInfo(_) => &ObjectLabel::ExternalLocationInfo,
            Resource::RecipientInfo(_) => &ObjectLabel::RecipientInfo,
            Resource::ColumnInfo(_) => &ObjectLabel::ColumnInfo,
        }
    }

    fn resource_name(&self) -> ResourceName {
        match self {
            Resource::ShareInfo(obj) => obj.resource_name(),
            Resource::SharingSchemaInfo(obj) => obj.resource_name(),
            Resource::SharingTable(obj) => obj.resource_name(),
            Resource::CredentialInfo(obj) => obj.resource_name(),
            Resource::CatalogInfo(obj) => obj.resource_name(),
            Resource::SchemaInfo(obj) => obj.resource_name(),
            Resource::TableInfo(obj) => obj.resource_name(),
            Resource::ExternalLocationInfo(obj) => obj.resource_name(),
            Resource::RecipientInfo(obj) => obj.resource_name(),
            Resource::ColumnInfo(obj) => obj.resource_name(),
        }
    }

    fn resource_ref(&self) -> ResourceRef {
        match self {
            Resource::ShareInfo(obj) => obj.resource_ref(),
            Resource::SharingSchemaInfo(obj) => obj.resource_ref(),
            Resource::SharingTable(obj) => obj.resource_ref(),
            Resource::CredentialInfo(obj) => obj.resource_ref(),
            Resource::CatalogInfo(obj) => obj.resource_ref(),
            Resource::SchemaInfo(obj) => obj.resource_ref(),
            Resource::TableInfo(obj) => obj.resource_ref(),
            Resource::ExternalLocationInfo(obj) => obj.resource_ref(),
            Resource::RecipientInfo(obj) => obj.resource_ref(),
            Resource::ColumnInfo(obj) => obj.resource_ref(),
        }
    }
}

impl TryFrom<Resource> for Object {
    type Error = Error;

    fn try_from(resource: Resource) -> Result<Self, Self::Error> {
        match resource {
            Resource::ShareInfo(obj) => obj.try_into(),
            Resource::SharingSchemaInfo(obj) => obj.try_into(),
            Resource::SharingTable(obj) => obj.try_into(),
            Resource::CredentialInfo(obj) => obj.try_into(),
            Resource::CatalogInfo(obj) => obj.try_into(),
            Resource::SchemaInfo(obj) => obj.try_into(),
            Resource::TableInfo(obj) => obj.try_into(),
            Resource::ExternalLocationInfo(obj) => obj.try_into(),
            Resource::RecipientInfo(obj) => obj.try_into(),
            Resource::ColumnInfo(obj) => obj.try_into(),
        }
    }
}

impl TryFrom<Object> for Resource {
    type Error = Error;

    fn try_from(obj: Object) -> Result<Self, Self::Error> {
        match obj.label {
            ObjectLabel::ShareInfo => Ok(Resource::ShareInfo(obj.try_into()?)),
            ObjectLabel::SharingSchemaInfo => Ok(Resource::SharingSchemaInfo(obj.try_into()?)),
            ObjectLabel::SharingTable => Ok(Resource::SharingTable(obj.try_into()?)),
            ObjectLabel::CredentialInfo => Ok(Resource::CredentialInfo(obj.try_into()?)),
            ObjectLabel::CatalogInfo => Ok(Resource::CatalogInfo(obj.try_into()?)),
            ObjectLabel::SchemaInfo => Ok(Resource::SchemaInfo(obj.try_into()?)),
            ObjectLabel::TableInfo => Ok(Resource::TableInfo(obj.try_into()?)),
            ObjectLabel::ExternalLocationInfo => {
                Ok(Resource::ExternalLocationInfo(obj.try_into()?))
            }
            ObjectLabel::RecipientInfo => Ok(Resource::RecipientInfo(obj.try_into()?)),
            ObjectLabel::ColumnInfo => Ok(Resource::ColumnInfo(obj.try_into()?)),
        }
    }
}

impl From<TableInfo> for TableSummary {
    fn from(table: TableInfo) -> Self {
        TableSummary {
            table_type: table.table_type,
            full_name: table.full_name.unwrap_or_else(|| {
                format!(
                    "{}.{}.{}",
                    table.catalog_name, table.schema_name, table.name
                )
            }),
        }
    }
}

object_conversions!(
    ExternalLocationInfo, ObjectLabel::ExternalLocationInfo, external_location_id, [name], true;
    ShareInfo, ObjectLabel::ShareInfo, id, [name], true;
    SharingSchemaInfo, ObjectLabel::SharingSchemaInfo, id, [share, name];
    SharingTable, ObjectLabel::SharingTable, id, [share, schema, name], true;
    CatalogInfo, ObjectLabel::CatalogInfo, id, [name], true;
    SchemaInfo, ObjectLabel::SchemaInfo, schema_id, [catalog_name, name], true;
    TableInfo, ObjectLabel::TableInfo, table_id, [catalog_name, schema_name, name], true;
    ColumnInfo, ObjectLabel::ColumnInfo, column_id, [name], true;
    CredentialInfo, ObjectLabel::CredentialInfo, id, [name];
    RecipientInfo, ObjectLabel::RecipientInfo, id, [name], true;
);
