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

use proc_macro2::Ident;
use quote::format_ident;

/// Extract the final type name from a fully qualified protobuf type and convert to Ident
pub(crate) fn extract_type_ident(full_type: &str) -> Ident {
    let type_name = full_type.split('.').next_back().unwrap_or(full_type);
    format_ident!("{}", type_name)
}
