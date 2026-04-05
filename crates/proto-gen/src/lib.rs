pub use error::*;

pub mod analysis;
pub mod codegen;
pub mod error;
pub mod output;
pub mod parsing;
pub mod utils;

pub use codegen::{
    BindingsConfig, CodeGenConfig, CodeGenOutput, GeneratedCode, ResourceEnumConfig, generate_code,
};

pub use analysis::{
    BodyField, GenerationPlan, ManagedResource, MethodPlan, PathParam, QueryParam, RequestParam,
    RequestType, ServicePlan, analyze_metadata, extract_managed_resources, split_body_fields,
};
// Note: MethodPlanner is pub(crate) — it is an internal helper, not part of the public API.
pub use parsing::{CodeGenMetadata, parse_file_descriptor_set, process_file_descriptor};

pub mod google {
    pub mod api {
        #![allow(unused)]
        #![allow(clippy::doc_overindented_list_items)]
        #![allow(clippy::doc_lazy_continuation)]
        include!("./gen/google.api.rs");
    }
}

pub(crate) mod gnostic {
    pub mod openapi {
        pub mod v3 {
            #![allow(unused)]
            #![allow(clippy::large_enum_variant)]
            include!("./gen/gnostic.openapi.v3.rs");
        }
    }
}
