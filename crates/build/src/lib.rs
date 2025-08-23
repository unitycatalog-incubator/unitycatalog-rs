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
    /// For oneof fields, contains the variants with their field names and types
    pub oneof_variants: Option<Vec<OneofVariant>>,
}

/// Information about a variant in a oneof field
#[derive(Debug, Clone)]
pub struct OneofVariant {
    pub field_name: String,   // e.g., "azure_service_principal"
    pub variant_name: String, // e.g., "AzureServicePrincipal"
    pub rust_type: String,    // e.g., "AzureServicePrincipal"
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
    pub resource_descriptor: Option<google::api::ResourceDescriptor>,
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

    /// Get all messages that have google.api.resource descriptors
    pub fn messages_with_resources(&self) -> std::collections::HashMap<String, &MessageInfo> {
        self.messages
            .iter()
            .filter(|(_, msg)| msg.resource_descriptor.is_some())
            .map(|(name, msg)| (name.clone(), msg))
            .collect()
    }

    /// Get resource descriptor for a specific message type
    pub fn get_resource_descriptor(
        &self,
        type_name: &str,
    ) -> Option<&google::api::ResourceDescriptor> {
        self.messages
            .get(type_name)
            .and_then(|msg| msg.resource_descriptor.as_ref())
    }

    /// Get all resource types defined in the messages
    pub fn resource_types(&self) -> Vec<String> {
        self.messages
            .values()
            .filter_map(|msg| msg.resource_descriptor.as_ref())
            .map(|rd| rd.r#type.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_messages_with_resources() {
        let mut codegen_metadata = CodeGenMetadata {
            methods: Vec::new(),
            messages: std::collections::HashMap::new(),
        };

        // Add a message without resource descriptor
        let message_without_resource = MessageInfo {
            name: ".test.MessageWithoutResource".to_string(),
            fields: vec![],
            resource_descriptor: None,
        };
        codegen_metadata.messages.insert(
            ".test.MessageWithoutResource".to_string(),
            message_without_resource,
        );

        // Add a message with resource descriptor
        let resource_descriptor = google::api::ResourceDescriptor {
            r#type: "test.io/TestResource".to_string(),
            pattern: vec!["test/{test}".to_string()],
            name_field: "name".to_string(),
            ..Default::default()
        };
        let message_with_resource = MessageInfo {
            name: ".test.MessageWithResource".to_string(),
            fields: vec![],
            resource_descriptor: Some(resource_descriptor),
        };
        codegen_metadata.messages.insert(
            ".test.MessageWithResource".to_string(),
            message_with_resource,
        );

        let messages_with_resources = codegen_metadata.messages_with_resources();
        assert_eq!(messages_with_resources.len(), 1);
        assert!(messages_with_resources.contains_key(".test.MessageWithResource"));
        assert!(!messages_with_resources.contains_key(".test.MessageWithoutResource"));
    }

    #[test]
    fn test_get_resource_descriptor() {
        let mut codegen_metadata = CodeGenMetadata {
            methods: Vec::new(),
            messages: std::collections::HashMap::new(),
        };

        let resource_descriptor = google::api::ResourceDescriptor {
            r#type: "test.io/TestResource".to_string(),
            pattern: vec!["test/{test}".to_string()],
            name_field: "name".to_string(),
            ..Default::default()
        };
        let message_info = MessageInfo {
            name: ".test.TestMessage".to_string(),
            fields: vec![],
            resource_descriptor: Some(resource_descriptor.clone()),
        };
        codegen_metadata
            .messages
            .insert(".test.TestMessage".to_string(), message_info);

        // Test getting existing resource descriptor
        let result = codegen_metadata.get_resource_descriptor(".test.TestMessage");
        assert!(result.is_some());
        let retrieved = result.unwrap();
        assert_eq!(retrieved.r#type, "test.io/TestResource");
        assert_eq!(retrieved.pattern, vec!["test/{test}"]);

        // Test getting non-existent resource descriptor
        let result = codegen_metadata.get_resource_descriptor(".test.NonExistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_resource_types() {
        let mut codegen_metadata = CodeGenMetadata {
            methods: Vec::new(),
            messages: std::collections::HashMap::new(),
        };

        // Add multiple messages with different resource types
        let resource1 = google::api::ResourceDescriptor {
            r#type: "test.io/TypeA".to_string(),
            pattern: vec!["typea/{id}".to_string()],
            ..Default::default()
        };
        let message1 = MessageInfo {
            name: ".test.MessageA".to_string(),
            fields: vec![],
            resource_descriptor: Some(resource1),
        };

        let resource2 = google::api::ResourceDescriptor {
            r#type: "test.io/TypeB".to_string(),
            pattern: vec!["typeb/{id}".to_string()],
            ..Default::default()
        };
        let message2 = MessageInfo {
            name: ".test.MessageB".to_string(),
            fields: vec![],
            resource_descriptor: Some(resource2),
        };

        // Add a message without resource descriptor
        let message3 = MessageInfo {
            name: ".test.MessageC".to_string(),
            fields: vec![],
            resource_descriptor: None,
        };

        codegen_metadata
            .messages
            .insert(".test.MessageA".to_string(), message1);
        codegen_metadata
            .messages
            .insert(".test.MessageB".to_string(), message2);
        codegen_metadata
            .messages
            .insert(".test.MessageC".to_string(), message3);

        let resource_types = codegen_metadata.resource_types();
        assert_eq!(resource_types.len(), 2);
        assert!(resource_types.contains(&"test.io/TypeA".to_string()));
        assert!(resource_types.contains(&"test.io/TypeB".to_string()));
    }
}
