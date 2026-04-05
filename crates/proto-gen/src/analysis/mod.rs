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
//! message Catalog {
//!   option (google.api.resource) = {
//!     type: "example.io/Catalog"
//!     pattern: "catalogs/{catalog}"
//!     plural: "catalogs"
//!     singular: "catalog"
//!   };
//!   string name = 1;
//!   // ... other fields
//! }
//!
//! service CatalogService {
//!   rpc GetCatalog(GetCatalogRequest) returns (Catalog);
//!   rpc CreateCatalog(CreateCatalogRequest) returns (Catalog);
//!   rpc UpdateCatalog(UpdateCatalogRequest) returns (Catalog);
//! }
//! ```
//!
//! The analysis will extract that `CatalogService` manages the `Catalog` resource,
//! making this information available for subsequent code generation phases.

use std::collections::{HashMap, HashSet};

use convert_case::{Case, Casing};
use tracing::warn;

use crate::Result;
use crate::parsing::types::BaseType;
use crate::parsing::{CodeGenMetadata, MessageField, MethodMetadata, ServiceInfo};
use crate::utils::strings;

pub(crate) use types::MethodPlanner;
pub use types::{
    BodyField, GenerationPlan, ManagedResource, MethodPlan, PathParam, QueryParam, RequestParam,
    RequestType, ServicePlan, SkippedMethod, extract_managed_resources, split_body_fields,
};

mod types;

/// Analyze collected metadata and create a generation plan.
///
/// Methods with missing HTTP annotations are excluded from the plan and recorded in
/// [`GenerationPlan::skipped_methods`] so callers can distinguish "zero methods generated"
/// from "all methods were silently dropped".
pub fn analyze_metadata(metadata: &CodeGenMetadata) -> Result<GenerationPlan> {
    let mut services = Vec::new();
    let mut skipped_methods = Vec::new();

    for service_info in metadata.services.values() {
        let (service_plan, skipped) = analyze_service(metadata, service_info)?;
        services.push(service_plan);
        skipped_methods.extend(skipped);
    }

    Ok(GenerationPlan {
        services,
        skipped_methods,
    })
}

/// Analyze a single service and create a service plan.
///
/// Returns the plan and a list of methods that were skipped due to incomplete metadata.
fn analyze_service(
    metadata: &CodeGenMetadata,
    info: &ServiceInfo,
) -> Result<(ServicePlan, Vec<SkippedMethod>)> {
    let handler_name = strings::service_to_handler_name(&info.name);
    let base_path = strings::service_to_base_path(&info.name);

    let mut method_plans = Vec::new();
    let mut skipped = Vec::new();

    for method in &info.methods {
        if let Some(method_plan) = analyze_method(metadata, method)? {
            method_plans.push(method_plan);
        } else {
            warn!(
                "Skipping method {}.{} - incomplete metadata",
                info.name, method.method_name
            );
            skipped.push(SkippedMethod {
                service_name: info.name.clone(),
                method_name: method.method_name.clone(),
                reason: "missing HTTP annotation".to_string(),
            });
        }
    }

    let managed_resources = types::extract_managed_resources(metadata, &method_plans);

    Ok((
        ServicePlan {
            service_name: info.name.clone(),
            handler_name,
            base_path,
            methods: method_plans,
            managed_resources,
            documentation: info.documentation.clone(),
        },
        skipped,
    ))
}

/// Analyze a single method and create a method plan.
///
/// Returns `None` if the method has incomplete metadata (e.g., missing HTTP annotation).
pub(crate) fn analyze_method(
    metadata: &CodeGenMetadata,
    method: &MethodMetadata,
) -> Result<Option<MethodPlan>> {
    let http_method = match method.http_method() {
        Some(m) => m.to_string(),
        None => {
            warn!(
                "Method {}.{} missing HTTP info",
                method.service_name, method.method_name
            );
            return Ok(None);
        }
    };

    let planner = MethodPlanner::try_new(method, metadata)?;
    let request_type = planner.request_type();
    let has_response = planner.has_response();
    let output_resource_type = planner.output_resource_type();
    let http_pattern = planner.into_http_pattern();

    let input_fields = metadata.get_message_fields(&method.input_type);
    let (path_params, query_params, body_fields) = extract_request_fields(method, &input_fields)?;

    let parameters = path_params
        .into_iter()
        .map(Into::into)
        .chain(query_params.into_iter().map(Into::into))
        .chain(body_fields.into_iter().map(Into::into))
        .collect();

    Ok(Some(MethodPlan {
        metadata: method.clone(),
        handler_function_name: method.method_name.to_case(Case::Snake),
        http_method,
        parameters,
        has_response,
        request_type,
        output_resource_type,
        http_pattern,
    }))
}

