use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::resources::{ResourceIdent, ResourceRef};
use crate::{Error, Result};

pub use catalogs::v1::CatalogInfo;
pub use credentials::v1::CredentialInfo;
pub use external_locations::v1::ExternalLocationInfo;
pub use internal::resource::{ObjectLabel, Resource};
pub use object::Object;
pub use recipients::v1::RecipientInfo;
pub use schemas::v1::SchemaInfo;
pub use shares::v1::ShareInfo;
pub use sharing::v1::{Share, SharingSchema, SharingSchemaInfo, SharingTable};
pub use tables::v1::{ColumnInfo, TableInfo};

mod object;

pub type PropertyMap = HashMap<String, serde_json::Value>;

pub mod google {
    #[allow(deprecated)]
    pub mod protobuf {
        include!("./gen/google.protobuf.rs");
    }
}

#[allow(clippy::empty_docs, clippy::large_enum_variant)]
pub mod sharing {
    pub mod v1 {
        include!("./gen/unitycatalog.sharing.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./gen/unitycatalog.sharing.v1.tonic.rs");
    }
}

pub mod catalogs {
    pub mod v1 {
        include!("./gen/unitycatalog.catalogs.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./gen/unitycatalog.catalogs.v1.tonic.rs");
    }
}

pub mod schemas {
    pub mod v1 {
        include!("./gen/unitycatalog.schemas.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./gen/unitycatalog.schemas.v1.tonic.rs");
    }
}

pub mod tables {
    pub mod v1 {
        include!("./gen/unitycatalog.tables.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./gen/unitycatalog.tables.v1.tonic.rs");
    }
}

pub mod shares {
    pub mod v1 {
        include!("./gen/unitycatalog.shares.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./gen/unitycatalog.shares.v1.tonic.rs");
    }
}

pub mod recipients {
    pub mod v1 {
        include!("./gen/unitycatalog.recipients.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./gen/unitycatalog.recipients.v1.tonic.rs");
    }
}

pub mod external_locations {
    pub mod v1 {
        include!("./gen/unitycatalog.external_locations.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./gen/unitycatalog.external_locations.v1.tonic.rs");
    }
}

pub mod credentials {
    pub mod v1 {
        include!("./gen/unitycatalog.credentials.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./gen/unitycatalog.credentials.v1.tonic.rs");
    }
}

pub mod temporary_credentials {
    pub mod v1 {
        include!("./gen/unitycatalog.temporary_credentials.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./gen/unitycatalog.temporary_credentials.v1.tonic.rs");
    }
}

pub(crate) mod internal {
    include!("./gen/unitycatalog.internal.rs");
}

impl ObjectLabel {
    pub fn to_ident(&self, id: impl Into<ResourceRef>) -> ResourceIdent {
        match self {
            ObjectLabel::ShareInfo => ResourceIdent::share(id),
            ObjectLabel::SharingSchemaInfo => ResourceIdent::schema(id),
            ObjectLabel::SharingTable => ResourceIdent::sharing_table(id),
            ObjectLabel::CredentialInfo => ResourceIdent::credential(id),
            ObjectLabel::CatalogInfo => ResourceIdent::catalog(id),
            ObjectLabel::SchemaInfo => ResourceIdent::schema(id),
            ObjectLabel::TableInfo => ResourceIdent::table(id),
            ObjectLabel::ExternalLocationInfo => ResourceIdent::external_location(id),
            ObjectLabel::RecipientInfo => ResourceIdent::recipient(id),
            ObjectLabel::ColumnInfo => ResourceIdent::column(id),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Hash, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "association_label", rename_all = "snake_case")
)]
pub enum AssociationLabel {
    OwnedBy,
    OwnerOf,
    DependsOn,
    DependencyOf,
    ParentOf,
    ChildOf,
    HasPart,
    PartOf,
    References,
    ReferencedBy,
}

impl AssociationLabel {
    /// Get the inverse of the association label.
    ///
    /// Associations may be bidirectional, either symmetric or asymmetric.
    /// Symmetric types are their own inverse. Asymmetric types have a distinct inverse.
    pub fn inverse(&self) -> Option<Self> {
        match self {
            AssociationLabel::HasPart => Some(AssociationLabel::PartOf),
            AssociationLabel::PartOf => Some(AssociationLabel::HasPart),
            AssociationLabel::DependsOn => Some(AssociationLabel::DependencyOf),
            AssociationLabel::DependencyOf => Some(AssociationLabel::DependsOn),
            AssociationLabel::ParentOf => Some(AssociationLabel::ChildOf),
            AssociationLabel::ChildOf => Some(AssociationLabel::ParentOf),
            AssociationLabel::References => Some(AssociationLabel::ReferencedBy),
            AssociationLabel::ReferencedBy => Some(AssociationLabel::References),
            AssociationLabel::OwnedBy => Some(AssociationLabel::OwnerOf),
            AssociationLabel::OwnerOf => Some(AssociationLabel::OwnedBy),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error_code: String,
    pub message: String,
}

/// Conversions from more specific types to reduced info sharing API types
impl TryFrom<Resource> for Share {
    type Error = Error;

    fn try_from(resource: Resource) -> Result<Self, Self::Error> {
        let info = ShareInfo::try_from(resource)?;
        Ok(Share {
            id: info.id,
            name: info.name,
        })
    }
}

impl TryFrom<Resource> for SharingSchema {
    type Error = Error;

    fn try_from(resource: Resource) -> Result<Self, Self::Error> {
        let info = SharingSchemaInfo::try_from(resource)?;
        Ok(SharingSchema {
            share: info.share,
            name: info.name,
            id: Some(info.id),
        })
    }
}
