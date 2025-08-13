use std::collections::HashMap;

use prettyplease;
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{File, Path, Type};

use super::super::{BodyField, MethodPlan, PathParam, QueryParam, ServicePlan, templates};
use super::{GeneratedCode, GenerationPlan};
use crate::RequestType;

/// Generate server side code for axum servers
///
/// This geneartes:
/// - FromRequestParts extractor implementations for path/query parameters
/// - FromRequest extractor implementations for JSON body
/// - Route handler functions
pub(super) fn generate(service: &ServicePlan) -> Result<String, Box<dyn std::error::Error>> {
    let mut handler_functions = Vec::new();
    for method in &service.methods {
        let handler_code = route_handler_function(method, &service.handler_name);
        handler_functions.push(handler_code);
    }

    let mut extractor_impls = Vec::new();
    for method in &service.methods {
        let extractor_code = generate_extractor_for_method(method)?;
        extractor_impls.push(extractor_code);
    }

    let module_code = server_module(
        &service.handler_name,
        &handler_functions,
        &extractor_impls,
        &service.base_path,
    );

    Ok(module_code)
}

/// Generate server module
pub fn server_module(
    trait_name: &str,
    handlers: &[String],
    extractors: &[String],
    service_namespace: &str,
) -> String {
    let handler_tokens: Vec<TokenStream> = handlers
        .iter()
        .map(|h| syn::parse_str::<TokenStream>(h).unwrap_or_else(|_| quote! {}))
        .collect();
    let extractor_tokens: Vec<TokenStream> = extractors
        .iter()
        .map(|e| syn::parse_str::<TokenStream>(e).unwrap_or_else(|_| quote! {}))
        .collect();
    let mod_path: Path =
        syn::parse_str(&format!("crate::models::{}::v1", service_namespace)).unwrap();
    let trait_path: Path = syn::parse_str(&format!("super::handler::{}", trait_name)).unwrap();

    let tokens = quote! {
        use crate::Result;
        use crate::api::RequestContext;
        use #mod_path::*;
        use #trait_path;
        use crate::services::Recipient;
        use axum::{RequestExt, RequestPartsExt};
        use axum::extract::{State, Extension};

        #(#handler_tokens)*

        #(#extractor_tokens)*
    };

    templates::format_tokens(tokens)
}

/// Generate extractor implementation for a specific method
fn generate_extractor_for_method(
    method: &MethodPlan,
) -> Result<String, Box<dyn std::error::Error>> {
    match method.metadata.request_type() {
        RequestType::List | RequestType::Get | RequestType::Delete => {
            // These use FromRequestParts for path/query parameters
            from_request_parts_impl(method)
        }
        RequestType::Create | RequestType::Update => {
            // These use FromRequest for JSON body
            from_request_impl(method)
        }
    }
}

/// Generate route handler function
fn route_handler_function(method: &MethodPlan, handler_trait: &str) -> String {
    let function_name = format_ident!("{}", method.route_function_name);
    let handler_method = format_ident!("{}", method.handler_function_name);
    let input_type = templates::extract_type_ident(&method.metadata.input_type);
    let handler_trait_ident = format_ident!("{}", handler_trait);

    let tokens = if method.has_response {
        let output_type = templates::extract_type_ident(&method.metadata.output_type);
        quote! {
            pub async fn #function_name<T: #handler_trait_ident>(
                State(handler): State<T>,
                Extension(recipient): Extension<Recipient>,
                request: #input_type,
            ) -> Result<::axum::Json<#output_type>> {
                let context = RequestContext { recipient };
                let result = handler.#handler_method(request, context).await?;
                Ok(axum::Json(result))
            }
        }
    } else {
        quote! {
            pub async fn #function_name<T: #handler_trait_ident>(
                State(handler): State<T>,
                Extension(recipient): Extension<Recipient>,
                request: #input_type,
            ) -> Result<()> {
                let context = RequestContext { recipient };
                handler.#handler_method(request, context).await?;
                Ok(())
            }
        }
    };

    templates::format_tokens(tokens)
}

/// Generate FromRequestParts implementation for path/query parameters
pub fn from_request_parts_impl(method: &MethodPlan) -> Result<String, Box<dyn std::error::Error>> {
    let input_type = templates::extract_type_ident(&method.metadata.input_type);
    let path_extractions = generate_path_extractions_tokens(&method.path_params, false);
    let query_extractions = generate_query_extractions_tokens(&method.query_params);
    let field_assignments = generate_field_assignments_tokens(
        &method.path_params,
        &method.query_params,
        &method.body_fields,
    );

    let tokens = quote! {
        impl<S: Send + Sync> axum::extract::FromRequestParts<S> for #input_type {
            type Rejection = crate::Error;

            async fn from_request_parts(
                parts: &mut axum::http::request::Parts,
                _state: &S,
            ) -> Result<Self, Self::Rejection> {
                #path_extractions
                #query_extractions

                Ok(#input_type {
                    #field_assignments
                })
            }
        }
    };

    Ok(templates::format_tokens(tokens))
}

