use std::collections::{HashMap, HashSet};

use crate::gnostic::openapi::v3::Operation;
use crate::google::api::{FieldBehavior, HttpRule, ResourceDescriptor};

pub mod codegen;
pub mod error;
pub mod parsing;
pub mod utils;

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
    /// Extract the operation ID from gnostic annotations
    /// Note: This is primarily used for OpenAPI spec generation, not request classification
    pub fn operation_id(&self) -> Option<&str> {
        let operation = self.operation.as_ref()?;
        if operation.operation_id.is_empty() {
            None
        } else {
            Some(&operation.operation_id)
        }
    }

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
            utils::paths::extract_path_parameters(&path)
        } else {
            Vec::new()
        }
    }

    /// Determine the request type category (List, Create, Get, Update, Delete)
    pub fn request_type(&self) -> RequestType {
        utils::requests::classify_request_type_from_http(&self.http_rule, &self.method_name)
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

/// Collected metadata for code generation
#[derive(Debug)]
pub struct CodeGenMetadata {
    pub messages: HashMap<String, MessageInfo>,
    pub services: HashMap<String, ServiceInfo>,
}

/// Information about a protobuf message
#[derive(Debug, Clone)]
pub struct MessageInfo {
    pub name: String,
    pub fields: Vec<MessageField>,
    pub resource_descriptor: Option<ResourceDescriptor>,
    pub documentation: Option<String>,
}

/// Information about a protobuf service
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub documentation: Option<String>,
    pub methods: Vec<MethodMetadata>,
}

impl CodeGenMetadata {
    /// Get all methods across all services
    pub fn all_methods(&self) -> impl Iterator<Item = &MethodMetadata> {
        self.services.values().flat_map(|service| &service.methods)
    }

    /// Get message fields for a given type name
    pub fn get_message_fields(&self, type_name: &str) -> Vec<MessageField> {
        self.messages
            .get(type_name)
            .map(|msg| msg.fields.clone())
            .unwrap_or_default()
    }

    /// Get all methods that have complete REST metadata (http_rule required)
    pub fn complete_methods(&self) -> Vec<&MethodMetadata> {
        self.all_methods()
            .filter(|m| m.http_info().is_some())
            .collect()
    }

    /// Get all messages that have google.api.resource descriptors
    pub fn messages_with_resources(&self) -> HashMap<String, &MessageInfo> {
        self.messages
            .iter()
            .filter(|(_, msg)| msg.resource_descriptor.is_some())
            .map(|(name, msg)| (name.clone(), msg))
            .collect()
    }