/// Extract and classify request fields from an input message into path, query, and body buckets.
///
/// - Path parameters are matched against URL template parameters and ordered accordingly.
/// - Fields matching the `body` spec (`"*"`, `""`, or a specific field name) become body fields.
/// - All remaining fields become query parameters. Fields not explicitly marked optional
///   via `UnifiedType.is_optional` are treated as required query parameters.
/// - Oneof fields are always placed in the body as optional variants.
fn extract_request_fields(
    method: &MethodMetadata,
    input_fields: &[MessageField],
) -> Result<(Vec<PathParam>, Vec<QueryParam>, Vec<BodyField>)> {
    let mut path_params = Vec::new();
    let mut query_params = Vec::new();
    let mut body_fields = Vec::new();

    let path_param_names = method.http_pattern.parameter_names();
    let body_spec = method.http_rule.body.as_str();

    // Build an O(1) lookup map for input fields by name.
    let fields_by_name: HashMap<&str, &MessageField> =
        input_fields.iter().map(|f| (f.name.as_str(), f)).collect();

    let mut processed_fields = HashSet::new();

    // Add path parameters in URL template order.
    for path_param_name in path_param_names {
        if let Some(field) = fields_by_name.get(path_param_name.as_str()) {
            path_params.push(PathParam {
                name: field.name.clone(),
                field_type: field.unified_type.clone(),
                documentation: field.documentation.clone(),
            });
            processed_fields.insert(field.name.as_str());
        }
    }

    // Classify remaining fields as body or query.
    for field in input_fields {
        let field_name = field.name.as_str();

        if processed_fields.contains(field_name) {
            continue;
        }

        // Oneof fields are always body fields and always optional.
        if matches!(field.unified_type.base_type, BaseType::OneOf(_)) {
            body_fields.push(BodyField {
                name: field.name.clone(),
                field_type: field.unified_type.clone().optional(),
                repeated: false,
                oneof_variants: field.oneof_variants.clone(),
                documentation: field.documentation.clone(),
            });
            processed_fields.insert(field_name);
            continue;
        }

        let is_body = match body_spec {
            "*" => true,
            "" => false,
            specific => specific == field_name,
        };

        if is_body {
            body_fields.push(BodyField {
                name: field.name.clone(),
                field_type: field.unified_type.clone(),
                repeated: field.unified_type.is_repeated,
                oneof_variants: None,
                documentation: field.documentation.clone(),
            });
        } else {
            query_params.push(QueryParam {
                name: field.name.clone(),
                field_type: field.unified_type.clone(),
                documentation: field.documentation.clone(),
            });
        }
        processed_fields.insert(field_name);
    }

    Ok((path_params, query_params, body_fields))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::google::api::{HttpRule, ResourceDescriptor, http_rule::Pattern};
    use crate::parsing::types::UnifiedType;
    use crate::parsing::{CodeGenMetadata, HttpPattern, MessageInfo, MethodMetadata, ServiceInfo};
    use std::collections::HashMap;

    fn make_metadata_with_catalog() -> CodeGenMetadata {
        let catalog_resource = ResourceDescriptor {
            r#type: "example.io/Catalog".to_string(),
            pattern: vec!["catalogs/{catalog}".to_string()],
            name_field: "name".to_string(),
            history: 0,
            plural: "catalogs".to_string(),
            singular: "catalog".to_string(),
            style: vec![],
        };
        let catalog_info = MessageInfo {
            name: "Catalog".to_string(),
            fields: vec![],
            resource_descriptor: Some(catalog_resource),
            documentation: None,
        };
        let mut messages = HashMap::new();
        messages.insert("Catalog".to_string(), catalog_info);
        CodeGenMetadata {
            messages,
            ..Default::default()
        }
    }

    fn make_get_method() -> MethodMetadata {
        MethodMetadata {
            service_name: "CatalogService".to_string(),
            method_name: "GetCatalog".to_string(),
            input_type: "GetCatalogRequest".to_string(),
            output_type: "Catalog".to_string(),
            operation: None,
            http_rule: HttpRule {
                selector: "".to_string(),
                pattern: Some(Pattern::Get("/catalogs/{name}".to_string())),
                body: "".to_string(),
                response_body: "".to_string(),
                additional_bindings: vec![],
            },
            http_pattern: HttpPattern::parse("/catalogs/{name}"),
            documentation: None,
        }
    }

    #[test]
    fn test_managed_resources_extraction() {
        let metadata = make_metadata_with_catalog();
        let service_info = ServiceInfo {
            name: "CatalogService".to_string(),
            documentation: None,
            methods: vec![make_get_method()],
        };
        let (service_plan, skipped) = analyze_service(&metadata, &service_info).unwrap();

        assert!(skipped.is_empty());
        assert_eq!(service_plan.managed_resources.len(), 1);
        assert_eq!(service_plan.managed_resources[0].type_name, "Catalog");
        assert_eq!(
            service_plan.managed_resources[0].descriptor.r#type,
            "example.io/Catalog"
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
        let metadata = make_metadata_with_catalog();
        let update_method = MethodMetadata {
            service_name: "CatalogService".to_string(),
            method_name: "UpdateCatalog".to_string(),
            input_type: "UpdateCatalogRequest".to_string(),
            output_type: "Catalog".to_string(),
            operation: None,
            http_rule: HttpRule {
                selector: "".to_string(),
                pattern: Some(Pattern::Patch("/catalogs/{name}".to_string())),
                body: "*".to_string(),
                response_body: "".to_string(),
                additional_bindings: vec![],
            },
            http_pattern: HttpPattern::parse("/catalogs/{name}"),
            documentation: None,
        };
        let service_info = ServiceInfo {
            name: "CatalogService".to_string(),
            documentation: None,
            methods: vec![make_get_method(), update_method],
        };
        let (service_plan, _skipped) = analyze_service(&metadata, &service_info).unwrap();

        assert_eq!(service_plan.managed_resources.len(), 1);
        assert_eq!(service_plan.managed_resources[0].type_name, "Catalog");
    }

    #[test]
    fn test_analyze_method_missing_http_pattern_returns_none() {
        let metadata = CodeGenMetadata::default();
        let method = MethodMetadata {
            service_name: "SomeService".to_string(),
            method_name: "SomeMethod".to_string(),
            input_type: "".to_string(),
            output_type: "".to_string(),
            operation: None,
            http_rule: HttpRule {
                selector: "".to_string(),
                pattern: None,
                body: "".to_string(),
                response_body: "".to_string(),
                additional_bindings: vec![],
            },
            http_pattern: HttpPattern::parse(""),
            documentation: None,
        };
        let result = analyze_method(&metadata, &method).unwrap();
        assert!(result.is_none());
    }

    // --- extract_request_fields unit tests ---

    fn make_string_field(name: &str, optional: bool) -> MessageField {
        use crate::parsing::types::BaseType;
        MessageField {
            name: name.to_string(),
            unified_type: UnifiedType {
                base_type: BaseType::String,
                is_optional: optional,
                is_repeated: false,
            },
            documentation: None,
            oneof_variants: None,
            field_behavior: vec![],
        }
    }

    fn make_repeated_field(name: &str) -> MessageField {
        use crate::parsing::types::BaseType;
        MessageField {
            name: name.to_string(),
            unified_type: UnifiedType {
                base_type: BaseType::String,
                is_optional: false,
                is_repeated: true,
            },
            documentation: None,
            oneof_variants: None,
            field_behavior: vec![],
        }
    }

    fn make_method_with_pattern(pattern: Pattern, body: &str, path: &str) -> MethodMetadata {
        MethodMetadata {
            service_name: "Svc".to_string(),
            method_name: "Method".to_string(),
            input_type: "".to_string(),
            output_type: "".to_string(),
            operation: None,
            http_rule: HttpRule {
                selector: "".to_string(),
                pattern: Some(pattern),
                body: body.to_string(),
                response_body: "".to_string(),
                additional_bindings: vec![],
            },
            http_pattern: HttpPattern::parse(path),
            documentation: None,
        }
    }

    #[test]
    fn test_extract_path_params_in_url_order() {
        let method =
            make_method_with_pattern(Pattern::Get("/a/{x}/b/{y}".to_string()), "", "/a/{x}/b/{y}");
        let fields = vec![make_string_field("y", false), make_string_field("x", false)];
        let (path, query, body) = extract_request_fields(&method, &fields).unwrap();
        // Path params should be in URL order: x, y — not field declaration order
        assert_eq!(path.len(), 2);
        assert_eq!(path[0].name, "x");
        assert_eq!(path[1].name, "y");
        assert!(query.is_empty());
        assert!(body.is_empty());
    }

    #[test]
    fn test_extract_body_wildcard() {
        let method = make_method_with_pattern(Pattern::Post("/items".to_string()), "*", "/items");
        let fields = vec![
            make_string_field("name", false),
            make_string_field("description", true),
        ];
        let (path, query, body) = extract_request_fields(&method, &fields).unwrap();
        assert!(path.is_empty());
        assert!(query.is_empty());
        assert_eq!(body.len(), 2);
    }

    #[test]
    fn test_extract_specific_body_field() {
        let method = make_method_with_pattern(
            Pattern::Patch("/items/{name}".to_string()),
            "payload",
            "/items/{name}",
        );
        let fields = vec![
            make_string_field("name", false),    // path
            make_string_field("payload", false), // body (specific)
            make_string_field("extra", true),    // query
        ];
        let (path, query, body) = extract_request_fields(&method, &fields).unwrap();
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].name, "name");
        assert_eq!(body.len(), 1);
        assert_eq!(body[0].name, "payload");
        assert_eq!(query.len(), 1);
        assert_eq!(query[0].name, "extra");
    }

    #[test]
    fn test_extract_no_body_spec_all_query() {
        let method = make_method_with_pattern(Pattern::Get("/items".to_string()), "", "/items");
        let fields = vec![
            make_string_field("filter", true),
            make_string_field("page_size", true),
        ];
        let (path, query, body) = extract_request_fields(&method, &fields).unwrap();
        assert!(path.is_empty());
        assert_eq!(query.len(), 2);
        assert!(body.is_empty());
    }

    #[test]
    fn test_extract_repeated_field_becomes_body_with_repeated_flag() {
        let method = make_method_with_pattern(Pattern::Post("/items".to_string()), "*", "/items");
        let fields = vec![make_repeated_field("tags")];
        let (_, _, body) = extract_request_fields(&method, &fields).unwrap();
        assert_eq!(body.len(), 1);
        assert!(body[0].repeated);
    }
}
