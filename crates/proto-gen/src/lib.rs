//! Code generation from protobuf descriptors.
//!
//! `proto-gen` turns compiled protobuf descriptor bytes into Rust REST-API glue code
//! (Axum handlers, HTTP clients, PyO3 bindings, NAPI bindings, and TypeScript clients).
//!
//! ## Pipeline overview
//!
//! ```text
//! descriptor bytes
//!      │
//!      ▼
//! parse_file_descriptor_set   ← parsing module
//!      │  CodeGenMetadata
//!      ▼
//! analyze_metadata            ← analysis module
//!      │  GenerationPlan
//!      ▼
//! generate_code               ← codegen module
//!      │  writes files to CodeGenOutput dirs
//!      ▼
//! (generated Rust / Python / TypeScript source files)
//! ```
//!
//! ## Quick-start example
//!
//! ```rust,no_run
//! use std::fs;
//! use proto_gen::{CodeGenConfig, CodeGenOutput, generate_code, parse_file_descriptor_set};
//! use proto_gen::{ResourceEnumConfig, BindingsConfig};
//! use protobuf::Message;
//! use protobuf::descriptor::FileDescriptorSet;
//!
//! // 1. Load descriptor bytes (produced by `buf build`)
//! let bytes = fs::read("descriptors.bin").unwrap();
//! let fds = FileDescriptorSet::parse_from_bytes(&bytes).unwrap();
//!
//! // 2. Parse into metadata
//! let metadata = parse_file_descriptor_set(&fds).unwrap();
//!
//! // 3. Configure outputs
//! let output = CodeGenOutput {
//!     common: "/tmp/out/common".into(),
//!     models_gen: Some("/tmp/out/models".into()),
//!     server: Some("/tmp/out/server".into()),
//!     client: Some("/tmp/out/client".into()),
//!     python: None,
//!     node: None,
//!     node_ts: None,
//!     python_typings_filename: "my_client.pyi".into(),
//! };
//!
//! let config = CodeGenConfig {
//!     context_type_path: "crate::api::RequestContext".into(),
//!     result_type_path: "crate::Result".into(),
//!     models_path_template: "my_crate::models::{service}::v1".into(),
//!     models_path_crate_template: "crate::models::{service}::v1".into(),
//!     output,
//!     resource_enum: Some(ResourceEnumConfig {
//!         package_prefix: ".mypackage.".into(),
//!         super_levels: 2,
//!     }),
//!     bindings: None,
//! };
//!
//! // 4. Optionally validate before running
//! config.validate().unwrap();
//!
//! // 5. Generate
//! generate_code(&metadata, &config).unwrap();
//! ```

pub use error::*;

pub mod analysis;
pub mod codegen;
pub mod error;
pub mod openapi_enrich;
pub mod output;
pub mod parsing;
pub mod utils;

pub use codegen::{
    BindingsConfig, CodeGenConfig, CodeGenOutput, GeneratedCode, ResourceEnumConfig, generate_code,
};

pub use analysis::{
    BodyField, GenerationPlan, ManagedResource, MethodPlan, PathParam, QueryParam, RequestParam,
    RequestType, ServicePlan, SkippedMethod, analyze_metadata, extract_managed_resources,
    split_body_fields,
};
// Note: MethodPlanner is pub(crate) — it is an internal helper, not part of the public API.
pub use openapi_enrich::run as enrich_openapi;
pub use parsing::http::HttpPattern;
pub use parsing::types::{BaseType, RenderContext, UnifiedType};
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
