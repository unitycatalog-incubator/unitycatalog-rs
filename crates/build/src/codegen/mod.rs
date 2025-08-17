//! Code generation module for Unity Catalog REST handlers
//!
//! This module provides the core functionality for generating Rust code from
//! protobuf metadata extracted from Unity Catalog service definitions.
//!
//! ## Architecture
//!
//! The code generation process follows these phases:
//! 1. **Analysis**: Process collected metadata to understand service structure
//! 2. **Planning**: Determine what code needs to be generated
//! 3. **Generation**: Create Rust code using templates and metadata
//! 4. **Output**: Write generated code to appropriate files
//!
//! ## Generated Code Types
//!
//! - **Handler Traits**: Async trait definitions for service operations
//! - **Request Extractors**: Axum FromRequest/FromRequestParts implementations
//! - **Route Handlers**: Axum handler functions that delegate to traits
//! - **Client Code**: HTTP client implementations for services
//! - **Type Mappings**: Conversions between protobuf and Rust types

use std::collections::HashMap;
use std::path::Path;

use crate::{CodeGenMetadata, MethodMetadata};

pub mod analysis;
pub mod generation;
pub mod output;
pub mod templates;

/// Main entry point for code generation
///
/// Takes collected metadata and generates all necessary Rust code for REST handlers.
pub fn generate_rest_handlers(
    metadata: &CodeGenMetadata,
    output_dir_common: &Path,
    output_dir_server: &Path,
    output_dir_client: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Analyze metadata and plan generation
    let plan = analysis::analyze_metadata(metadata)?;

    // Generate code from plan
    let common_code = generation::generate_common_code(&plan)?;
    output::write_generated_code(&common_code, output_dir_common)?;

    // Generate server
    let server_code = generation::generate_server_code(&plan)?;
    output::write_generated_code(&server_code, output_dir_server)?;

    // Generate client
    let client_code = generation::generate_client_code(&plan)?;
    output::write_generated_code(&client_code, output_dir_client)?;

    Ok(())
}

/// High-level plan for what code to generate
#[derive(Debug)]
pub struct GenerationPlan {
    /// Services to generate handlers for
    pub services: Vec<ServicePlan>,
}

/// Plan for generating code for a single service
#[derive(Debug)]
pub struct ServicePlan {
    /// Service name (e.g., "CatalogsService")
    pub service_name: String,
    /// Handler trait name (e.g., "CatalogHandler")
    pub handler_name: String,
    /// Base URL path for this service (e.g., "catalogs")
    pub base_path: String,
    /// Methods to generate for this service
    pub methods: Vec<MethodPlan>,
}

/// Plan for generating code for a single method
#[derive(Debug)]
pub struct MethodPlan {
    /// Original method metadata
    pub metadata: MethodMetadata,
    /// Rust function name for the handler method
    pub handler_function_name: String,
    /// Rust function name for the route handler
    pub route_function_name: String,
    /// HTTP method and path for routing
    pub http_method: String,
    pub http_path: String,
    /// Path parameters extracted from the URL template
    pub path_params: Vec<PathParam>,
    /// Query parameters (for List operations)
    pub query_params: Vec<QueryParam>,
    /// Body fields that should be extracted from request body
    pub body_fields: Vec<BodyField>,
    /// Whether this method returns a response body
    pub has_response: bool,
}

/// A path parameter in a URL template
#[derive(Debug)]
pub struct PathParam {
    /// Template parameter name (e.g., "name" from "/catalogs/{name}")
    pub template_param: String,
    /// Field name in the request struct (e.g., "full_name")
    pub field_name: String,
    /// Rust type for this parameter
    pub rust_type: String,
}

/// A query parameter for HTTP requests
#[derive(Debug)]
pub struct QueryParam {
    /// Parameter name
    pub name: String,
    /// Rust type for this parameter
    pub rust_type: String,
    /// Whether this parameter is optional
    pub optional: bool,
}

/// A body field that should be extracted from request body
#[derive(Debug)]
pub struct BodyField {
    /// Field name
    pub name: String,
    /// Rust type for this field
    pub rust_type: String,
    /// Whether this field is optional
    pub optional: bool,
}

/// Generated code ready for output
#[derive(Debug)]
pub struct GeneratedCode {
    /// Generated files mapped by relative path
    pub files: HashMap<String, String>,
}

// Utils module moved to crate::utils - use that instead
pub use crate::utils::strings as utils;

// Tests moved to crate::utils module
