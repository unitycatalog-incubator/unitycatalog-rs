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
use quote::{format_ident, quote};
use syn::{File, Path};

use super::ServicePlan;

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
pub(crate) fn extract_type_ident(full_type: &str) -> Ident {
    let type_name = full_type.split('.').next_back().unwrap_or(full_type);
    format_ident!("{}", type_name)
}

/// Helper function to format TokenStream as properly formatted Rust code
pub(crate) fn format_tokens(tokens: TokenStream) -> String {
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
