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

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::File;

use super::{CodeGenConfig, GeneratedCode};
use crate::analysis::MethodPlan;
use crate::analysis::{GenerationPlan, RequestType, ServicePlan};
use crate::codegen::ServiceHandler;
use crate::parsing::CodeGenMetadata;

mod builder;
mod client;
mod handler;
pub(crate) mod node;
mod python;
mod server;

impl MethodPlan {
    fn resource_client_method(&self) -> syn::Ident {
        match self.request_type {
            RequestType::Get => format_ident!("get"),
            RequestType::Update => format_ident!("update"),
            RequestType::Delete => format_ident!("delete"),
            _ => format_ident!("{}", self.handler_function_name),
        }
    }

    fn base_method_ident(&self) -> syn::Ident {
        format_ident!("{}", self.handler_function_name)
    }
}

pub fn generate_common_code(
    plan: &GenerationPlan,
    metadata: &CodeGenMetadata,
    config: &CodeGenConfig,
) -> Result<GeneratedCode, Box<dyn std::error::Error>> {
    let mut files = HashMap::new();

    // Generate code for each service
    for service in &plan.services {
        let handler = ServiceHandler {
            plan: service,
            metadata,
            config,
        };

        // Generate server code (FromRequestParts/FromRequest extractor impls)
        let server_code = server::generate_common(&handler);
        files.insert(format!("{}/server.rs", service.base_path), server_code);

        // Generate service module
        let module_code = generate_common_module();
        files.insert(format!("{}/mod.rs", service.base_path), module_code);
    }

    // Generate the main module file that ties everything together
    let module_code = main_module(&plan.services);
    files.insert("mod.rs".to_string(), module_code);

    Ok(GeneratedCode { files })
}

/// Generate service module that exports all components
fn generate_common_module() -> String {
    let tokens = quote! {
        #[cfg(feature = "axum")]
        pub mod server;
    };

    format_tokens(tokens)
}

pub fn generate_server_code(
    plan: &GenerationPlan,
    metadata: &CodeGenMetadata,
    config: &CodeGenConfig,
) -> Result<GeneratedCode, Box<dyn std::error::Error>> {
    let mut files = HashMap::new();

    // Generate code for each service
    for service in &plan.services {
        let handler = ServiceHandler {
            plan: service,
            metadata,
            config,
        };

        let trait_code = handler::generate(&handler)?;
        files.insert(format!("{}/handler.rs", service.base_path), trait_code);

        let server_code = server::generate_server(&handler);
        files.insert(format!("{}/server.rs", service.base_path), server_code);

        // Generate client code
        let module_code = generate_server_module(service);
        files.insert(format!("{}/mod.rs", service.base_path), module_code);
    }

    // Generate the main module file that ties everything together
    let module_code = main_module(&plan.services);
    files.insert("mod.rs".to_string(), module_code);

    Ok(GeneratedCode { files })
}

pub fn generate_python_code(
    plan: &GenerationPlan,
    metadata: &CodeGenMetadata,
    config: &CodeGenConfig,
) -> Result<GeneratedCode, Box<dyn std::error::Error>> {
    let mut files = HashMap::new();

    let services = plan.services.to_vec();

    let handlers = services
        .iter()
        .map(|service| ServiceHandler {
            plan: service,
            metadata,
            config,
        })
        .collect_vec();

    // Generate code for each service
    for service in &handlers {
        // Generate Python client code
        let python_code = python::generate(service);
        files.insert(format!("{}.rs", service.plan.base_path), python_code);
    }

    // Generate the main module file that ties everything together
    let module_code = python::main_module(&handlers);
    files.insert("mod.rs".to_string(), module_code);

    // Generate single unified typings (.pyi) file for all services
    let python_typings_code = python::generate_typings(&handlers);
    files.insert(
        config.output.python_typings_filename.clone(),
        python_typings_code,
    );

    Ok(GeneratedCode { files })
}

pub fn generate_node_code(
    plan: &GenerationPlan,
    metadata: &CodeGenMetadata,
    config: &CodeGenConfig,
) -> Result<GeneratedCode, Box<dyn std::error::Error>> {
    let mut files = HashMap::new();

    let services = plan.services.to_vec();

    let handlers = services
        .iter()
        .map(|service| ServiceHandler {
            plan: service,
            metadata,
            config,
        })
        .collect_vec();

    for service in &handlers {
        let napi_code = node::generate(service);
        files.insert(format!("{}.rs", service.plan.base_path), napi_code);
    }

    let module_code = node::main_module(&handlers);
    files.insert("mod.rs".to_string(), module_code);

    Ok(GeneratedCode { files })
}

