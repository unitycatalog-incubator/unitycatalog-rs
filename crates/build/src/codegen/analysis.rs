//! Analysis module for processing protobuf metadata into code generation plans
//!
//! This module takes the raw metadata extracted from protobuf files and analyzes it
//! to create a structured plan for code generation. It handles:
//!
//! - Grouping methods by service
//! - Extracting HTTP routing information
//! - Determining parameter types and sources
//! - Planning the structure of generated code

use super::{BodyField, GenerationPlan, MethodPlan, PathParam, QueryParam, ServicePlan};
use crate::utils::{paths, requests, strings, types};
use crate::{CodeGenMetadata, MessageField, MethodMetadata};
use std::collections::HashSet;

/// Analyze collected metadata and create a generation plan
pub fn analyze_metadata(
    metadata: &CodeGenMetadata,
) -> Result<GenerationPlan, Box<dyn std::error::Error>> {
    let mut services = Vec::new();

    for (service_name, methods) in metadata.services() {
        let service_plan = analyze_service(&service_name, methods)?;
        services.push(service_plan);
    }

    Ok(GenerationPlan { services })
}

/// Analyze a single service and create a service plan
fn analyze_service(
    service_name: &str,
    methods: Vec<&MethodMetadata>,
) -> Result<ServicePlan, Box<dyn std::error::Error>> {
    let handler_name = strings::service_to_handler_name(service_name);
    let base_path = strings::service_to_base_path(service_name);

    let mut method_plans = Vec::new();

    for method in methods {
        if let Some(method_plan) = analyze_method(method)? {
            method_plans.push(method_plan);
        } else {
            println!(
                "cargo:warning=Skipping method {}.{} - incomplete metadata",
                service_name, method.method_name
            );
        }
    }

    Ok(ServicePlan {
        service_name: service_name.to_string(),
        handler_name,
        base_path,
        methods: method_plans,
    })
}

/// Analyze a single method and create a method plan
pub fn analyze_method(
    method: &MethodMetadata,
) -> Result<Option<MethodPlan>, Box<dyn std::error::Error>> {
    // Check if we have the required metadata
    let operation_id = match method.operation_id() {
        Some(id) => id,
        None => {
            println!(
                "cargo:warning=Method {}.{} missing operation_id",
                method.service_name, method.method_name
            );
            return Ok(None);
        }
    };

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

    // Generate function names
    let handler_function_name = strings::operation_to_method_name(operation_id);
    let route_function_name = strings::operation_to_route_name(operation_id);

    // Get input message fields from metadata
    let input_fields = method.input_fields.clone();

    // Extract parameters based on HTTP rule
    let (path_params, query_params, body_fields) =
        extract_request_fields(method, &http_path, &input_fields)?;

    // Determine if method has response
    let request_type = method.request_type();
    let has_response = types::has_response_body(&request_type);

    Ok(Some(MethodPlan {
        metadata: method.clone(),
        handler_function_name,
        route_function_name,
        http_method,
        http_path,
        path_params,
        query_params,
        body_fields,
        has_response,
    }))
}

