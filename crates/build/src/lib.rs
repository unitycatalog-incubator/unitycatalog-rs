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

pub mod codegen;
pub mod error;
pub mod parsing;
pub mod utils;

/// Metadata extracted from a service method
#[derive(Debug, Clone)]
pub struct MethodMetadata {
    pub service_name: String,
    pub method_name: String,
    pub input_type: String,
    pub output_type: String,
    pub operation: Option<gnostic::openapi::v3::Operation>,
    pub http_rule: Option<google::api::HttpRule>,
    pub input_fields: Vec<MessageField>,
}

/// Information about a field in a protobuf message
#[derive(Debug, Clone)]
pub struct MessageField {
    pub name: String,
    pub field_type: String,
    pub optional: bool,
    pub repeated: bool,
    pub oneof_name: Option<String>,
    pub documentation: Option<String>,
}

impl MethodMetadata {
    /// Extract the operation ID from gnostic annotations
    pub fn operation_id(&self) -> Option<&str> {
        let operation = self.operation.as_ref()?;
        if operation.operation_id.is_empty() {
            None
        } else {
            Some(&operation.operation_id)
        }
    }

    /// Extract HTTP method and path from google.api.http annotations
    pub fn http_info(&self) -> Option<(String, String)> {
        let rule = self.http_rule.as_ref()?;

        // Extract HTTP method and path from the rule
        if let Some(pattern) = &rule.pattern {
            use crate::google::api::http_rule::Pattern;
            match pattern {
                Pattern::Get(path) => Some(("GET".to_string(), path.clone())),
                Pattern::Put(path) => Some(("PUT".to_string(), path.clone())),
                Pattern::Post(path) => Some(("POST".to_string(), path.clone())),
                Pattern::Delete(path) => Some(("DELETE".to_string(), path.clone())),
                Pattern::Patch(path) => Some(("PATCH".to_string(), path.clone())),
                Pattern::Custom(custom) => Some((custom.kind.clone(), custom.path.clone())),
            }
        } else {
            None
        }
    }

    /// Extract path parameters from HTTP path template
    pub fn path_parameters(&self) -> Vec<String> {
        if let Some((_, path)) = self.http_info() {
            utils::paths::extract_path_parameters(&path)
        } else {
            Vec::new()
        }
    }

    /// Determine the request type category (List, Create, Get, Update, Delete)
    pub fn request_type(&self) -> RequestType {
        utils::requests::classify_request_type(self.operation_id(), &self.method_name)
    }
}

/// Type of REST request operation
#[derive(Debug, Clone, PartialEq)]
pub enum RequestType {
    List,
    Create,
    Get,
    Update,
    Delete,
}

/// Collected metadata for code generation
#[derive(Debug)]
pub struct CodeGenMetadata {
    pub methods: Vec<MethodMetadata>,
    pub messages: std::collections::HashMap<String, MessageInfo>,
}

/// Information about a protobuf message
#[derive(Debug, Clone)]
pub struct MessageInfo {
    pub name: String,
    pub fields: Vec<MessageField>,
}

impl CodeGenMetadata {
    /// Group methods by service name
    pub fn services(&self) -> std::collections::HashMap<String, Vec<&MethodMetadata>> {
        let mut services = std::collections::HashMap::new();
        for method in &self.methods {
            services
                .entry(method.service_name.clone())
                .or_insert_with(Vec::new)
                .push(method);
        }
        services
    }

    /// Get message fields for a given type name
    pub fn get_message_fields(&self, type_name: &str) -> Vec<MessageField> {
        self.messages
            .get(type_name)
            .map(|msg| msg.fields.clone())
            .unwrap_or_default()
    }

    /// Get all methods that have complete REST metadata (operation_id + http_rule)
    pub fn complete_methods(&self) -> Vec<&MethodMetadata> {
        self.methods
            .iter()
            .filter(|m| m.operation_id().is_some() && m.http_info().is_some())
            .collect()
    }
}
