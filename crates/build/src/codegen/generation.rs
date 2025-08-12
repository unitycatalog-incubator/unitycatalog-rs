//! Code generation module for producing Rust code from analyzed plans
//!
//! This module takes the analyzed generation plans and produces actual Rust code
//! using templates and structured generation. It handles:
//!
//! - Handler trait generation
//! - Route handler function generation
//! - Request extractor implementations
//! - Client code generation
//! - Module organization and exports

use std::collections::HashMap;

use super::templates;
use super::{GeneratedCode, GenerationPlan, MethodPlan, ServicePlan};
use crate::RequestType;

/// Generate all Rust code from the generation plan
pub fn generate_code(plan: &GenerationPlan) -> Result<GeneratedCode, Box<dyn std::error::Error>> {
    let mut files = HashMap::new();

    println!(
        "cargo:warning=Generating code for {} services",
        plan.services.len()
    );

    // Generate code for each service
    for service in &plan.services {
        generate_service_code(service, &mut files)?;
    }

    // Generate the main module file that ties everything together
    generate_main_module(&plan.services, &mut files)?;

    Ok(GeneratedCode { files })
}

/// Generate code for a single service
fn generate_service_code(
    service: &ServicePlan,
    files: &mut HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "cargo:warning=Generating code for service {} with {} methods",
        service.service_name,
        service.methods.len()
    );

    // Generate handler trait
    let trait_code = generate_handler_trait(service)?;
    files.insert(format!("{}/handler.rs", service.base_path), trait_code);

    // Generate route handlers
    let route_code = generate_route_handlers(service)?;
    files.insert(format!("{}/routes.rs", service.base_path), route_code);

    // Generate request extractors
    let extractor_code = generate_request_extractors(service)?;
    files.insert(
        format!("{}/extractors.rs", service.base_path),
        extractor_code,
    );

    // Generate client code
    let client_code = generate_client_code(service)?;
    files.insert(format!("{}/client.rs", service.base_path), client_code);

    // Generate service module
    let module_code = generate_service_module(service)?;
    files.insert(format!("{}/mod.rs", service.base_path), module_code);

    Ok(())
}

/// Generate handler trait for a service
fn generate_handler_trait(service: &ServicePlan) -> Result<String, Box<dyn std::error::Error>> {
    let mut trait_methods = Vec::new();

    for method in &service.methods {
        let method_code = templates::handler_trait_method(method);
        trait_methods.push(method_code);
    }

    let trait_code = templates::handler_trait(
        &service.handler_name,
        &trait_methods,
        service.base_path.clone(),
    );

    println!(
        "cargo:warning=Generated handler trait {} with {} methods",
        service.handler_name,
        service.methods.len()
    );

    Ok(trait_code)
}

/// Generate route handler functions for a service
fn generate_route_handlers(service: &ServicePlan) -> Result<String, Box<dyn std::error::Error>> {
    let mut handler_functions = Vec::new();

    for method in &service.methods {
        let handler_code = templates::route_handler_function(method, &service.handler_name);
        handler_functions.push(handler_code);
    }

    let module_code = templates::route_handlers_module(
        &service.handler_name,
        &handler_functions,
        &service.base_path,
    );

    println!(
        "cargo:warning=Generated {} route handlers for {}",
        service.methods.len(),
        service.service_name
    );

    Ok(module_code)
}

/// Generate request extractor implementations for a service
fn generate_request_extractors(
    service: &ServicePlan,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut extractor_impls = Vec::new();

    for method in &service.methods {
        let extractor_code = generate_extractor_for_method(method)?;
        extractor_impls.push(extractor_code);
    }

    let module_code = templates::request_extractors_module(&extractor_impls, &service.base_path);

    println!(
        "cargo:warning=Generated {} request extractors for {}",
        service.methods.len(),
        service.service_name
    );

    Ok(module_code)
}

