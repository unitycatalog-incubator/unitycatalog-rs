// @generated — do not edit by hand.
use std::collections::HashMap;
pub type PropertyMap = HashMap<String, serde_json::Value>;
pub mod open_sharing {
    pub mod v1 {
        include!("./../gen/open_sharing.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./../gen/open_sharing.v1.tonic.rs");
    }
}
