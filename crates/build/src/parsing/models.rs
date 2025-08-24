use std::collections::HashMap;

use super::http::{classify_request_type_from_http, extract_path_parameters};
use crate::{
    gnostic::openapi::v3::Operation,
    google::api::{FieldBehavior, HttpRule, ResourceDescriptor},
};

/// Collected metadata for code generation
#[derive(Debug)]
pub struct CodeGenMetadata {
    pub messages: HashMap<String, MessageInfo>,
    pub services: HashMap<String, ServiceInfo>,
}

impl CodeGenMetadata {
    /// Get message fields for a given type name
    pub(crate) fn get_message_fields(&self, type_name: &str) -> Vec<MessageField> {
        self.messages
            .get(type_name)
            .map(|msg| msg.fields.clone())
            .unwrap_or_default()
    }
}

/// Information about a protobuf message
#[derive(Debug, Clone)]
pub struct MessageInfo {
    pub name: String,
    pub fields: Vec<MessageField>,
    pub resource_descriptor: Option<ResourceDescriptor>,
    pub documentation: Option<String>,
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
    /// For oneof fields, contains the variants with their field names and types
    pub oneof_variants: Option<Vec<OneofVariant>>,
    /// Field behavior annotations from google.api.field_behavior
    pub field_behavior: Vec<FieldBehavior>,
}

/// Information about a variant in a oneof field
#[derive(Debug, Clone)]
pub struct OneofVariant {
    pub field_name: String,   // e.g., "azure_service_principal"
    pub variant_name: String, // e.g., "AzureServicePrincipal"
    pub rust_type: String,    // e.g., "AzureServicePrincipal"
    pub documentation: Option<String>,
}

/// Information about a protobuf service
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub documentation: Option<String>,
    pub methods: Vec<MethodMetadata>,
}

/// Metadata extracted from a service method
#[derive(Debug, Clone)]
pub struct MethodMetadata {
    pub service_name: String,
    pub method_name: String,
    pub input_type: String,
    pub output_type: String,
    pub operation: Option<Operation>,
    pub http_rule: HttpRule,
    pub input_fields: Vec<MessageField>,
    pub documentation: Option<String>,
}

impl MethodMetadata {
    /// Extract HTTP method and path from google.api.http annotations
    /// Get HTTP method and path from the rule if available
    pub fn http_info(&self) -> Option<(String, String)> {
        // Extract HTTP method and path from the rule
        if let Some(pattern) = &self.http_rule.pattern {
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
            extract_path_parameters(&path)
        } else {
            Vec::new()
        }
    }

    /// Determine the request type category (List, Create, Get, Update, Delete)
    pub fn request_type(&self) -> RequestType {
        classify_request_type_from_http(&self.http_rule, &self.method_name)
    }

    pub fn is_root_method(&self) -> bool {
        matches!(self.request_type(), RequestType::Get)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_based_method_structure() {
        let mut codegen_metadata = CodeGenMetadata {
            messages: HashMap::new(),
            services: HashMap::new(),
        };

        // Create a test service with methods
        let mut service_info = ServiceInfo {
            name: "TestService".to_string(),
            documentation: Some("Test service documentation".to_string()),
            methods: Vec::new(),
        };

        // Add a method to the service
        let http_rule = crate::google::api::HttpRule {
            pattern: Some(crate::google::api::http_rule::Pattern::Get(
                "/test".to_string(),
            )),
            ..Default::default()
        };

        let method = MethodMetadata {
            service_name: "TestService".to_string(),
            method_name: "TestMethod".to_string(),
            input_type: ".test.TestRequest".to_string(),
            output_type: ".test.TestResponse".to_string(),
            operation: None,
            http_rule: http_rule,
            input_fields: Vec::new(),
            documentation: Some("Test method documentation".to_string()),
        };
        service_info.methods.push(method);

        codegen_metadata
            .services
            .insert("TestService".to_string(), service_info);

        // Test that we can access the service and its methods directly
        let service = codegen_metadata.services.get("TestService").unwrap();
        assert_eq!(service.name, "TestService");
        assert_eq!(service.methods.len(), 1);
        assert_eq!(service.methods[0].method_name, "TestMethod");
    }
}
