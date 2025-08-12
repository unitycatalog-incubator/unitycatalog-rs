pub mod google {
    pub mod api {
        include!("./gen/google.api.rs");
    }
}

pub mod gnostic {
    pub mod openapi {
        pub mod v3 {
            include!("./gen/gnostic.openapi.v3.rs");
        }
    }
}

/// Metadata extracted from a service method
#[derive(Debug, Clone)]
pub struct MethodMetadata {
    pub service_name: String,
    pub method_name: String,
    pub input_type: String,
    pub output_type: String,
    pub operation: Option<gnostic::openapi::v3::Operation>,
    pub http_rule: Option<google::api::HttpRule>,
}

/// Collected metadata for code generation
pub struct CodeGenMetadata {
    pub methods: Vec<MethodMetadata>,
}
