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

use crate::analysis::analyze_metadata;
use crate::output;
use crate::parsing::CodeGenMetadata;

pub mod generation;
pub mod templates;

/// Main entry point for code generation
///
/// Takes collected metadata and generates all necessary Rust code for REST handlers.
pub fn generate_code(
    metadata: &CodeGenMetadata,
    output_dir_common: &Path,
    output_dir_server: &Path,
    output_dir_client: &Path,
    output_dir_python: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Analyze metadata and plan generation
    let plan = analyze_metadata(metadata)?;

    // Generate code from plan
    let common_code = generation::generate_common_code(&plan)?;
    output::write_generated_code(&common_code, output_dir_common)?;

    // Generate server
    let server_code = generation::generate_server_code(&plan)?;
    output::write_generated_code(&server_code, output_dir_server)?;

    // Generate client
    let client_code = generation::generate_client_code(&plan)?;
    output::write_generated_code(&client_code, output_dir_client)?;

    // Generate Python bindings if output directory is provided
    let python_code = generation::generate_python_code(&plan)?;
    output::write_generated_code(&python_code, output_dir_python)?;

    Ok(())
}

/// Generated code ready for output
#[derive(Debug)]
pub struct GeneratedCode {
    /// Generated files mapped by relative path
    pub files: HashMap<String, String>,
}
