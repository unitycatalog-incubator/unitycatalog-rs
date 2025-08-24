use std::collections::HashMap;

use crate::google::api::ResourceDescriptor;
use crate::parsing::MessageInfo;

pub(crate) struct MessageRegistry<'a> {
    messages: &'a HashMap<String, MessageInfo>,
    resources: HashMap<String, ResourceDescriptor>,
}

impl<'a> MessageRegistry<'a> {
    pub fn new(messages: &'a HashMap<String, MessageInfo>) -> Self {
        let resources = messages
            .iter()
            .filter_map(|(name, info)| {
                info.resource_descriptor
                    .as_ref()
                    .map(|descriptor| (name.clone(), descriptor.clone()))
            })
            .collect();
        MessageRegistry {
            messages,
            resources,
        }
    }

    pub fn resource_from_singular(&self, name: &str) -> Option<&ResourceDescriptor> {
        self.resources.values().find(|r| r.singular == name)
    }

    pub fn resource_from_plural(&self, name: &str) -> Option<&ResourceDescriptor> {
        self.resources.values().find(|r| r.plural == name)
    }
}
