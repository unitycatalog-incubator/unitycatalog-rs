use std::collections::HashMap;

use crate::google::api::ResourceDescriptor;
use crate::parsing::MessageInfo;

pub(crate) struct MessageRegistry<'a> {
    messages: &'a HashMap<String, MessageInfo>,
    resources: HashMap<String, ResourceDescriptor>,
    // Mapping from simple type name to fully qualified name
    simple_to_qualified: HashMap<String, String>,
}

impl<'a> MessageRegistry<'a> {
    pub fn new(messages: &'a HashMap<String, MessageInfo>) -> Self {
        let resources: HashMap<String, ResourceDescriptor> = messages
            .iter()
            .filter_map(|(name, info)| {
                info.resource_descriptor
                    .as_ref()
                    .map(|descriptor| (name.clone(), descriptor.clone()))
            })
            .collect();

        // Create reverse mapping from simple type name to fully qualified name
        let simple_to_qualified: HashMap<String, String> = resources
            .keys()
            .filter_map(|qualified_name| {
                if let Some(last_dot) = qualified_name.rfind('.') {
                    let simple_name = &qualified_name[last_dot + 1..];
                    Some((simple_name.to_string(), qualified_name.clone()))
                } else {
                    Some((qualified_name.clone(), qualified_name.clone()))
                }
            })
            .collect();

        MessageRegistry {
            messages,
            resources,
            simple_to_qualified,
        }
    }

    pub fn resource_from_singular(&self, name: &str) -> Option<&ResourceDescriptor> {
        self.resources.values().find(|r| r.singular == name)
    }

    pub fn resource_from_plural(&self, name: &str) -> Option<&ResourceDescriptor> {
        self.resources.values().find(|r| r.plural == name)
    }

    /// Get resource descriptor by message type name
    pub fn get_resource_descriptor(&self, type_name: &str) -> Option<&ResourceDescriptor> {
        // First try direct lookup (for fully qualified names)
        if let Some(descriptor) = self.resources.get(type_name) {
            return Some(descriptor);
        }

        // Then try lookup by simple name
        if let Some(qualified_name) = self.simple_to_qualified.get(type_name) {
            return self.resources.get(qualified_name);
        }

        None
    }
}
