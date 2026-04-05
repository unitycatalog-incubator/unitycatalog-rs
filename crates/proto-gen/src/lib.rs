// Phase 1: parsing layer copied verbatim from crates/build.
// Dead-code and style lints are suppressed here; they will be addressed
// in later phases as proto-gen diverges from the build crate.
#![allow(dead_code)]
#![allow(clippy::manual_strip)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::only_used_in_recursion)]
#![allow(rustdoc::broken_intra_doc_links)]
#![allow(rustdoc::invalid_html_tags)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::doc_overindented_list_items)]
#![allow(clippy::doc_lazy_continuation)]

pub use error::*;

pub mod error;
pub mod parsing;
pub mod utils;

pub use parsing::{CodeGenMetadata, parse_file_descriptor_set, process_file_descriptor};

pub mod google {
    #[allow(unused)]
    pub mod api {
        include!("./gen/google.api.rs");
    }
}

pub(crate) mod gnostic {
    pub mod openapi {
        #[allow(unused)]
        pub mod v3 {
            include!("./gen/gnostic.openapi.v3.rs");
        }
    }
}