/// Extract request fields based on HTTP rule analysis
fn extract_request_fields(
    method: &MethodMetadata,
    http_path: &str,
    input_fields: &[MessageField],
) -> Result<(Vec<PathParam>, Vec<QueryParam>, Vec<BodyField>), Box<dyn std::error::Error>> {
    let mut path_params = Vec::new();
    let mut query_params = Vec::new();
    let mut body_fields = Vec::new();

    // Extract path parameters from URL template in order
    let path_param_names_ordered: Vec<String> = paths::extract_path_parameters(http_path);
    let _path_param_names: HashSet<String> = path_param_names_ordered.iter().cloned().collect();

    // Get body field specification from HTTP rule
    let body_spec = method
        .http_rule
        .as_ref()
        .map(|rule| rule.body.as_str())
        .unwrap_or("");

    // Track which fields we've already processed to avoid duplicates
    let mut processed_fields = std::collections::HashSet::new();

    // First, add path parameters in URL order
    for path_param_name in &path_param_names_ordered {
        let field = paths::find_matching_field_for_path_param(path_param_name, input_fields);
        if let Some(field) = field {
            path_params.push(PathParam {
                name: field.name.clone(),
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

        if requests::should_be_body_field(field_name, body_spec) {
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

    // Add standard pagination parameters for list operations ONLY if not already present
    if requests::needs_pagination(&method.request_type()) {
        if !processed_fields.contains("max_results") {
            query_params.push(QueryParam {
                name: "max_results".to_string(),
                rust_type: types::make_optional("i32"),
                optional: true,
            });
        }
        if !processed_fields.contains("page_token") {
            query_params.push(QueryParam {
                name: "page_token".to_string(),
                rust_type: types::make_optional("String"),
                optional: true,
            });
        }
    }

    Ok((path_params, query_params, body_fields))
}

/// Validate that a generation plan is complete and consistent
pub fn validate_plan(plan: &GenerationPlan) -> Result<(), Box<dyn std::error::Error>> {
    crate::utils::validation::validate_plan(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MethodMetadata, gnostic::openapi::v3::Operation, google::api::HttpRule};

    fn create_test_metadata() -> MethodMetadata {
        let operation = Operation {
            operation_id: "ListCatalogs".to_string(),
            ..Default::default()
        };

        let http_rule = HttpRule {
            pattern: Some(crate::google::api::http_rule::Pattern::Get(
                "/catalogs".to_string(),
            )),
            ..Default::default()
        };

        MethodMetadata {
            service_name: "CatalogsService".to_string(),
            method_name: "ListCatalogs".to_string(),
            input_type: ".unitycatalog.catalogs.v1.ListCatalogsRequest".to_string(),
            output_type: ".unitycatalog.catalogs.v1.ListCatalogsResponse".to_string(),
            operation: Some(operation),
            http_rule: Some(http_rule),
            input_fields: Vec::new(),
        }
    }

    #[test]
    fn test_path_parameter_ordering() {
        // Test that path parameters are extracted in URL order, not struct field order
        let input_fields = vec![
            // Note: fields are in different order than they appear in URL
            MessageField {
                name: "name".to_string(), // This appears last in URL
                field_type: "TYPE_STRING".to_string(),
                optional: false,
                oneof_name: None,
            },
            MessageField {
                name: "share".to_string(), // This appears first in URL
                field_type: "TYPE_STRING".to_string(),
                optional: false,
                oneof_name: None,
            },
            MessageField {
                name: "schema".to_string(), // This appears second in URL
                field_type: "TYPE_STRING".to_string(),
                optional: false,
                oneof_name: None,
            },
        ];

        let operation = Operation {
            operation_id: "GetTableMetadata".to_string(),
            ..Default::default()
        };

        let http_rule = HttpRule {
            pattern: Some(crate::google::api::http_rule::Pattern::Get(
                "/shares/{share}/schemas/{schema}/tables/{name}/metadata".to_string(),
            )),
            ..Default::default()
        };

        let method = MethodMetadata {
            service_name: "SharingService".to_string(),
            method_name: "GetTableMetadata".to_string(),
            input_type: ".unitycatalog.sharing.v1.GetTableMetadataRequest".to_string(),
            output_type: ".unitycatalog.sharing.v1.QueryResponse".to_string(),
            operation: Some(operation),
            http_rule: Some(http_rule),
            input_fields,
        };

        let (path_params, query_params, body_fields) = extract_request_fields(
            &method,
            "/shares/{share}/schemas/{schema}/tables/{name}/metadata",
            &method.input_fields,
        )
        .unwrap();

        assert_eq!(path_params.len(), 3);
        assert_eq!(query_params.len(), 0);
        assert_eq!(body_fields.len(), 0);

        // Verify path parameters are in URL order: share, schema, name
        assert_eq!(path_params[0].name, "share");
        assert_eq!(path_params[1].name, "schema");
        assert_eq!(path_params[2].name, "name");
    }

    #[test]
    fn test_credential_fields_extraction() {
        // Test ListCredentialsRequest - should have purpose as query param and no oneof
        let list_fields = vec![
            MessageField {
                name: "max_results".to_string(),
                field_type: "TYPE_INT32".to_string(),
                optional: true,
                oneof_name: None,
            },
            MessageField {
                name: "page_token".to_string(),
                field_type: "TYPE_STRING".to_string(),
                optional: true,
                oneof_name: None,
            },
            MessageField {
                name: "purpose".to_string(),
                field_type: "TYPE_ENUM:.unitycatalog.credentials.v1.Purpose".to_string(),
                optional: true,
                oneof_name: None,
            },
        ];

        let list_operation = Operation {
            operation_id: "ListCredentials".to_string(),
            ..Default::default()
        };

        let list_http_rule = HttpRule {
            pattern: Some(crate::google::api::http_rule::Pattern::Get(
                "/credentials".to_string(),
            )),
            ..Default::default()
        };

        let list_method = MethodMetadata {
            service_name: "CredentialsService".to_string(),
            method_name: "ListCredentials".to_string(),
            input_type: ".unitycatalog.credentials.v1.ListCredentialsRequest".to_string(),
            output_type: ".unitycatalog.credentials.v1.ListCredentialsResponse".to_string(),
            operation: Some(list_operation),
            http_rule: Some(list_http_rule),
            input_fields: list_fields,
        };

        let (path_params, query_params, body_fields) =
            extract_request_fields(&list_method, "/credentials", &list_method.input_fields)
                .unwrap();

        assert_eq!(path_params.len(), 0);
        assert_eq!(query_params.len(), 3); // max_results, page_token, purpose
        assert_eq!(body_fields.len(), 0);

        // Test CreateCredentialRequest - should have oneof field
        let create_fields = vec![
            MessageField {
                name: "name".to_string(),
                field_type: "TYPE_STRING".to_string(),
                optional: false,
                oneof_name: None,
            },
            MessageField {
                name: "purpose".to_string(),
                field_type: "TYPE_ENUM:.unitycatalog.credentials.v1.Purpose".to_string(),
                optional: false,
                oneof_name: None,
            },
            MessageField {
                name: "comment".to_string(),
                field_type: "TYPE_STRING".to_string(),
                optional: true,
                oneof_name: None,
            },
            MessageField {
                name: "read_only".to_string(),
                field_type: "TYPE_BOOL".to_string(),
                optional: true,
                oneof_name: None,
            },
            MessageField {
                name: "skip_validation".to_string(),
                field_type: "TYPE_BOOL".to_string(),
                optional: false,
                oneof_name: None,
            },
            MessageField {
                name: "credential".to_string(),
                field_type: "TYPE_ONEOF:createcredentialrequest::Credential".to_string(),
                optional: true,
                oneof_name: None,
            },
        ];

        let create_operation = Operation {
            operation_id: "CreateCredential".to_string(),
            ..Default::default()
        };

        let create_http_rule = HttpRule {
            pattern: Some(crate::google::api::http_rule::Pattern::Post(
                "/credentials".to_string(),
            )),
            body: "*".to_string(),
            ..Default::default()
        };

        let create_method = MethodMetadata {
            service_name: "CredentialsService".to_string(),
            method_name: "CreateCredential".to_string(),
            input_type: ".unitycatalog.credentials.v1.CreateCredentialRequest".to_string(),
            output_type: ".unitycatalog.credentials.v1.CredentialInfo".to_string(),
            operation: Some(create_operation),
            http_rule: Some(create_http_rule),
            input_fields: create_fields,
        };

        let (path_params, query_params, body_fields) =
            extract_request_fields(&create_method, "/credentials", &create_method.input_fields)
                .unwrap();

        assert_eq!(path_params.len(), 0);
        assert_eq!(query_params.len(), 0);
        assert_eq!(body_fields.len(), 6); // All fields go to body because body: "*"

        // Check that credential field is detected as oneof type
        let credential_field = body_fields.iter().find(|f| f.name == "credential").unwrap();
        assert!(credential_field.rust_type.contains("::"));
        assert!(credential_field.optional);
    }

    #[test]
    fn test_analyze_method() {
        let method = create_test_metadata();
        let result = analyze_method(&method).unwrap();

        assert!(result.is_some());
        let plan = result.unwrap();

        assert_eq!(plan.handler_function_name, "list_catalogs");
        assert_eq!(plan.route_function_name, "list_catalogs_handler");
        assert_eq!(plan.http_method, "GET");
        assert_eq!(plan.http_path, "/catalogs");
        assert!(plan.has_response);
    }

    #[test]
    fn test_extract_request_fields() {
        let mut method = create_test_metadata();
        method.input_fields = vec![
            MessageField {
                name: "max_results".to_string(),
                field_type: "TYPE_INT32".to_string(),
                optional: true,
                oneof_name: None,
            },
            MessageField {
                name: "page_token".to_string(),
                field_type: "TYPE_STRING".to_string(),
                optional: true,
                oneof_name: None,
            },
        ];

        let input_fields = vec![
            MessageField {
                name: "max_results".to_string(),
                field_type: "TYPE_INT32".to_string(),
                optional: true,
                oneof_name: None,
            },
            MessageField {
                name: "page_token".to_string(),
                field_type: "TYPE_STRING".to_string(),
                optional: true,
                oneof_name: None,
            },
        ];

        let (path_params, query_params, body_fields) =
            extract_request_fields(&method, "/catalogs", &input_fields).unwrap();

        assert_eq!(path_params.len(), 0);
        assert_eq!(query_params.len(), 2); // Only 2 from fields, no duplicates
        assert_eq!(body_fields.len(), 0);

        // Verify proper optionality
        assert_eq!(query_params[0].rust_type, "Option<i32>");
        assert_eq!(query_params[1].rust_type, "Option<String>");
        assert!(query_params[0].optional);
        assert!(query_params[1].optional);
    }

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
