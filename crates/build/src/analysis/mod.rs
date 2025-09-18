//! Analysis module for processing protobuf metadata into code generation plans
//!
//! This module takes the raw metadata extracted from protobuf files and analyzes it
//! to create a structured plan for code generation. It handles:
//!
//! - Grouping methods by service
//! - Extracting HTTP routing information
//! - Determining parameter types and sources
//! - Planning the structure of generated code
//! - Extracting managed resources from method return types
//!
//! ## Managed Resources
//!
//! Services often manage one or more resource types. These resources are automatically
//! extracted from the return types of get, create, and update methods. For example:
//!
//! ```proto
//! message CatalogInfo {
//!   option (google.api.resource) = {
//!     type: "unitycatalog.io/Catalog"
//!     pattern: "catalogs/{catalog}"
//!     plural: "catalogs"
//!     singular: "catalog"
//!   };
//!   string name = 1;
//!   // ... other fields
//! }
//!
//! service CatalogsService {
//!   rpc GetCatalog(GetCatalogRequest) returns (CatalogInfo);
//!   rpc CreateCatalog(CreateCatalogRequest) returns (CatalogInfo);
//!   rpc UpdateCatalog(UpdateCatalogRequest) returns (CatalogInfo);
//! }
//! ```
//!
//! The analysis will extract that `CatalogsService` manages the `Catalog` resource,
//! making this information available for subsequent code generation phases.

use std::collections::HashSet;

use convert_case::{Case, Casing};

use crate::analysis::messages::MessageRegistry;
use crate::parsing::types::BaseType;
use crate::parsing::{
    CodeGenMetadata, MessageField, MethodMetadata, ServiceInfo, extract_http_rule_pattern,
    find_matching_field_for_path_param, should_be_body_field,
};
use crate::utils::{strings, types};

use services::extract_managed_resources;
pub(crate) use services::*;

mod messages;
mod services;

/// Analyze collected metadata and create a generation plan
pub fn analyze_metadata(
    metadata: &CodeGenMetadata,
) -> Result<GenerationPlan, Box<dyn std::error::Error>> {
    let mut services = Vec::new();

    let registry = MessageRegistry::new(&metadata.messages);

    for service_info in metadata.services.values() {
        let service_plan = analyze_service(&registry, service_info)?;
        services.push(service_plan);
    }

    Ok(GenerationPlan { services })
}

/// Analyze a single service and create a service plan
fn analyze_service(
    registry: &MessageRegistry<'_>,
    info: &ServiceInfo,
) -> Result<ServicePlan, Box<dyn std::error::Error>> {
    let handler_name = strings::service_to_handler_name(&info.name);
    let base_path = strings::service_to_base_path(&info.name);

    let mut method_plans = Vec::new();

    for method in &info.methods {
        if let Some(method_plan) = analyze_method(registry, method)? {
            method_plans.push(method_plan);
        } else {
            println!(
                "Skipping method {}.{} - incomplete metadata",
                info.name, method.method_name
            );
        }
    }

    // Extract managed resources from method return types
    let managed_resources = extract_managed_resources(registry, &method_plans);

    Ok(ServicePlan {
        service_name: info.name.clone(),
        handler_name,
        base_path,
        methods: method_plans,
        managed_resources,
    })
}

/// Analyze a single method and create a method plan
pub fn analyze_method(
    registry: &MessageRegistry<'_>,
    method: &MethodMetadata,
) -> Result<Option<MethodPlan>, Box<dyn std::error::Error>> {
    let (http_method, http_path) = match method.http_info() {
        Some(info) => info,
        None => {
            println!(
                "Method {}.{} missing HTTP info",
                method.service_name, method.method_name
            );
            return Ok(None);
        }
    };

    let planner = MethodPlanner::try_new(method, registry)?;

    // Determine if method has response
    let request_type = planner.request_type();

    // Extract parameters based on HTTP rule
    let (path_params, query_params, body_fields) =
        extract_request_fields(method, &method.input_fields)?;

    let parameters = path_params
        .clone()
        .into_iter()
        .map(Into::into)
        .chain(query_params.clone().into_iter().map(Into::into))
        .chain(body_fields.clone().into_iter().map(Into::into))
        .collect();

    let method_plan = MethodPlan {
        metadata: method.clone(),
        handler_function_name: method.method_name.to_case(Case::Snake),
        http_method,
        http_path,
        parameters,
        path_params,
        query_params,
        body_fields,
        has_response: planner.has_response(),
        request_type,
        is_collection_client_method: planner.is_collection_client_method(),
        returns_resource: planner.returns_resource(),
        output_resource_type: planner.output_resource_type(),
    };

    Ok(Some(method_plan))
}

