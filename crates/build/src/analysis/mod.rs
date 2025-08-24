//! Analysis module for processing protobuf metadata into code generation plans
//!
//! This module takes the raw metadata extracted from protobuf files and analyzes it
//! to create a structured plan for code generation. It handles:
//!
//! - Grouping methods by service
//! - Extracting HTTP routing information
//! - Determining parameter types and sources
//! - Planning the structure of generated code

use std::collections::HashSet;

use convert_case::{Case, Casing};

use crate::analysis::messages::MessageRegistry;
use crate::parsing::{
    CodeGenMetadata, MessageField, MethodMetadata, ServiceInfo, extract_http_rule_pattern,
    find_matching_field_for_path_param, should_be_body_field,
};
use crate::utils::{strings, types};

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
                "cargo:warning=Skipping method {}.{} - incomplete metadata",
                info.name, method.method_name
            );
        }
    }

    Ok(ServicePlan {
        service_name: info.name.clone(),
        handler_name,
        base_path,
        methods: method_plans,
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
                "cargo:warning=Method {}.{} missing HTTP info",
                method.service_name, method.method_name
            );
            return Ok(None);
        }
    };

    let planner = MethodPlanner::try_new(method, registry)?;

    // Generate function names
    let handler_function_name = strings::operation_to_method_name(&method.method_name);

    // Get input message fields from metadata
    let input_fields = method.input_fields.clone();

    // Determine if method has response
    let request_type = planner.request_type();

    // Extract parameters based on HTTP rule
    let (path_params, query_params, body_fields) = extract_request_fields(method, &input_fields)?;

    Ok(Some(MethodPlan {
        metadata: method.clone(),
        handler_function_name,
        route_function_name: method.method_name.to_case(Case::Snake),
        http_method,
        http_path,
        path_params,
        query_params,
        body_fields,
        has_response: planner.has_response(),
        request_type,
        is_collection_client_method: planner.is_collection_client_method(),
    }))
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
                rust_type: types::field_type_to_rust_type(&field.field_type),
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
        if field.field_type.starts_with("TYPE_ONEOF:") {
            // Oneof fields are always body fields and always optional
            body_fields.push(BodyField {
                name: field_name.clone(),
                rust_type: types::field_type_to_rust_type(&field.field_type),
                optional: true, // oneof fields are always optional
            });
            processed_fields.insert(field_name.clone());
            continue;
        }

        processed_fields.insert(field_name.clone());

        if should_be_body_field(field_name, body_spec) {
            // Field should be extracted from request body
            body_fields.push(BodyField {
                name: field_name.clone(),
                rust_type: types::field_type_to_rust_type(&field.field_type),
                optional: field.optional,
            });
        } else {
            // Field is a query parameter - handle optionality correctly
            let rust_type = if field.optional {
                types::make_optional(&types::field_type_to_rust_type(&field.field_type))
            } else {
                types::field_type_to_rust_type(&field.field_type)
            };
            query_params.push(QueryParam {
                name: field_name.clone(),
                rust_type,
                optional: field.optional,
            });
        }
    }

    Ok((path_params, query_params, body_fields))
}

#[cfg(test)]
mod tests {
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
}
