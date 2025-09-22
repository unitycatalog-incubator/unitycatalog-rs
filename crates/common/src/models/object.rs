use serde::{Deserialize, Serialize};
use unitycatalog_derive::object_conversions;
use uuid::Uuid;

use super::ExternalLocationInfo;
use super::tables::v1::TableSummary;
use crate::Error;
use crate::models::{
    CatalogInfo, Column, Credential, ObjectLabel, Recipient, Resource, ResourceExt, ResourceName,
    ResourceRef, SchemaInfo, Share, Table,
};

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
            Resource::Share(_) => &ObjectLabel::Share,
            Resource::Credential(_) => &ObjectLabel::Credential,
            Resource::CatalogInfo(_) => &ObjectLabel::CatalogInfo,
            Resource::SchemaInfo(_) => &ObjectLabel::SchemaInfo,
            Resource::Table(_) => &ObjectLabel::Table,
            Resource::ExternalLocationInfo(_) => &ObjectLabel::ExternalLocationInfo,
            Resource::Recipient(_) => &ObjectLabel::Recipient,
            Resource::Column(_) => &ObjectLabel::Column,
        }
    }

    fn resource_name(&self) -> ResourceName {
        match self {
            Resource::Share(obj) => obj.resource_name(),
            Resource::Credential(obj) => obj.resource_name(),
            Resource::CatalogInfo(obj) => obj.resource_name(),
            Resource::SchemaInfo(obj) => obj.resource_name(),
            Resource::Table(obj) => obj.resource_name(),
            Resource::ExternalLocationInfo(obj) => obj.resource_name(),
            Resource::Recipient(obj) => obj.resource_name(),
            Resource::Column(obj) => obj.resource_name(),
        }
    }

    fn resource_ref(&self) -> ResourceRef {
        match self {
            Resource::Share(obj) => obj.resource_ref(),
            Resource::Credential(obj) => obj.resource_ref(),
            Resource::CatalogInfo(obj) => obj.resource_ref(),
            Resource::SchemaInfo(obj) => obj.resource_ref(),
            Resource::Table(obj) => obj.resource_ref(),
            Resource::ExternalLocationInfo(obj) => obj.resource_ref(),
            Resource::Recipient(obj) => obj.resource_ref(),
            Resource::Column(obj) => obj.resource_ref(),
        }
    }
}

impl TryFrom<Resource> for Object {
    type Error = Error;

    fn try_from(resource: Resource) -> Result<Self, Self::Error> {
        match resource {
            Resource::Share(obj) => obj.try_into(),
            Resource::Credential(obj) => obj.try_into(),
            Resource::CatalogInfo(obj) => obj.try_into(),
            Resource::SchemaInfo(obj) => obj.try_into(),
            Resource::Table(obj) => obj.try_into(),
            Resource::ExternalLocationInfo(obj) => obj.try_into(),
            Resource::Recipient(obj) => obj.try_into(),
            Resource::Column(obj) => obj.try_into(),
        }
    }
}

impl TryFrom<Object> for Resource {
    type Error = Error;

    fn try_from(obj: Object) -> Result<Self, Self::Error> {
        match obj.label {
            ObjectLabel::Share => Ok(Resource::Share(obj.try_into()?)),
            ObjectLabel::Credential => Ok(Resource::Credential(obj.try_into()?)),
            ObjectLabel::CatalogInfo => Ok(Resource::CatalogInfo(obj.try_into()?)),
            ObjectLabel::SchemaInfo => Ok(Resource::SchemaInfo(obj.try_into()?)),
            ObjectLabel::Table => Ok(Resource::Table(obj.try_into()?)),
            ObjectLabel::ExternalLocationInfo => {
                Ok(Resource::ExternalLocationInfo(obj.try_into()?))
            }
            ObjectLabel::Recipient => Ok(Resource::Recipient(obj.try_into()?)),
            ObjectLabel::Column => Ok(Resource::Column(obj.try_into()?)),
        }
    }
}

impl From<Table> for TableSummary {
    fn from(table: Table) -> Self {
        TableSummary {
            table_type: table.table_type,
            full_name: table.full_name,
        }
    }
}

object_conversions!(
    ExternalLocationInfo, ObjectLabel::ExternalLocationInfo, external_location_id, [name], true;
    Share, ObjectLabel::Share, id, [name], true;
    CatalogInfo, ObjectLabel::CatalogInfo, id, [name], true;
    SchemaInfo, ObjectLabel::SchemaInfo, schema_id, [catalog_name, name], true;
    Table, ObjectLabel::Table, table_id, [catalog_name, schema_name, name], true;
    Column, ObjectLabel::Column, column_id, [name], true;
    Credential, ObjectLabel::Credential, id, [name], true;
    Recipient, ObjectLabel::Recipient, id, [name], true;
);
