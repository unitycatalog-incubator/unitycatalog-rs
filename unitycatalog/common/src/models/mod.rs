use serde::Serialize;
use std::collections::HashMap;

use crate::Error;
use crate::resources::{ResourceIdent, ResourceName, ResourceRef};

pub use catalogs::v1::CatalogInfo;
pub use credentials::v1::CredentialInfo;
pub use external_locations::v1::ExternalLocationInfo;
pub use internal::resource::{ObjectLabel, Resource};
pub use profiles::v1::Profile;
pub use recipients::v1::RecipientInfo;
pub use schemas::v1::SchemaInfo;
pub use shares::v1::ShareInfo;
pub use sharing::v1::{Share, SharingSchema, SharingSchemaInfo, SharingTable};
pub use tables::v1::{ColumnInfo, TableInfo};

mod object;

pub type PropertyMap = HashMap<String, serde_json::Value>;

#[allow(clippy::empty_docs, clippy::large_enum_variant)]
pub mod sharing {
    pub mod v1 {
        include!("../gen/unitycatalog.sharing.v1.rs");
        #[cfg(feature = "grpc")]
        include!("../gen/unitycatalog.sharing.v1.tonic.rs");
    }
}

pub mod catalogs {
    pub mod v1 {
        include!("../gen/unitycatalog.catalogs.v1.rs");
        #[cfg(feature = "grpc")]
        include!("../gen/unitycatalog.catalogs.v1.tonic.rs");
    }
}

pub mod schemas {
    pub mod v1 {
        include!("../gen/unitycatalog.schemas.v1.rs");
        #[cfg(feature = "grpc")]
        include!("../gen/unitycatalog.schemas.v1.tonic.rs");
    }
}

pub mod tables {
    pub mod v1 {
        include!("../gen/unitycatalog.tables.v1.rs");
        #[cfg(feature = "grpc")]
        include!("../gen/unitycatalog.tables.v1.tonic.rs");
    }
}

pub mod shares {
    pub mod v1 {
        include!("../gen/unitycatalog.shares.v1.rs");
        #[cfg(feature = "grpc")]
        include!("../gen/unitycatalog.shares.v1.tonic.rs");
    }
}

pub mod recipients {
    pub mod v1 {
        include!("../gen/unitycatalog.recipients.v1.rs");
        #[cfg(feature = "grpc")]
        include!("../gen/unitycatalog.recipients.v1.tonic.rs");
    }
}

pub mod external_locations {
    pub mod v1 {
        include!("../gen/unitycatalog.external_locations.v1.rs");
        #[cfg(feature = "grpc")]
        include!("../gen/unitycatalog.external_locations.v1.tonic.rs");
    }
}

pub mod credentials {
    pub mod v1 {
        include!("../gen/unitycatalog.credentials.v1.rs");
        #[cfg(feature = "grpc")]
        include!("../gen/unitycatalog.credentials.v1.tonic.rs");
    }
}

pub mod profiles {
    pub mod v1 {
        include!("../gen/unitycatalog.profiles.v1.rs");
        // #[cfg(feature = "grpc")]
        // include!("../gen/unitycatalog.profiles.v1.tonic.rs");
    }
}

pub(crate) mod internal {
    include!("../gen/unitycatalog.internal.rs");
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