/// Generate client code for a service
fn generate_client_code(service: &ServicePlan) -> Result<String, Box<dyn std::error::Error>> {
    let mut client_methods = Vec::new();

    for method in &service.methods {
        let method_code = templates::client_method(method);
        client_methods.push(method_code);
    }

    let client_name = format!(
        "{}Client",
        service
            .handler_name
            .strip_suffix("Handler")
            .unwrap_or(&service.handler_name)
    );
    let client_code = templates::client_struct(&client_name, &client_methods, &service.base_path);

    println!(
        "cargo:warning=Generated client {} with {} methods",
        client_name,
        service.methods.len()
    );

    Ok(client_code)
}

/// Generate service module that exports all components
fn generate_service_module(service: &ServicePlan) -> Result<String, Box<dyn std::error::Error>> {
    let module_code =
        templates::service_module(&service.handler_name, &service.base_path, &service.methods);

    Ok(module_code)
}

/// Generate main module file that ties all services together
fn generate_main_module(
    services: &[ServicePlan],
    files: &mut HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let module_code = templates::main_module(services);
    files.insert("mod.rs".to_string(), module_code);

    println!("cargo:warning=Generated main module file");
    Ok(())
}

/// Generate extractor implementation for a specific method
fn generate_extractor_for_method(
    method: &MethodPlan,
) -> Result<String, Box<dyn std::error::Error>> {
    match method.metadata.request_type() {
        RequestType::List | RequestType::Get | RequestType::Delete => {
            // These use FromRequestParts for path/query parameters
            templates::from_request_parts_impl(method)
        }
        RequestType::Create | RequestType::Update => {
            // These use FromRequest for JSON body
            templates::from_request_impl(method)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        MessageField, MethodMetadata, gnostic::openapi::v3::Operation, google::api::HttpRule,
    };

    fn create_test_service_plan() -> ServicePlan {
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

        let metadata = MethodMetadata {
            service_name: "CatalogsService".to_string(),
            method_name: "ListCatalogs".to_string(),
            input_type: ".unitycatalog.catalogs.v1.ListCatalogsRequest".to_string(),
            output_type: ".unitycatalog.catalogs.v1.ListCatalogsResponse".to_string(),
            operation: Some(operation),
            http_rule: Some(http_rule),
            input_fields: vec![
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
            ],
        };

        let method_plan = MethodPlan {
            metadata,
            handler_function_name: "list_catalogs".to_string(),
            route_function_name: "list_catalogs_handler".to_string(),
            http_method: "GET".to_string(),
            http_path: "/catalogs".to_string(),
            path_params: vec![],
            query_params: vec![],
            body_fields: vec![],
            has_response: true,
        };

        ServicePlan {
            service_name: "CatalogsService".to_string(),
            handler_name: "CatalogHandler".to_string(),
            base_path: "catalogs".to_string(),
            methods: vec![method_plan],
        }
    }

    #[test]
    fn test_generate_handler_trait() {
        let service = create_test_service_plan();
        let result = generate_handler_trait(&service);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("CatalogHandler"));
        assert!(code.contains("list_catalogs"));
    }

    #[test]
    fn test_generated_code_format() {
        let service = create_test_service_plan();
        let result = generate_handler_trait(&service);
        assert!(result.is_ok());
        let code = result.unwrap();

        // Print generated code to verify format
        println!("Generated handler trait:\n{}", code);

        // Verify the code contains expected elements
        assert!(code.contains("pub trait CatalogHandler"));
        assert!(code.contains("async fn list_catalogs"));
        assert!(code.contains("RequestContext"));
        assert!(code.contains("async_trait"));

        // Verify proper Rust syntax (no extra escaping or formatting issues)
        assert!(!code.contains("\\n"));
        assert!(!code.contains("\\t"));
        assert!(!code.contains("\\\""));
    }

    #[test]
    fn test_generate_service_code() {
        let service = create_test_service_plan();
        let mut files = HashMap::new();
        let result = generate_service_code(&service, &mut files);
        assert!(result.is_ok());

        assert!(files.contains_key("catalogs/handler.rs"));
        assert!(files.contains_key("catalogs/routes.rs"));
        assert!(files.contains_key("catalogs/extractors.rs"));
        assert!(files.contains_key("catalogs/client.rs"));
        assert!(files.contains_key("catalogs/mod.rs"));
    }

    #[test]
    fn test_field_extraction_scenarios() {
        // Test case 1: Path parameters only (GET request)
        let get_operation = Operation {
            operation_id: "GetCatalog".to_string(),
            ..Default::default()
        };

        let get_http_rule = HttpRule {
            pattern: Some(crate::google::api::http_rule::Pattern::Get(
                "/catalogs/{name}".to_string(),
            )),
            body: "".to_string(),
            ..Default::default()
        };

        let get_metadata = MethodMetadata {
            service_name: "CatalogsService".to_string(),
            method_name: "GetCatalog".to_string(),
            input_type: ".unitycatalog.catalogs.v1.GetCatalogRequest".to_string(),
            output_type: ".unitycatalog.catalogs.v1.CatalogInfo".to_string(),
            operation: Some(get_operation),
            http_rule: Some(get_http_rule),
            input_fields: vec![
                MessageField {
                    name: "name".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: false,
                    oneof_name: None,
                },
                MessageField {
                    name: "include_browse".to_string(),
                    field_type: "TYPE_BOOL".to_string(),
                    optional: true,
                    oneof_name: None,
                },
            ],
        };

        let get_plan = crate::codegen::analysis::analyze_method(&get_metadata)
            .unwrap()
            .unwrap();
        assert_eq!(get_plan.path_params.len(), 1);
        assert_eq!(get_plan.path_params[0].name, "name");
        assert_eq!(get_plan.query_params.len(), 1);
        assert_eq!(get_plan.query_params[0].name, "include_browse");
        assert_eq!(get_plan.body_fields.len(), 0);

        // Test case 2: Body fields only (POST request)
        let post_operation = Operation {
            operation_id: "CreateCatalog".to_string(),
            ..Default::default()
        };

        let post_http_rule = HttpRule {
            pattern: Some(crate::google::api::http_rule::Pattern::Post(
                "/catalogs".to_string(),
            )),
            body: "*".to_string(),
            ..Default::default()
        };

        let post_metadata = MethodMetadata {
            service_name: "CatalogsService".to_string(),
            method_name: "CreateCatalog".to_string(),
            input_type: ".unitycatalog.catalogs.v1.CreateCatalogRequest".to_string(),
            output_type: ".unitycatalog.catalogs.v1.CatalogInfo".to_string(),
            operation: Some(post_operation),
            http_rule: Some(post_http_rule),
            input_fields: vec![
                MessageField {
                    name: "name".to_string(),
                    field_type: "TYPE_STRING".to_string(),
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
                    name: "properties".to_string(),
                    field_type: "TYPE_MESSAGE:.unitycatalog.Properties".to_string(),
                    optional: true,
                    oneof_name: None,
                },
            ],
        };

        let post_plan = crate::codegen::analysis::analyze_method(&post_metadata)
            .unwrap()
            .unwrap();
        assert_eq!(post_plan.path_params.len(), 0);
        assert_eq!(post_plan.query_params.len(), 0);
        assert_eq!(post_plan.body_fields.len(), 3);
        assert_eq!(post_plan.body_fields[0].name, "name");
        assert_eq!(post_plan.body_fields[1].name, "comment");
        assert_eq!(post_plan.body_fields[2].name, "properties");

        // Test case 3: Mixed parameters (UPDATE request with path and body)
        let update_operation = Operation {
            operation_id: "UpdateCatalog".to_string(),
            ..Default::default()
        };

        let update_http_rule = HttpRule {
            pattern: Some(crate::google::api::http_rule::Pattern::Patch(
                "/catalogs/{name}".to_string(),
            )),
            body: "catalog".to_string(),
            ..Default::default()
        };

        let update_metadata = MethodMetadata {
            service_name: "CatalogsService".to_string(),
            method_name: "UpdateCatalog".to_string(),
            input_type: ".unitycatalog.catalogs.v1.UpdateCatalogRequest".to_string(),
            output_type: ".unitycatalog.catalogs.v1.CatalogInfo".to_string(),
            operation: Some(update_operation),
            http_rule: Some(update_http_rule),
            input_fields: vec![
                MessageField {
                    name: "name".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: false,
                    oneof_name: None,
                },
                MessageField {
                    name: "catalog".to_string(),
                    field_type: "TYPE_MESSAGE:.unitycatalog.CatalogInfo".to_string(),
                    optional: false,
                    oneof_name: None,
                },
                MessageField {
                    name: "force".to_string(),
                    field_type: "TYPE_BOOL".to_string(),
                    optional: true,
                    oneof_name: None,
                },
            ],
        };

        let update_plan = crate::codegen::analysis::analyze_method(&update_metadata)
            .unwrap()
            .unwrap();
        assert_eq!(update_plan.path_params.len(), 1);
        assert_eq!(update_plan.path_params[0].name, "name");
        assert_eq!(update_plan.query_params.len(), 1);
        assert_eq!(update_plan.query_params[0].name, "force");
        assert_eq!(update_plan.body_fields.len(), 1);
        assert_eq!(update_plan.body_fields[0].name, "catalog");
    }

    #[test]
    fn test_generate_client_code() {
        let service = create_test_service_plan();
        let result = generate_client_code(&service);
        assert!(result.is_ok());
        let code = result.unwrap();

        // Print generated client code to verify format
        println!("Generated client code:\n{}", code);

        // Verify the code contains expected elements
        assert!(code.contains("pub struct CatalogClient"));
        assert!(code.contains("pub async fn list_catalogs"));
        assert!(code.contains("CloudClient"));
        assert!(code.contains("impl CatalogClient"));

        // Verify proper Rust syntax
        assert!(!code.contains("\\n"));
        assert!(!code.contains("\\t"));
        assert!(!code.contains("\\\""));
    }

    #[test]
    fn test_generated_extractor_field_mapping() {
        // Create a method with mixed field types for comprehensive testing
        let operation = Operation {
            operation_id: "UpdateCatalog".to_string(),
            ..Default::default()
        };

        let http_rule = HttpRule {
            pattern: Some(crate::google::api::http_rule::Pattern::Patch(
                "/catalogs/{name}".to_string(),
            )),
            body: "catalog".to_string(),
            ..Default::default()
        };

        let metadata = MethodMetadata {
            service_name: "CatalogsService".to_string(),
            method_name: "UpdateCatalog".to_string(),
            input_type: ".unitycatalog.catalogs.v1.UpdateCatalogRequest".to_string(),
            output_type: ".unitycatalog.catalogs.v1.CatalogInfo".to_string(),
            operation: Some(operation),
            http_rule: Some(http_rule),
            input_fields: vec![
                MessageField {
                    name: "name".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: false,
                    oneof_name: None,
                },
                MessageField {
                    name: "catalog".to_string(),
                    field_type: "TYPE_MESSAGE:.unitycatalog.CatalogInfo".to_string(),
                    optional: false,
                    oneof_name: None,
                },
                MessageField {
                    name: "force".to_string(),
                    field_type: "TYPE_BOOL".to_string(),
                    optional: true,
                    oneof_name: None,
                },
            ],
        };

        let method_plan = crate::codegen::analysis::analyze_method(&metadata)
            .unwrap()
            .unwrap();

        // Test hybrid extractor generation (has path, query, and body fields)
        let extractor_code = templates::from_request_impl(&method_plan).unwrap();

        println!("Generated extractor code:\n{}", extractor_code);

        // Verify the extractor includes all field types
        assert!(extractor_code.contains("let (mut parts, body) = req.into_parts();"));
        assert!(extractor_code.contains("axum::extract::Path"));
        assert!(extractor_code.contains("axum::extract::Query"));
        assert!(extractor_code.contains("axum::extract::Json"));

        // Verify field assignments include all fields
        assert!(extractor_code.contains("name,"));
        assert!(extractor_code.contains("catalog,"));
        assert!(extractor_code.contains("force,"));

        // Verify struct construction
        assert!(extractor_code.contains("UpdateCatalogRequest {"));
    }

    #[test]
    fn test_no_duplicate_fields_and_correct_optionality() {
        // Test a List operation with existing max_results field to ensure no duplication
        let operation = Operation {
            operation_id: "ListCatalogs".to_string(),
            ..Default::default()
        };

        let http_rule = HttpRule {
            pattern: Some(crate::google::api::http_rule::Pattern::Get(
                "/catalogs".to_string(),
            )),
            body: "".to_string(),
            ..Default::default()
        };

        let metadata = MethodMetadata {
            service_name: "CatalogsService".to_string(),
            method_name: "ListCatalogs".to_string(),
            input_type: ".unitycatalog.catalogs.v1.ListCatalogsRequest".to_string(),
            output_type: ".unitycatalog.catalogs.v1.ListCatalogsResponse".to_string(),
            operation: Some(operation),
            http_rule: Some(http_rule),
            input_fields: vec![
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
                    name: "parent".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: false,
                    oneof_name: None,
                },
                MessageField {
                    name: "include_browse".to_string(),
                    field_type: "TYPE_BOOL".to_string(),
                    optional: true,
                    oneof_name: None,
                },
            ],
        };

        let method_plan = crate::codegen::analysis::analyze_method(&metadata)
            .unwrap()
            .unwrap();

        // Verify no duplicate fields
        let query_field_names: std::collections::HashSet<_> =
            method_plan.query_params.iter().map(|p| &p.name).collect();

        // Should have exactly 4 unique query fields
        assert_eq!(query_field_names.len(), 4);
        assert!(query_field_names.contains(&"max_results".to_string()));
        assert!(query_field_names.contains(&"page_token".to_string()));
        assert!(query_field_names.contains(&"parent".to_string()));
        assert!(query_field_names.contains(&"include_browse".to_string()));

        // Verify correct optionality
        for param in &method_plan.query_params {
            match param.name.as_str() {
                "max_results" => {
                    assert_eq!(param.rust_type, "Option<i32>");
                    assert!(param.optional);
                }
                "page_token" => {
                    assert_eq!(param.rust_type, "Option<String>");
                    assert!(param.optional);
                }
                "parent" => {
                    assert_eq!(param.rust_type, "String");
                    assert!(!param.optional);
                }
                "include_browse" => {
                    assert_eq!(param.rust_type, "Option<bool>");
                    assert!(param.optional);
                }
                _ => panic!("Unexpected query parameter: {}", param.name),
            }
        }

        // Test the generated extractor
        let extractor_code = templates::from_request_parts_impl(&method_plan).unwrap();

        println!("Generated list extractor code:\n{}", extractor_code);

        // Verify serde(default) is used for optional fields
        assert!(extractor_code.contains("#[serde(default)]"));

        // Verify no duplicate field assignments
        let field_count = extractor_code.matches("max_results").count();
        assert_eq!(field_count, 3); // Once in struct definition, once in destructuring, once in assignment

        let page_token_count = extractor_code.matches("page_token").count();
        assert_eq!(page_token_count, 3); // Once in struct definition, once in destructuring, once in assignment
    }

    #[test]
    fn test_edge_case_field_scenarios() {
        // Test edge case: List operation with no existing max_results/page_token fields
        let operation = Operation {
            operation_id: "ListSchemas".to_string(),
            ..Default::default()
        };

        let http_rule = HttpRule {
            pattern: Some(crate::google::api::http_rule::Pattern::Get(
                "/catalogs/{catalog_name}/schemas".to_string(),
            )),
            body: "".to_string(),
            ..Default::default()
        };

        let metadata = MethodMetadata {
            service_name: "SchemasService".to_string(),
            method_name: "ListSchemas".to_string(),
            input_type: ".unitycatalog.schemas.v1.ListSchemasRequest".to_string(),
            output_type: ".unitycatalog.schemas.v1.ListSchemasResponse".to_string(),
            operation: Some(operation),
            http_rule: Some(http_rule),
            input_fields: vec![
                MessageField {
                    name: "catalog_name".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: false,
                    oneof_name: None,
                },
                MessageField {
                    name: "name_pattern".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: true,
                    oneof_name: None,
                },
            ],
        };

        let method_plan = crate::codegen::analysis::analyze_method(&metadata)
            .unwrap()
            .unwrap();

        // Should have 1 path param, 3 query params (name_pattern + auto-added pagination)
        assert_eq!(method_plan.path_params.len(), 1);
        assert_eq!(method_plan.query_params.len(), 3);
        assert_eq!(method_plan.body_fields.len(), 0);

        // Verify path param
        assert_eq!(method_plan.path_params[0].name, "catalog_name");
        assert_eq!(method_plan.path_params[0].rust_type, "String");

        // Verify query params include both original and auto-added pagination
        let query_names: std::collections::HashSet<_> =
            method_plan.query_params.iter().map(|p| &p.name).collect();
        assert!(query_names.contains(&"name_pattern".to_string()));
        assert!(query_names.contains(&"max_results".to_string()));
        assert!(query_names.contains(&"page_token".to_string()));

        // Verify correct typing for each field
        for param in &method_plan.query_params {
            match param.name.as_str() {
                "name_pattern" => {
                    assert_eq!(param.rust_type, "Option<String>");
                    assert!(param.optional);
                }
                "max_results" => {
                    assert_eq!(param.rust_type, "Option<i32>");
                    assert!(param.optional);
                }
                "page_token" => {
                    assert_eq!(param.rust_type, "Option<String>");
                    assert!(param.optional);
                }
                _ => panic!("Unexpected query parameter: {}", param.name),
            }
        }

        // Test that generated extractor includes all fields without duplication
        let extractor_code = templates::from_request_parts_impl(&method_plan).unwrap();

        // Verify each field appears exactly 3 times (struct def, destructure, assignment)
        let name_pattern_count = extractor_code.matches("name_pattern").count();
        assert_eq!(name_pattern_count, 3);

        let max_results_count = extractor_code.matches("max_results").count();
        assert_eq!(max_results_count, 3);

        let page_token_count = extractor_code.matches("page_token").count();
        assert_eq!(page_token_count, 3);

        let catalog_name_count = extractor_code.matches("catalog_name").count();
        assert_eq!(catalog_name_count, 2); // Once in path extraction, once in assignment
    }

    #[test]
    fn test_required_query_parameters() {
        // Test case: ListTables with required catalog_name and schema_name query parameters
        let operation = Operation {
            operation_id: "ListTables".to_string(),
            ..Default::default()
        };

        let http_rule = HttpRule {
            pattern: Some(crate::google::api::http_rule::Pattern::Get(
                "/tables".to_string(),
            )),
            body: "".to_string(),
            ..Default::default()
        };

        let metadata = MethodMetadata {
            service_name: "TablesService".to_string(),
            method_name: "ListTables".to_string(),
            input_type: ".unitycatalog.tables.v1.ListTablesRequest".to_string(),
            output_type: ".unitycatalog.tables.v1.ListTablesResponse".to_string(),
            operation: Some(operation),
            http_rule: Some(http_rule),
            input_fields: vec![
                MessageField {
                    name: "catalog_name".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: false, // Required field
                    oneof_name: None,
                },
                MessageField {
                    name: "schema_name".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: false, // Required field
                    oneof_name: None,
                },
                MessageField {
                    name: "max_results".to_string(),
                    field_type: "TYPE_INT32".to_string(),
                    optional: true, // Optional field
                    oneof_name: None,
                },
                MessageField {
                    name: "page_token".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: true, // Optional field
                    oneof_name: None,
                },
            ],
        };

        let method_plan = crate::codegen::analysis::analyze_method(&metadata)
            .unwrap()
            .unwrap();

        println!("Query params for ListTables:");
        for param in &method_plan.query_params {
            println!(
                "  {}: {} (optional: {})",
                param.name, param.rust_type, param.optional
            );
        }

        // Verify required fields are NOT wrapped in Option<T>
        for param in &method_plan.query_params {
            match param.name.as_str() {
                "catalog_name" => {
                    assert_eq!(param.rust_type, "String");
                    assert!(!param.optional);
                }
                "schema_name" => {
                    assert_eq!(param.rust_type, "String");
                    assert!(!param.optional);
                }
                "max_results" => {
                    assert_eq!(param.rust_type, "Option<i32>");
                    assert!(param.optional);
                }
                "page_token" => {
                    assert_eq!(param.rust_type, "Option<String>");
                    assert!(param.optional);
                }
                _ => panic!("Unexpected query parameter: {}", param.name),
            }
        }

        // Test the generated extractor
        let extractor_code = templates::from_request_parts_impl(&method_plan).unwrap();
        println!("Generated ListTables extractor:\n{}", extractor_code);

        // Verify required fields don't have #[serde(default)]
        assert!(extractor_code.contains("catalog_name: String,"));
        assert!(extractor_code.contains("schema_name: String,"));

        // Verify optional fields have #[serde(default)]
        assert!(
            extractor_code.contains("#[serde(default)]\n            max_results: Option<i32>,")
        );
        assert!(
            extractor_code.contains("#[serde(default)]\n            page_token: Option<String>,")
        );
    }

    #[test]
    fn test_proto3_field_optionality() {
        // Test case: Realistic Proto3 message with mixed required/optional fields
        let operation = Operation {
            operation_id: "ListTables".to_string(),
            ..Default::default()
        };

        let http_rule = HttpRule {
            pattern: Some(crate::google::api::http_rule::Pattern::Get(
                "/tables".to_string(),
            )),
            body: "".to_string(),
            ..Default::default()
        };

        let metadata = MethodMetadata {
            service_name: "TablesService".to_string(),
            method_name: "ListTables".to_string(),
            input_type: ".unitycatalog.tables.v1.ListTablesRequest".to_string(),
            output_type: ".unitycatalog.tables.v1.ListTablesResponse".to_string(),
            operation: Some(operation),
            http_rule: Some(http_rule),
            input_fields: vec![
                // These should be required in Proto3 (no proto3_optional flag)
                MessageField {
                    name: "catalog_name".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: false,
                    oneof_name: None,
                },
                MessageField {
                    name: "schema_name".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: false,
                    oneof_name: None,
                },
                // These should be optional (with proto3_optional flag)
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
                    name: "include_history".to_string(),
                    field_type: "TYPE_BOOL".to_string(),
                    optional: true,
                    oneof_name: None,
                },
            ],
        };

        let method_plan = crate::codegen::analysis::analyze_method(&metadata)
            .unwrap()
            .unwrap();

        // Test the generated extractor
        let extractor_code = templates::from_request_parts_impl(&method_plan).unwrap();
        println!("Generated Proto3 extractor:\n{}", extractor_code);

        // Required fields should NOT have #[serde(default)]
        assert!(extractor_code.contains("catalog_name: String,"));
        assert!(extractor_code.contains("schema_name: String,"));

        // Optional fields should have #[serde(default)]
        assert!(extractor_code.contains("#[serde(default)]"));
        assert!(extractor_code.contains("max_results: Option<i32>,"));
        assert!(extractor_code.contains("page_token: Option<String>,"));
        assert!(extractor_code.contains("include_history: Option<bool>,"));

        // Verify no #[serde(default)] on required fields
        let lines: Vec<&str> = extractor_code.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.trim() == "catalog_name: String," || line.trim() == "schema_name: String," {
                // Check that the previous line is not #[serde(default)]
                if i > 0 {
                    assert!(
                        !lines[i - 1].trim().contains("#[serde(default)]"),
                        "Required field should not have #[serde(default)]: {}",
                        line
                    );
                }
            }
        }
    }
}