/// Generate FromRequest implementation for JSON body
pub fn from_request_impl(method: &MethodPlan) -> Result<String, Box<dyn std::error::Error>> {
    let input_type = templates::extract_type_ident(&method.metadata.input_type);

    // Check if we need a hybrid extractor (path/query + body)
    if !method.path_params.is_empty() || !method.query_params.is_empty() {
        // Generate hybrid implementation
        generate_hybrid_request_impl(method)
    } else {
        // Simple JSON body extraction
        let tokens = quote! {
            impl<S: Send + Sync> axum::extract::FromRequest<S> for #input_type {
                type Rejection = axum::response::Response;

                async fn from_request(
                    req: axum::extract::Request<axum::body::Body>,
                    _state: &S,
                ) -> Result<Self, Self::Rejection> {
                    let axum::extract::Json(request) = req
                        .extract()
                        .await
                        .map_err(axum::response::IntoResponse::into_response)?;
                    Ok(request)
                }
            }
        };

        Ok(templates::format_tokens(tokens))
    }
}

/// Generate hybrid FromRequest implementation for methods with path/query + body
fn generate_hybrid_request_impl(method: &MethodPlan) -> Result<String, Box<dyn std::error::Error>> {
    let input_type = templates::extract_type_ident(&method.metadata.input_type);
    let path_extractions = generate_path_extractions_tokens(&method.path_params, true);
    let query_extractions = generate_query_extractions_tokens(&method.query_params);

    // Check if we have any oneof fields
    let has_oneof_fields = method
        .body_fields
        .iter()
        .any(|f| f.rust_type.contains("::"));

    if has_oneof_fields {
        // Use mixed body extraction for oneof fields
        let body_extractions =
            generate_mixed_body_extractions_tokens(&method.body_fields, &input_type);
        let field_assignments = generate_mixed_field_assignments_tokens(
            &method.path_params,
            &method.query_params,
            &method.body_fields,
        );

        let tokens = quote! {
            impl<S: Send + Sync> axum::extract::FromRequest<S> for #input_type {
                type Rejection = axum::response::Response;

                async fn from_request(
                    mut req: axum::extract::Request<axum::body::Body>,
                    _state: &S,
                ) -> Result<Self, Self::Rejection> {
                    // Extract path and query parameters
                    let (mut parts, body) = req.into_parts();
                    #path_extractions
                    #query_extractions

                    // Extract body fields
                    let body_req = axum::extract::Request::from_parts(parts, body);
                    #body_extractions

                    Ok(#input_type {
                        #field_assignments
                    })
                }
            }
        };

        Ok(templates::format_tokens(tokens))
    } else {
        // Use traditional destructuring for regular fields
        let body_extractions = generate_body_extractions_tokens(&method.body_fields, &input_type);
        let field_assignments = generate_field_assignments_tokens(
            &method.path_params,
            &method.query_params,
            &method.body_fields,
        );

        let tokens = quote! {
            impl<S: Send + Sync> axum::extract::FromRequest<S> for #input_type {
                type Rejection = axum::response::Response;

                async fn from_request(
                    mut req: axum::extract::Request<axum::body::Body>,
                    _state: &S,
                ) -> Result<Self, Self::Rejection> {
                    // Extract path and query parameters
                    let (mut parts, body) = req.into_parts();
                    #path_extractions
                    #query_extractions

                    // Extract body fields
                    let body_req = axum::extract::Request::from_parts(parts, body);
                    #body_extractions

                    Ok(#input_type {
                        #field_assignments
                    })
                }
            }
        };

        Ok(templates::format_tokens(tokens))
    }
}

