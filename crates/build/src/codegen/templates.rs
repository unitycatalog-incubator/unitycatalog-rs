//! Templates module for generating Rust code snippets
//!
//! This module contains all the code templates used to generate various parts
//! of the REST handler implementation. Templates are functions that take
//! structured data and return formatted Rust code strings using AST-based
//! code generation with syn and quote.
//!
//! ## Template Categories
//!
//! - **Handler Traits**: Async trait definitions for service operations
//! - **Route Handlers**: Axum handler functions that delegate to traits
//! - **Request Extractors**: FromRequest/FromRequestParts implementations
//! - **Client Code**: HTTP client method implementations
//! - **Module Structure**: Module definitions and exports

use prettyplease;
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{File, Path, Type};

use super::{BodyField, MethodPlan, PathParam, QueryParam, ServicePlan};
use crate::RequestType;
use crate::utils::strings;

/// Generate handler trait definition
pub fn handler_trait(trait_name: &str, methods: &[TokenStream], service_base: String) -> String {
    let trait_ident = format_ident!("{}", trait_name);
    let mod_path: Path = syn::parse_str(&format!("crate::models::{}::v1", service_base)).unwrap();

    let tokens = quote! {
        use async_trait::async_trait;

        use crate::Result;
        use crate::api::RequestContext;
        use #mod_path::*;

        #[async_trait]
        pub trait #trait_ident: Send + Sync + 'static {
            #(#methods)*
        }
    };

    format_tokens(tokens)
}

/// Generate a single handler trait method
pub fn handler_trait_method(method: &MethodPlan) -> TokenStream {
    let input_type = extract_type_ident(&method.metadata.input_type);
    let method_name = format_ident!("{}", method.handler_function_name);

    if method.has_response {
        let output_type = extract_type_ident(&method.metadata.output_type);
        quote! {
            async fn #method_name(
                &self,
                request: #input_type,
                context: RequestContext,
            ) -> Result<#output_type>;
        }
    } else {
        quote! {
            async fn #method_name(
                &self,
                request: #input_type,
                context: RequestContext,
            ) -> Result<()>;
        }
    }
}

/// Generate route handler function
pub fn route_handler_function(method: &MethodPlan, handler_trait: &str) -> String {
    let function_name = format_ident!("{}", method.route_function_name);
    let handler_method = format_ident!("{}", method.handler_function_name);
    let input_type = extract_type_ident(&method.metadata.input_type);
    let handler_trait_ident = format_ident!("{}", handler_trait);

    let tokens = if method.has_response {
        let output_type = extract_type_ident(&method.metadata.output_type);
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

    format_tokens(tokens)
}

/// Generate FromRequestParts implementation for path/query parameters
pub fn from_request_parts_impl(method: &MethodPlan) -> Result<String, Box<dyn std::error::Error>> {
    let input_type = extract_type_ident(&method.metadata.input_type);
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

    Ok(format_tokens(tokens))
}

/// Generate FromRequest implementation for JSON body
pub fn from_request_impl(method: &MethodPlan) -> Result<String, Box<dyn std::error::Error>> {
    let input_type = extract_type_ident(&method.metadata.input_type);

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

        Ok(format_tokens(tokens))
    }
}

/// Generate hybrid FromRequest implementation for methods with path/query + body
fn generate_hybrid_request_impl(method: &MethodPlan) -> Result<String, Box<dyn std::error::Error>> {
    let input_type = extract_type_ident(&method.metadata.input_type);
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

        Ok(format_tokens(tokens))
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

        Ok(format_tokens(tokens))
    }
}

/// Generate client method implementation
pub fn client_method(method: &MethodPlan) -> String {
    let method_name = format_ident!("{}", method.handler_function_name);
    let input_type = strings::extract_simple_type_name(&method.metadata.input_type);
    let input_type_ident = format_ident!("{}", input_type);
    let http_method = format_ident!("{}", method.http_method.to_lowercase());
    let url_formatting = generate_url_formatting(&method.http_path, &method.path_params);

    let body_handling = if matches!(
        method.metadata.request_type(),
        RequestType::Create | RequestType::Update
    ) {
        quote! { .json(request) }
    } else {
        quote! {}
    };

    let tokens = if method.has_response {
        let output_type = strings::extract_simple_type_name(&method.metadata.output_type);
        let output_type_ident = format_ident!("{}", output_type);
        quote! {
            pub async fn #method_name(&self, request: &#input_type_ident) -> crate::Result<#output_type_ident> {
                #url_formatting
                let response = self.client.#http_method(url)#body_handling.send().await?;
                response.error_for_status_ref()?;
                let result = response.bytes().await?;
                Ok(serde_json::from_slice(&result)?)
            }
        }
    } else {
        quote! {
            pub async fn #method_name(&self, request: &#input_type_ident) -> crate::Result<()> {
                #url_formatting
                let response = self.client.#http_method(url)#body_handling.send().await?;
                response.error_for_status()?;
                Ok(())
            }
        }
    };

    format_tokens(tokens)
}

