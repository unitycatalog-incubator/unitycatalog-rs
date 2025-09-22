use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use catalogs::v1::Catalog;
pub use credentials::v1::Credential;
pub use external_locations::v1::ExternalLocation;
pub use internal::resource::{ObjectLabel, Resource};
pub use object::Object;
pub use recipients::v1::Recipient;
pub use resources::*;
pub use schemas::v1::Schema;
pub use shares::v1::Share;
pub use tables::v1::{Column, Table};
pub use volumes::v1::{Volume, VolumeType};

mod object;
mod resources;

pub type PropertyMap = HashMap<String, serde_json::Value>;

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

pub mod volumes {
    pub mod v1 {
        include!("./gen/unitycatalog.volumes.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./gen/unitycatalog.volumes.v1.tonic.rs");
    }
}

pub(crate) mod internal {
    include!("./gen/unitycatalog.internal.rs");
}

impl ObjectLabel {
    pub fn to_ident(&self, id: impl Into<ResourceRef>) -> ResourceIdent {
        match self {
            ObjectLabel::Share => ResourceIdent::share(id),
            ObjectLabel::Credential => ResourceIdent::credential(id),
            ObjectLabel::Catalog => ResourceIdent::catalog(id),
            ObjectLabel::Schema => ResourceIdent::schema(id),
            ObjectLabel::Table => ResourceIdent::table(id),
            ObjectLabel::ExternalLocation => ResourceIdent::external_location(id),
            ObjectLabel::Recipient => ResourceIdent::recipient(id),
            ObjectLabel::Column => ResourceIdent::column(id),
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