pub fn generate_node_ts_code(
    plan: &GenerationPlan,
    metadata: &CodeGenMetadata,
    config: &CodeGenConfig,
) -> Result<GeneratedCode, Box<dyn std::error::Error>> {
    let services = plan.services.to_vec();

    let handlers = services
        .iter()
        .map(|service| ServiceHandler {
            plan: service,
            metadata,
            config,
        })
        .collect_vec();

    let ts_code = node::typescript::generate_client_ts(&handlers);
    let mut files = HashMap::new();
    files.insert("client.ts".to_string(), ts_code);

    Ok(GeneratedCode { files })
}

pub fn generate_client_code(
    plan: &GenerationPlan,
    metadata: &CodeGenMetadata,
    config: &CodeGenConfig,
) -> Result<GeneratedCode, Box<dyn std::error::Error>> {
    let mut files = HashMap::new();

    // Generate code for each service
    for service in &plan.services {
        let handler = ServiceHandler {
            plan: service,
            metadata,
            config,
        };

        // Generate client code
        let client_code = client::generate(&handler)?;
        files.insert(format!("{}/client.rs", service.base_path), client_code);

        let client_code = builder::generate(&handler)?;
        files.insert(format!("{}/builders.rs", service.base_path), client_code);

        // Generate service module
        let module_code = generate_client_module();
        files.insert(format!("{}/mod.rs", service.base_path), module_code);
    }

    // Generate the main module file that ties everything together
    let module_code = generate_client_main_module(&plan.services);
    files.insert("mod.rs".to_string(), module_code);

    Ok(GeneratedCode { files })
}

fn generate_server_module(service: &ServicePlan) -> String {
    let handler_ident = format_ident!("{}", service.handler_name);

    let tokens = quote! {
        pub use handler::#handler_ident;

        mod handler;
        #[cfg(feature = "axum")]
        pub mod server;
    };

    format_tokens(tokens)
}

fn generate_client_module() -> String {
    let tokens = quote! {
        pub use client::*;
        pub use builders::*;

        pub mod client;
        pub mod builders;
    };

    format_tokens(tokens)
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

    let tokens = quote! {
        // Service modules
        #(#service_modules)*

    };

    format_tokens(tokens)
}

/// Generate main module file for client codegen, including the `stream_paginated` utility.
///
/// This inlines the pagination helper so generated builders have no dependency on
/// `crate::utils::stream_paginated`.
fn generate_client_main_module(services: &[ServicePlan]) -> String {
    let service_modules: Vec<TokenStream> = services
        .iter()
        .map(|s| {
            let module_name = format_ident!("{}", s.base_path);
            quote! { pub mod #module_name; }
        })
        .collect();

    let tokens = quote! {
        #(#service_modules)*

        use futures::Future;

        pub(super) fn stream_paginated<F, Fut, S, T>(
            state: S,
            op: F,
        ) -> impl futures::Stream<Item = crate::Result<T>>
        where
            F: Fn(S, Option<String>) -> Fut + Copy,
            Fut: Future<Output = crate::Result<(T, S, Option<String>)>>,
        {
            enum PaginationState<T> {
                Start(T),
                HasMore(T, String),
                Done,
            }

            futures::stream::unfold(
                PaginationState::Start(state),
                move |state| async move {
                    let (s, page_token) = match state {
                        PaginationState::Start(s) => (s, None),
                        PaginationState::HasMore(s, page_token) if !page_token.is_empty() => {
                            (s, Some(page_token))
                        }
                        _ => {
                            return None;
                        }
                    };

                    let (resp, s, continuation) = match op(s, page_token).await {
                        Ok(resp) => resp,
                        Err(e) => return Some((Err(e), PaginationState::Done)),
                    };

                    let next_state = match continuation {
                        Some(token) => PaginationState::HasMore(s, token),
                        None => PaginationState::Done,
                    };

                    Some((Ok(resp), next_state))
                },
            )
        }
    };

    format_tokens(tokens)
}

/// Convert optional documentation into `#[doc = "..."]` token stream attributes.
///
/// `prettyplease` renders `#[doc = " text"]` as `/// text`. The leading space is required.
pub(super) fn doc_tokens(documentation: Option<&str>) -> TokenStream {
    let Some(doc) = documentation else {
        return quote! {};
    };
    let doc = doc.trim();
    if doc.is_empty() {
        return quote! {};
    }
    let attrs: Vec<TokenStream> = doc
        .lines()
        .map(|line| {
            let line = line.trim();
            if line.is_empty() {
                quote! { #[doc = ""] }
            } else {
                let spaced = format!(" {}", line);
                quote! { #[doc = #spaced] }
            }
        })
        .collect();
    quote! { #(#attrs)* }
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