    /// Get resource descriptor for a specific message type
    pub fn get_resource_descriptor(&self, type_name: &str) -> Option<&ResourceDescriptor> {
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

    /// Get all messages that have fields with specific field behaviors
    pub fn messages_with_field_behavior(
        &self,
        behavior: FieldBehavior,
    ) -> HashMap<String, &MessageInfo> {
        self.messages
            .iter()
            .filter(|(_, msg)| {
                msg.fields
                    .iter()
                    .any(|field| field.field_behavior.contains(&behavior))
            })
            .map(|(name, msg)| (name.clone(), msg))
            .collect()
    }

    /// Get all fields with a specific field behavior across all messages
    pub fn fields_with_behavior(&self, behavior: FieldBehavior) -> Vec<(&str, &MessageField)> {
        self.messages
            .values()
            .flat_map(|msg| {
                msg.fields
                    .iter()
                    .filter(|field| field.field_behavior.contains(&behavior))
                    .map(|field| (msg.name.as_str(), field))
            })
            .collect()
    }

    /// Get all unique field behaviors used across all messages
    pub fn all_field_behaviors(&self) -> HashSet<FieldBehavior> {
        self.messages
            .values()
            .flat_map(|msg| &msg.fields)
            .flat_map(|field| &field.field_behavior)
            .cloned()
            .collect()
    }

    /// Get field behavior statistics
    pub fn field_behavior_stats(&self) -> HashMap<FieldBehavior, usize> {
        let mut stats = HashMap::new();
        for msg in self.messages.values() {
            for field in &msg.fields {
                for behavior in &field.field_behavior {
                    *stats.entry(*behavior).or_insert(0) += 1;
                }
            }
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_messages_with_resources() {
        let mut codegen_metadata = CodeGenMetadata {
            messages: HashMap::new(),
            services: HashMap::new(),
        };

        // Add a message without resource descriptor
        let message_without_resource = MessageInfo {
            name: ".test.MessageWithoutResource".to_string(),
            fields: vec![],
            resource_descriptor: None,
            documentation: None,
        };
        codegen_metadata.messages.insert(
            ".test.MessageWithoutResource".to_string(),
            message_without_resource,
        );

        // Add a message with resource descriptor
        let resource_descriptor = ResourceDescriptor {
            r#type: "test.io/TestResource".to_string(),
            pattern: vec!["test/{test}".to_string()],
            name_field: "name".to_string(),
            ..Default::default()
        };
        let message_with_resource = MessageInfo {
            name: ".test.MessageWithResource".to_string(),
            fields: vec![],
            resource_descriptor: Some(resource_descriptor),
            documentation: None,
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
            messages: HashMap::new(),
            services: HashMap::new(),
        };

        let resource_descriptor = ResourceDescriptor {
            r#type: "test.io/TestResource".to_string(),
            pattern: vec!["test/{test}".to_string()],
            name_field: "name".to_string(),
            ..Default::default()
        };
        let message_info = MessageInfo {
            name: ".test.TestMessage".to_string(),
            fields: vec![],
            resource_descriptor: Some(resource_descriptor.clone()),
            documentation: None,
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
            messages: HashMap::new(),
            services: HashMap::new(),
        };

        // Add multiple messages with different resource types
        let resource1 = ResourceDescriptor {
            r#type: "test.io/TypeA".to_string(),
            pattern: vec!["typea/{id}".to_string()],
            ..Default::default()
        };
        let message1 = MessageInfo {
            name: ".test.MessageA".to_string(),
            fields: vec![],
            resource_descriptor: Some(resource1),
            documentation: None,
        };

        let resource2 = ResourceDescriptor {
            r#type: "test.io/TypeB".to_string(),
            pattern: vec!["typeb/{id}".to_string()],
            ..Default::default()
        };
        let message2 = MessageInfo {
            name: ".test.MessageB".to_string(),
            fields: vec![],
            resource_descriptor: Some(resource2),
            documentation: None,
        };

        // Add a message without resource descriptor
        let message3 = MessageInfo {
            name: ".test.MessageC".to_string(),
            fields: vec![],
            resource_descriptor: None,
            documentation: None,
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

    #[test]
    fn test_messages_with_field_behavior() {
        let mut codegen_metadata = CodeGenMetadata {
            messages: HashMap::new(),
            services: HashMap::new(),
        };

        // Add a message with required fields
        let message_with_required = MessageInfo {
            name: ".test.MessageWithRequired".to_string(),
            fields: vec![
                MessageField {
                    name: "required_field".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: false,
                    repeated: false,
                    oneof_name: None,
                    documentation: None,
                    oneof_variants: None,
                    field_behavior: vec![FieldBehavior::Required],
                },
                MessageField {
                    name: "optional_field".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: true,
                    repeated: false,
                    oneof_name: None,
                    documentation: None,
                    oneof_variants: None,
                    field_behavior: vec![FieldBehavior::Optional],
                },
            ],
            resource_descriptor: None,
            documentation: None,
        };
        codegen_metadata.messages.insert(
            ".test.MessageWithRequired".to_string(),
            message_with_required,
        );

        // Add a message without required fields
        let message_without_required = MessageInfo {
            name: ".test.MessageWithoutRequired".to_string(),
            fields: vec![MessageField {
                name: "output_field".to_string(),
                field_type: "TYPE_STRING".to_string(),
                optional: true,
                repeated: false,
                oneof_name: None,
                documentation: None,
                oneof_variants: None,
                field_behavior: vec![FieldBehavior::OutputOnly],
            }],
            resource_descriptor: None,
            documentation: None,
        };
        codegen_metadata.messages.insert(
            ".test.MessageWithoutRequired".to_string(),
            message_without_required,
        );

        let required_messages =
            codegen_metadata.messages_with_field_behavior(FieldBehavior::Required);
        assert_eq!(required_messages.len(), 1);
        assert!(required_messages.contains_key(".test.MessageWithRequired"));

        let output_only_messages =
            codegen_metadata.messages_with_field_behavior(FieldBehavior::OutputOnly);
        assert_eq!(output_only_messages.len(), 1);
        assert!(output_only_messages.contains_key(".test.MessageWithoutRequired"));
    }

    #[test]
    fn test_fields_with_behavior() {
        let mut codegen_metadata = CodeGenMetadata {
            messages: HashMap::new(),
            services: HashMap::new(),
        };

        let message_info = MessageInfo {
            name: ".test.TestMessage".to_string(),
            fields: vec![
                MessageField {
                    name: "id_field".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: false,
                    repeated: false,
                    oneof_name: None,
                    documentation: None,
                    oneof_variants: None,
                    field_behavior: vec![FieldBehavior::Required, FieldBehavior::Identifier],
                },
                MessageField {
                    name: "readonly_field".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: true,
                    repeated: false,
                    oneof_name: None,
                    documentation: None,
                    oneof_variants: None,
                    field_behavior: vec![FieldBehavior::OutputOnly],
                },
            ],
            resource_descriptor: None,
            documentation: None,
        };
        codegen_metadata
            .messages
            .insert(".test.TestMessage".to_string(), message_info);

        let required_fields = codegen_metadata.fields_with_behavior(FieldBehavior::Required);
        assert_eq!(required_fields.len(), 1);
        assert_eq!(required_fields[0].1.name, "id_field");

        let identifier_fields = codegen_metadata.fields_with_behavior(FieldBehavior::Identifier);
        assert_eq!(identifier_fields.len(), 1);
        assert_eq!(identifier_fields[0].1.name, "id_field");

        let output_only_fields = codegen_metadata.fields_with_behavior(FieldBehavior::OutputOnly);
        assert_eq!(output_only_fields.len(), 1);
        assert_eq!(output_only_fields[0].1.name, "readonly_field");
    }

    #[test]
    fn test_field_behavior_stats() {
        let mut codegen_metadata = CodeGenMetadata {
            messages: HashMap::new(),
            services: HashMap::new(),
        };

        let message_info = MessageInfo {
            name: ".test.TestMessage".to_string(),
            fields: vec![
                MessageField {
                    name: "field1".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: false,
                    repeated: false,
                    oneof_name: None,
                    documentation: None,
                    oneof_variants: None,
                    field_behavior: vec![FieldBehavior::Required],
                },
                MessageField {
                    name: "field2".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: false,
                    repeated: false,
                    oneof_name: None,
                    documentation: None,
                    oneof_variants: None,
                    field_behavior: vec![FieldBehavior::Required],
                },
                MessageField {
                    name: "field3".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: true,
                    repeated: false,
                    oneof_name: None,
                    documentation: None,
                    oneof_variants: None,
                    field_behavior: vec![FieldBehavior::Optional],
                },
            ],
            resource_descriptor: None,
            documentation: None,
        };
        codegen_metadata
            .messages
            .insert(".test.TestMessage".to_string(), message_info);

        let stats = codegen_metadata.field_behavior_stats();
        assert_eq!(stats.get(&FieldBehavior::Required), Some(&2));
        assert_eq!(stats.get(&FieldBehavior::Optional), Some(&1));
        assert_eq!(stats.get(&FieldBehavior::OutputOnly), None);
    }

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

        // Test that all_methods returns the method from the service
        let all_methods: Vec<&MethodMetadata> = codegen_metadata.all_methods().collect();
        assert_eq!(all_methods.len(), 1);
        assert_eq!(all_methods[0].method_name, "TestMethod");
        assert_eq!(all_methods[0].service_name, "TestService");

        // Test that we can access the service and its methods directly
        let service = codegen_metadata.services.get("TestService").unwrap();
        assert_eq!(service.name, "TestService");
        assert_eq!(service.methods.len(), 1);
        assert_eq!(service.methods[0].method_name, "TestMethod");
    }
}
