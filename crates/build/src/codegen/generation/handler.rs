use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::format_tokens;
use crate::codegen::{MethodHandler, ServiceHandler};

/// Generate handler trait for a service
pub(super) fn generate(service: &ServiceHandler<'_>) -> Result<String, Box<dyn std::error::Error>> {
    let mut trait_methods = Vec::new();
    for method in service.methods() {
        let method_code = handler_trait_method(&method);
        trait_methods.push(method_code);
    }

    let trait_code = handler_trait(service, &service.plan.handler_name, &trait_methods);
    let module_header = generate_module_header(service);

    Ok(format!("{}{}", module_header, trait_code))
}

/// Generate module-level `//!` documentation for the handler module
fn generate_module_header(service: &ServiceHandler<'_>) -> String {
    let mut lines = vec![
        format!("//! Handler trait for [`{}`].", service.plan.handler_name),
        "//!".to_string(),
        "//! Implement this trait to provide a custom backend for this service.".to_string(),
        "//! Register your implementation with the generated route setup functions.".to_string(),
        "//!".to_string(),
        "//! # Composability".to_string(),
        "//!".to_string(),
        "//! A single struct can implement multiple handler traits to serve multiple".to_string(),
        "//! services. Use [`axum::Router::merge`] to compose routers together.".to_string(),
    ];
    if let Some(doc) = service.plan.documentation.as_deref() {
        lines.push("//!".to_string());
        for line in doc.trim().lines() {
            let line = line.trim();
            if line.is_empty() {
                lines.push("//!".to_string());
            } else {
                lines.push(format!("//! {}", line));
            }
        }
    }
    lines.push(String::new());
    lines.join("\n")
}

/// Convert optional documentation into `#[doc = "..."]` token stream attributes.
///
/// `prettyplease` renders `#[doc = " text"]` as `/// text`. The leading space is required.
fn doc_tokens(documentation: Option<&str>) -> TokenStream {
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

/// Generate handler trait definition
pub fn handler_trait(
    service: &ServiceHandler<'_>,
    trait_name: &str,
    methods: &[TokenStream],
) -> String {
    let trait_ident = format_ident!("{}", trait_name);
    let mod_path = service.models_path();

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
pub fn handler_trait_method(method: &MethodHandler<'_>) -> TokenStream {
    let doc_attrs = doc_tokens(method.plan.metadata.documentation.as_deref());
    let input_type = method.input_type();
    let method_name = method.plan.base_method_ident();

    if method.plan.has_response {
        let output_type = method.output_type();
        quote! {
            #doc_attrs
            async fn #method_name(
                &self,
                request: #input_type,
                context: RequestContext,
            ) -> Result<#output_type>;
        }
    } else {
        quote! {
            #doc_attrs
            async fn #method_name(
                &self,
                request: #input_type,
                context: RequestContext,
            ) -> Result<()>;
        }
    }
}