/// Extract request fields based on HTTP rule analysis
fn extract_request_fields(
    method: &MethodMetadata,
    input_fields: &[MessageField],
) -> Result<(Vec<PathParam>, Vec<QueryParam>, Vec<BodyField>), Box<dyn std::error::Error>> {
    let mut path_params = Vec::new();
    let mut query_params = Vec::new();
    let mut body_fields = Vec::new();

    // Extract path parameters from HTTP pattern in order
    let http_pattern = extract_http_rule_pattern(&method.http_rule).unwrap();
    let path_param_names_ordered = http_pattern.parameter_names().to_vec();

    // Get body field specification from HTTP rule
    let body_spec = method.http_rule.body.as_str();

    // Track which fields we've already processed to avoid duplicates
    let mut processed_fields = HashSet::new();

    // First, add path parameters in URL order
    for path_param_name in &path_param_names_ordered {
        let field = find_matching_field_for_path_param(path_param_name, input_fields);
        if let Some(field) = field {
            path_params.push(PathParam {
                template_param: path_param_name.clone(),
                field_name: field.name.clone(),
                field_type: field.unified_type.clone(),
            });
            processed_fields.insert(field.name.clone());
        }
    }

    // Then analyze remaining fields
    for field in input_fields {
        let field_name = &field.name;

        // Skip if already processed as path parameter
        if processed_fields.contains(field_name) {
            continue;
        }

        // Skip oneof fields that should be handled as individual enum variants in the body
        if matches!(field.unified_type.base_type, BaseType::OneOf(_)) {
            // Oneof fields are always body fields and always optional
            body_fields.push(BodyField {
                name: field_name.clone(),
                rust_type: types::field_type_to_rust_type(&field.field_type),
                optional: true, // oneof fields are always optional
                field_type: field.unified_type.clone(),
            });
            processed_fields.insert(field_name.clone());
            continue;
        }

        processed_fields.insert(field_name.clone());

        if should_be_body_field(field_name, body_spec) {
            body_fields.push(BodyField {
                name: field_name.clone(),
                rust_type: types::field_type_to_rust_type(&field.field_type),
                optional: field.optional,
                field_type: field.unified_type.clone(),
            });
        } else {
            query_params.push(QueryParam {
                name: field_name.clone(),
                optional: field.optional,
                field_type: field.unified_type.clone(),
            });
        }
    }

    Ok((path_params, query_params, body_fields))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::google::api::{HttpRule, ResourceDescriptor, http_rule::Pattern};
    use crate::parsing::{MessageInfo, MethodMetadata, ServiceInfo};
    use std::collections::HashMap;

    #[test]
    fn test_field_type_conversion() {
        use crate::utils::types::field_type_to_rust_type;
        assert_eq!(field_type_to_rust_type("TYPE_STRING"), "String");
        assert_eq!(field_type_to_rust_type("TYPE_INT32"), "i32");
        assert_eq!(field_type_to_rust_type("TYPE_BOOL"), "bool");
        assert_eq!(
            field_type_to_rust_type("TYPE_MESSAGE:.unitycatalog.CatalogInfo"),
            "CatalogInfo"
        );
    }

    #[test]
    fn test_managed_resources_extraction() {
        // Create a mock message with resource descriptor
        let mut messages = HashMap::new();
        let catalog_resource = ResourceDescriptor {
            r#type: "unitycatalog.io/Catalog".to_string(),
            pattern: vec!["catalogs/{catalog}".to_string()],
            name_field: "name".to_string(),
            history: 0,
            plural: "catalogs".to_string(),
            singular: "catalog".to_string(),
            style: vec![],
        };

        let catalog_info = MessageInfo {
            name: "CatalogInfo".to_string(),
            fields: vec![],
            resource_descriptor: Some(catalog_resource.clone()),
            documentation: None,
        };
        messages.insert("CatalogInfo".to_string(), catalog_info);

        let registry = MessageRegistry::new(&messages);

        // Create a mock service with methods that return resources
        let get_method = MethodMetadata {
            service_name: "CatalogsService".to_string(),
            method_name: "GetCatalog".to_string(),
            input_type: "GetCatalogRequest".to_string(),
            output_type: "CatalogInfo".to_string(),
            operation: None,
            http_rule: HttpRule {
                selector: "".to_string(),
                pattern: Some(Pattern::Get("/catalogs/{name}".to_string())),
                body: "".to_string(),
                response_body: "".to_string(),
                additional_bindings: vec![],
            },
            input_fields: vec![],
            documentation: None,
        };

        let service_info = ServiceInfo {
            name: "CatalogsService".to_string(),
            documentation: None,
            methods: vec![get_method],
        };

        // Analyze the service
        let service_plan = analyze_service(&registry, &service_info).unwrap();

        // Verify that managed resources were extracted
        assert_eq!(service_plan.managed_resources.len(), 1);
        assert_eq!(service_plan.managed_resources[0].type_name, "CatalogInfo");
        assert_eq!(
            service_plan.managed_resources[0].descriptor.r#type,
            "unitycatalog.io/Catalog"
        );
        assert_eq!(
            service_plan.managed_resources[0].descriptor.singular,
            "catalog"
        );
        assert_eq!(
            service_plan.managed_resources[0].descriptor.plural,
            "catalogs"
        );
    }

    #[test]
    fn test_no_duplicate_managed_resources() {
        // Create a mock message with resource descriptor
        let mut messages = HashMap::new();
        let catalog_resource = ResourceDescriptor {
            r#type: "unitycatalog.io/Catalog".to_string(),
            pattern: vec!["catalogs/{catalog}".to_string()],
            name_field: "name".to_string(),
            history: 0,
            plural: "catalogs".to_string(),
            singular: "catalog".to_string(),
            style: vec![],
        };

        let catalog_info = MessageInfo {
            name: "CatalogInfo".to_string(),
            fields: vec![],
            resource_descriptor: Some(catalog_resource.clone()),
            documentation: None,
        };
        messages.insert("CatalogInfo".to_string(), catalog_info);

        let registry = MessageRegistry::new(&messages);

        // Create multiple methods that return the same resource type
        let get_method = MethodMetadata {
            service_name: "CatalogsService".to_string(),
            method_name: "GetCatalog".to_string(),
            input_type: "GetCatalogRequest".to_string(),
            output_type: "CatalogInfo".to_string(),
            operation: None,
            http_rule: HttpRule {
                selector: "".to_string(),
                pattern: Some(Pattern::Get("/catalogs/{name}".to_string())),
                body: "".to_string(),
                response_body: "".to_string(),
                additional_bindings: vec![],
            },
            input_fields: vec![],
            documentation: None,
        };

        let update_method = MethodMetadata {
            service_name: "CatalogsService".to_string(),
            method_name: "UpdateCatalog".to_string(),
            input_type: "UpdateCatalogRequest".to_string(),
            output_type: "CatalogInfo".to_string(),
            operation: None,
            http_rule: HttpRule {
                selector: "".to_string(),
                pattern: Some(Pattern::Patch("/catalogs/{name}".to_string())),
                body: "*".to_string(),
                response_body: "".to_string(),
                additional_bindings: vec![],
            },
            input_fields: vec![],
            documentation: None,
        };

        let service_info = ServiceInfo {
            name: "CatalogsService".to_string(),
            documentation: None,
            methods: vec![get_method, update_method],
        };

        // Analyze the service
        let service_plan = analyze_service(&registry, &service_info).unwrap();

        // Verify that we only have one managed resource despite multiple methods returning it
        assert_eq!(service_plan.managed_resources.len(), 1);
        assert_eq!(service_plan.managed_resources[0].type_name, "CatalogInfo");
    }
}