/// Generate path parameter extractions as TokenStream
pub(crate) fn generate_path_extractions_tokens(
    params: &[PathParam],
    is_request: bool,
) -> TokenStream {
    if params.is_empty() {
        quote! {}
    } else {
        let param_names: Vec<Ident> = params.iter().map(|p| format_ident!("{}", p.name)).collect();
        let param_types: Vec<TokenStream> = params
            .iter()
            .map(|p| {
                syn::parse_str::<Type>(&p.rust_type)
                    .unwrap()
                    .to_token_stream()
            })
            .collect();

        if is_request {
            quote! {
                let axum::extract::Path((#(#param_names),*)) = parts
                    .extract::<axum::extract::Path<(#(#param_types),*)>>()
                    .await
                    .map_err(axum::response::IntoResponse::into_response)?;
            }
        } else {
            quote! {
                let axum::extract::Path((#(#param_names),*)) = parts
                    .extract::<axum::extract::Path<(#(#param_types),*)>>()
                    .await?;
            }
        }
    }
}

/// Generate query parameter extractions as TokenStream
pub(crate) fn generate_query_extractions_tokens(params: &[QueryParam]) -> TokenStream {
    if params.is_empty() {
        quote! {}
    } else {
        let query_fields: Vec<TokenStream> = params
            .iter()
            .map(|p| {
                let name = format_ident!("{}", p.name);
                // Handle Option<T> types by parsing the inner type
                let type_tokens =
                    if p.rust_type.starts_with("Option<") && p.rust_type.ends_with(">") {
                        let inner_type = &p.rust_type[7..p.rust_type.len() - 1];
                        let inner = format_ident!("{}", inner_type);
                        quote! { Option<#inner> }
                    } else {
                        let type_ident = format_ident!("{}", p.rust_type);
                        quote! { #type_ident }
                    };

                if p.optional {
                    quote! { #[serde(default)] #name: #type_tokens }
                } else {
                    quote! { #name: #type_tokens }
                }
            })
            .collect();

        let param_names: Vec<Ident> = params.iter().map(|p| format_ident!("{}", p.name)).collect();

        quote! {
            #[derive(serde::Deserialize)]
            struct QueryParams {
                #(#query_fields,)*
            }
            let axum::extract::Query(QueryParams { #(#param_names),* }) = parts.extract::<axum::extract::Query<QueryParams>>().await?;
        }
    }
}

/// Generate field assignments for request struct construction as TokenStream
pub(crate) fn generate_field_assignments_tokens(
    path_params: &[PathParam],
    query_params: &[QueryParam],
    body_fields: &[BodyField],
) -> TokenStream {
    let mut assignments = Vec::new();

    for param in path_params {
        let name = format_ident!("{}", param.name);
        assignments.push(quote! { #name });
    }

    for param in query_params {
        let name = format_ident!("{}", param.name);
        assignments.push(quote! { #name });
    }

    for field in body_fields {
        let name = format_ident!("{}", field.name);
        assignments.push(quote! { #name });
    }

    quote! { #(#assignments,)* }
}

/// Generate body parameter extractions as TokenStream
fn generate_body_extractions_tokens(
    body_fields: &[BodyField],
    response_type: &Ident,
) -> TokenStream {
    if body_fields.is_empty() {
        quote! {}
    } else if body_fields.len() == 1 {
        // Single body field - extract directly
        let field_name = format_ident!("{}", body_fields[0].name);
        quote! {
            let axum::extract::Json(#field_name) = body_req
                .extract()
                .await
                .map_err(axum::response::IntoResponse::into_response)?;
        }
    } else {
        // Multiple body fields - extract as a struct and destructure
        let field_names: Vec<_> = body_fields
            .iter()
            .map(|f| format_ident!("{}", f.name))
            .collect();
        quote! {
            let axum::extract::Json::<#response_type>(body) = body_req
                .extract()
                .await
                .map_err(axum::response::IntoResponse::into_response)?;
            let (#(#field_names),*) = (
                #(body.#field_names),*
            );
        }
    }
}

/// Generate body parameter extractions for mixed fields (including oneof) as TokenStream
fn generate_mixed_body_extractions_tokens(
    body_fields: &[BodyField],
    response_type: &Ident,
) -> TokenStream {
    if body_fields.is_empty() {
        quote! {}
    } else {
        // Extract the full request body as a struct
        quote! {
            let axum::extract::Json::<#response_type>(body) = body_req
                .extract()
                .await
                .map_err(axum::response::IntoResponse::into_response)?;
        }
    }
}

/// Generate field assignments for mixed fields (including oneof) as TokenStream
fn generate_mixed_field_assignments_tokens(
    path_params: &[PathParam],
    query_params: &[QueryParam],
    body_fields: &[BodyField],
) -> TokenStream {
    let mut assignments = Vec::new();

    for param in path_params {
        let name = format_ident!("{}", param.name);
        assignments.push(quote! { #name });
    }

    for param in query_params {
        let name = format_ident!("{}", param.name);
        assignments.push(quote! { #name });
    }

    for field in body_fields {
        let name = format_ident!("{}", field.name);
        assignments.push(quote! { #name: body.#name });
    }

    quote! { #(#assignments,)* }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        MessageField, MethodMetadata, gnostic::openapi::v3::Operation, google::api::HttpRule,
    };

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
        let extractor_code = from_request_impl(&method_plan).unwrap();

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
        let extractor_code = from_request_parts_impl(&method_plan).unwrap();

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
        let extractor_code = from_request_parts_impl(&method_plan).unwrap();

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
        let extractor_code = from_request_parts_impl(&method_plan).unwrap();
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
        let extractor_code = from_request_parts_impl(&method_plan).unwrap();
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
