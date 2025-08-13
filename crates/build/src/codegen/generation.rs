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
use super::{GeneratedCode, GenerationPlan, ServicePlan};

mod client;
mod handler;
mod server;

/// Generate all Rust code from the generation plan
pub fn generate_code(plan: &GenerationPlan) -> Result<GeneratedCode, Box<dyn std::error::Error>> {
    let mut files = HashMap::new();

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
    // Generate handler trait
    let trait_code = handler::generate(service)?;
    files.insert(format!("{}/handler.rs", service.base_path), trait_code);

    // Generate server code
    let server_code = server::generate(service)?;
    files.insert(format!("{}/server.rs", service.base_path), server_code);

    // Generate client code
    let client_code = client::generate(service)?;
    files.insert(format!("{}/client.rs", service.base_path), client_code);

    // Generate service module
    let module_code = generate_service_module(service)?;
    files.insert(format!("{}/mod.rs", service.base_path), module_code);

    Ok(())
}

/// Generate service module that exports all components
fn generate_service_module(service: &ServicePlan) -> Result<String, Box<dyn std::error::Error>> {
    let module_code = templates::service_module(&service.handler_name);

    Ok(module_code)
}

/// Generate main module file that ties all services together
fn generate_main_module(
    services: &[ServicePlan],
    files: &mut HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let module_code = templates::main_module(services);
    files.insert("mod.rs".to_string(), module_code);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::MethodPlan;
    use super::*;
    use crate::{
        MessageField, MethodMetadata, codegen::QueryParam, gnostic::openapi::v3::Operation,
        google::api::HttpRule,
    };

    pub(crate) fn create_test_service_plan() -> ServicePlan {
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
            query_params: vec![
                QueryParam {
                    name: "max_results".to_string(),
                    rust_type: "Option<i32>".to_string(),
                    optional: true,
                },
                QueryParam {
                    name: "page_token".to_string(),
                    rust_type: "Option<String>".to_string(),
                    optional: true,
                },
            ],
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
    fn test_generate_service_code() {
        let service = create_test_service_plan();
        let mut files = HashMap::new();
        let result = generate_service_code(&service, &mut files);
        assert!(result.is_ok());

        assert!(files.contains_key("catalogs/handler.rs"));
        assert!(files.contains_key("catalogs/server.rs"));
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
}