/// Generate client struct definition
pub fn client_struct(client_name: &str, methods: &[String], service_namespace: &str) -> String {
    let client_ident = format_ident!("{}", client_name);
    let method_tokens: Vec<TokenStream> = methods
        .iter()
        .map(|m| syn::parse_str::<TokenStream>(m).unwrap_or_else(|_| quote! {}))
        .collect();
    let mod_path: Path =
        syn::parse_str(&format!("crate::models::{}::v1", service_namespace)).unwrap();

    let tokens = quote! {
        use cloud_client::CloudClient;
        use url::Url;

        use #mod_path::*;

        /// HTTP client for service operations
        #[derive(Clone)]
        pub struct #client_ident {
            pub(crate) client: CloudClient,
            pub(crate) base_url: Url,
        }

        impl #client_ident {
            /// Create a new client instance
            pub fn new(client: CloudClient, base_url: Url) -> Self {
                Self { client, base_url }
            }

            #(#method_tokens)*
        }
    };

    format_tokens(tokens)
}

/// Generate route handlers module
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

    format_tokens(tokens)
}

/// Generate service module
pub fn service_module(handler_name: &str) -> String {
    let handler_ident = format_ident!("{}", handler_name);

    let tokens = quote! {
        pub use handler::#handler_ident;
        pub use client::*;

        mod handler;
        #[cfg(feature = "axum")]
        pub mod server;
        pub mod client;
    };

    tokens.to_string()
}

/// Generate main module file
pub fn main_module(services: &[ServicePlan]) -> String {
    let service_modules: Vec<TokenStream> = services
        .iter()
        .map(|s| {
            let module_name = format_ident!("{}", s.base_path);
            quote! { pub mod #module_name; }
        })
        .collect();

    let service_exports: Vec<TokenStream> = services
        .iter()
        .map(|s| {
            let module_name = format_ident!("{}", s.base_path);
            quote! { pub use #module_name::*; }
        })
        .collect();

    let tokens = quote! {
        // Service modules
        #(#service_modules)*

        // Re-exports
        #(#service_exports)*
    };

    format_tokens(tokens)
}

/// Generate error types
pub fn error_types() -> String {
    let tokens = quote! {
        //! Error types for generated handlers

        use thiserror::Error;

        /// Result type used throughout the generated code
        pub type Result<T> = std::result::Result<T, Error>;

        /// Error type for handler operations
        #[derive(Error, Debug)]
        pub enum Error {
            #[error("Generic error: {0}")]
            Generic(String),

            #[error("Not found: {0}")]
            NotFound(String),

            #[error("Permission denied")]
            PermissionDenied,

            #[error("Invalid request: {0}")]
            InvalidRequest(String),
        }

        impl axum::response::IntoResponse for Error {
            fn into_response(self) -> axum::response::Response {
                use axum::http::StatusCode;

                let (status, message) = match self {
                    Error::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
                    Error::PermissionDenied => (StatusCode::FORBIDDEN, "Permission denied".to_string()),
                    Error::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
                    Error::Generic(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
                };

                (status, message).into_response()
            }
        }
    };

    format_tokens(tokens)
}

/// Generate protobuf exports
pub fn proto_exports() -> String {
    let tokens = quote! {
        //! Protobuf type exports

        // Re-export generated protobuf types
        pub use crate::models::gen::unitycatalog::*;
    };

    format_tokens(tokens)
}

// Helper functions

/// Extract the final type name from a fully qualified protobuf type and convert to Ident
fn extract_type_ident(full_type: &str) -> Ident {
    let type_name = full_type.split('.').last().unwrap_or(full_type);
    format_ident!("{}", type_name)
}

/// Generate path parameter extractions as TokenStream
fn generate_path_extractions_tokens(params: &[PathParam], is_request: bool) -> TokenStream {
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
fn generate_query_extractions_tokens(params: &[QueryParam]) -> TokenStream {
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
fn generate_field_assignments_tokens(
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

/// Generate URL formatting code that properly substitutes path parameters
fn generate_url_formatting(path: &str, params: &[PathParam]) -> proc_macro2::TokenStream {
    if params.is_empty() {
        return quote! {
            let url = self.base_url.join(#path)?;
        };
    }

    let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
    let (format_string, format_args) = crate::utils::paths::format_url_template(path, &param_names);

    if format_args.is_empty() {
        quote! {
            let url = self.base_url.join(#path)?;
        }
    } else {
        let field_idents: Vec<_> = format_args
            .iter()
            .map(|arg| format_ident!("{}", arg))
            .collect();
        quote! {
            let formatted_path = format!(#format_string, #(request.#field_idents),*);
            let url = self.base_url.join(&formatted_path)?;
        }
    }
}

/// Helper function to format TokenStream as properly formatted Rust code
fn format_tokens(tokens: TokenStream) -> String {
    let tokens_string = tokens.to_string();

    let syntax_tree = syn::parse2::<File>(tokens).unwrap_or_else(|_| {
        // Fallback to basic token string if parsing fails
        syn::parse_str::<File>(&tokens_string).unwrap_or_else(|_| {
            syn::parse_quote! {
                // Failed to parse generated code
            }
        })
    });

    prettyplease::unparse(&syntax_tree)
}
