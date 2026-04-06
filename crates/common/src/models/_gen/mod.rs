use std::collections::HashMap;
pub mod labels;
pub use catalogs::v1::Catalog;
pub use credentials::v1::Credential;
pub use external_locations::v1::ExternalLocation;
pub use functions::v1::Function;
pub use labels::{ObjectLabel, Resource};
pub use recipients::v1::Recipient;
pub use schemas::v1::Schema;
pub use shares::v1::Share;
pub use tables::v1::Column;
pub use tables::v1::Table;
pub use volumes::v1::Volume;
pub type PropertyMap = HashMap<String, serde_json::Value>;
pub mod catalogs {
    pub mod v1 {
        include!("./../gen/unitycatalog.catalogs.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./../gen/unitycatalog.catalogs.v1.tonic.rs");
    }
}
pub mod credentials {
    pub mod v1 {
        include!("./../gen/unitycatalog.credentials.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./../gen/unitycatalog.credentials.v1.tonic.rs");
    }
}
pub mod external_locations {
    pub mod v1 {
        include!("./../gen/unitycatalog.external_locations.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./../gen/unitycatalog.external_locations.v1.tonic.rs");
    }
}
pub mod functions {
    pub mod v1 {
        include!("./../gen/unitycatalog.functions.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./../gen/unitycatalog.functions.v1.tonic.rs");
    }
}
pub mod recipients {
    pub mod v1 {
        include!("./../gen/unitycatalog.recipients.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./../gen/unitycatalog.recipients.v1.tonic.rs");
    }
}
pub mod schemas {
    pub mod v1 {
        include!("./../gen/unitycatalog.schemas.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./../gen/unitycatalog.schemas.v1.tonic.rs");
    }
}
pub mod shares {
    pub mod v1 {
        include!("./../gen/unitycatalog.shares.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./../gen/unitycatalog.shares.v1.tonic.rs");
    }
}
pub mod tables {
    pub mod v1 {
        include!("./../gen/unitycatalog.tables.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./../gen/unitycatalog.tables.v1.tonic.rs");
    }
}
pub mod temporary_credentials {
    pub mod v1 {
        include!("./../gen/unitycatalog.temporary_credentials.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./../gen/unitycatalog.temporary_credentials.v1.tonic.rs");
    }
}
pub mod volumes {
    pub mod v1 {
        include!("./../gen/unitycatalog.volumes.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./../gen/unitycatalog.volumes.v1.tonic.rs");
    }
}
