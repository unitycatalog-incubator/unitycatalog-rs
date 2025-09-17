use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{Path, Type};

use super::{extract_type_ident, format_tokens};
use crate::{
    analysis::{BodyField, MethodPlan, PathParam, QueryParam, RequestType},
    codegen::ServiceHandler,
    google::api::http_rule::Pattern,
};

/// Generate server side code for axum servers
///
/// This geneartes:
/// - FromRequestParts extractor implementations for path/query parameters
/// - FromRequest extractor implementations for JSON body
pub(super) fn generate_common(service: &ServiceHandler<'_>) -> String {
    let extractor_impls = service
        .methods()
        .map(|method| from_request_extractor(method.plan))
        .collect_vec();

    let mod_path: Path =
        syn::parse_str(&format!("crate::models::{}::v1", &service.plan.base_path)).unwrap();

    let tokens = quote! {
        #![allow(unused_mut)]
        use crate::Result;
        use #mod_path::*;
        use axum::{RequestExt, RequestPartsExt};

        #(#extractor_impls)*
    };

    format_tokens(tokens)
}

pub(super) fn generate_server(service: &ServiceHandler<'_>) -> String {
    let mut handler_function_impls = Vec::new();
    for method in &service.plan.methods {
        let handler_code = route_handler_function(method, &service.plan.handler_name);
        handler_function_impls.push(handler_code);
    }

    let mod_path: Path = syn::parse_str(&format!(
        "unitycatalog_common::models::{}::v1",
        &service.plan.base_path
    ))
    .unwrap();
    let trait_path: Path =
        syn::parse_str(&format!("super::handler::{}", &service.plan.handler_name)).unwrap();

    let tokens = quote! {
        #![allow(unused_mut)]
        use crate::Result;
        use crate::api::RequestContext;
        use #mod_path::*;
        use #trait_path;
        use crate::policy::Recipient;
        use axum::extract::{State, Extension};

        #(#handler_function_impls)*

    };

    format_tokens(tokens)
}

/// Generate extractor implementation for a specific method
fn from_request_extractor(method: &MethodPlan) -> TokenStream {
    match &method.request_type {
        RequestType::List | RequestType::Get | RequestType::Delete => {
            // These use FromRequestParts for path/query parameters
            from_request_parts_impl(method)
        }
        RequestType::Create | RequestType::Update => {
            // These use FromRequest for JSON body
            from_request_impl(method)
        }
        RequestType::Custom(pattern) => match pattern {
            Pattern::Get(_) | Pattern::Delete(_) => from_request_parts_impl(method),
            Pattern::Post(_) | Pattern::Patch(_) => from_request_impl(method),
            Pattern::Custom(_) => todo!("Implement custom request type"),
            Pattern::Put(_) => todo!("Implement PUT request type"),
        },
    }
}

/// Generate route handler function
fn route_handler_function(method: &MethodPlan, handler_trait: &str) -> TokenStream {
    let handler_method = format_ident!("{}", method.handler_function_name);
    let input_type = extract_type_ident(&method.metadata.input_type);
    let handler_trait_ident = format_ident!("{}", handler_trait);

    if method.has_response {
        let output_type = extract_type_ident(&method.metadata.output_type);
        quote! {
            pub async fn #handler_method<T: #handler_trait_ident>(
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
            pub async fn #handler_method<T: #handler_trait_ident>(
                State(handler): State<T>,
                Extension(recipient): Extension<Recipient>,
                request: #input_type,
            ) -> Result<()> {
                let context = RequestContext { recipient };
                handler.#handler_method(request, context).await?;
                Ok(())
            }
        }
    }
}

/// Generate FromRequestParts implementation for path/query parameters
fn from_request_parts_impl(method: &MethodPlan) -> TokenStream {
    let input_type = extract_type_ident(&method.metadata.input_type);
    let path_extractions = path_extractions(method, false);
    let query_extractions = query_extractions(method);
    let field_assignments = field_assignments_plain(method);

    quote! {
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
    }
}

/// Generate FromRequest implementation for JSON body
fn from_request_impl(method: &MethodPlan) -> TokenStream {
    let input_type = extract_type_ident(&method.metadata.input_type);

    // Check if we need a hybrid extractor (path/query + body)
    if !method.path_params.is_empty() || !method.query_params.is_empty() {
        // Generate hybrid implementation
        generate_hybrid_request_impl(method)
    } else {
        // Simple JSON body extraction
        quote! {
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
        }
    }
}

/// Generate hybrid FromRequest implementation for methods with path/query + body
fn generate_hybrid_request_impl(method: &MethodPlan) -> TokenStream {
    let input_type = extract_type_ident(&method.metadata.input_type);
    let path_extractions = path_extractions(method, true);
    let query_extractions = query_extractions(method);

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

        quote! {
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
        }
    } else {
        // Use traditional destructuring for regular fields
        let body_extractions = generate_body_extractions_tokens(&method.body_fields, &input_type);
        let field_assignments = field_assignments_plain(method);

        quote! {
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
        }
    }
}

/// Generate path parameter extractions as TokenStream
fn path_extractions(method: &MethodPlan, is_request: bool) -> TokenStream {
    let params = &method.path_parameters().collect_vec();

    if params.is_empty() {
        quote! {}
    } else {
        let param_names: Vec<Ident> = params
            .iter()
            .map(|p| format_ident!("{}", p.field_name))
            .collect();
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
fn query_extractions(method: &MethodPlan) -> TokenStream {
    let params = method.query_parameters().collect_vec();
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
fn field_assignments_plain(method: &MethodPlan) -> TokenStream {
    let assignments = method.parameters.iter().map(|param| {
        let ident = param.field_ident();
        quote! { #ident }
    });
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
        let name = format_ident!("{}", param.field_name);
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
