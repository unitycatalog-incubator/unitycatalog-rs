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

use convert_case::{Case, Casing};
use quote::format_ident;
use syn::Ident;

use crate::analysis::{ManagedResource, MethodPlan, RequestType, ServicePlan, analyze_metadata};
use crate::google::api::http_rule::Pattern;
use crate::output;
use crate::parsing::{CodeGenMetadata, MessageInfo};

pub mod generation;

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
    let common_code = generation::generate_common_code(&plan, metadata)?;
    output::write_generated_code(&common_code, output_dir_common)?;

    // Generate server
    let server_code = generation::generate_server_code(&plan, metadata)?;
    output::write_generated_code(&server_code, output_dir_server)?;

    // Generate client
    let client_code = generation::generate_client_code(&plan, metadata)?;
    output::write_generated_code(&client_code, output_dir_client)?;

    // Generate Python bindings if output directory is provided
    let python_code = generation::generate_python_code(&plan, metadata)?;
    output::write_generated_code(&python_code, output_dir_python)?;

    // Generate Python typing file
    let _python_typing = generation::generate_python_typing(&plan, metadata)?;
    let _typing_file_path = output_dir_python
        .parent()
        .and_then(|p| p.parent())
        .ok_or("Could not find Python client directory")?
        .join("unitycatalog_client_generated.pyi");
    // std::fs::write(&typing_file_path, python_typing)?;

    Ok(())
}

/// Generated code ready for output
#[derive(Debug)]
pub struct GeneratedCode {
    /// Generated files mapped by relative path
    pub files: HashMap<String, String>,
}

impl CodeGenMetadata {
    fn get_message_meta(&self, message_name: &str) -> Option<MessageMeta<'_>> {
        self.messages.get(message_name).map(|info| MessageMeta {
            info,
            metadata: self,
        })
    }
}

pub(crate) struct MessageMeta<'a> {
    info: &'a MessageInfo,
    metadata: &'a CodeGenMetadata,
}

pub(crate) struct ServiceHandler<'a> {
    plan: &'a ServicePlan,
    metadata: &'a CodeGenMetadata,
}

impl ServiceHandler<'_> {
    pub(crate) fn resource(&self) -> Option<&ManagedResource> {
        self.plan.managed_resources.first()
    }

    pub(crate) fn methods(&self) -> impl Iterator<Item = MethodHandler<'_>> {
        self.plan.methods.iter().map(|plan| MethodHandler {
            plan,
            metadata: self.metadata,
        })
    }

    pub(crate) fn client_type(&self) -> Ident {
        if let Some(resource) = self.resource() {
            format_ident!(
                "{}",
                format!("{} client", resource.descriptor.singular).to_case(Case::Pascal)
            )
        } else {
            format_ident!(
                "{}Client",
                self.plan
                    .service_name
                    .trim_end_matches("Service")
                    .trim_end_matches('s')
            )
        }
    }

    pub(crate) fn models_path(&self) -> syn::Path {
        syn::parse_str(&format!(
            "unitycatalog_common::models::{}::v1",
            self.plan.base_path
        ))
        .unwrap()
    }
}

pub(crate) struct MethodHandler<'a> {
    plan: &'a MethodPlan,
    metadata: &'a CodeGenMetadata,
}

impl MethodHandler<'_> {
    pub(crate) fn is_collection_method(&self) -> bool {
        matches!(
            self.plan.request_type,
            RequestType::List | RequestType::Create
        ) || (matches!(self.plan.request_type, RequestType::Custom(Pattern::Get(_)))
            && self.plan.metadata.method_name.starts_with("List"))
    }

    pub(crate) fn output_message(&self) -> Option<MessageMeta<'_>> {
        if self.plan.metadata.output_type.ends_with("Empty") {
            None
        } else {
            self.metadata
                .get_message_meta(&self.plan.metadata.output_type)
        }
    }

    pub(crate) fn output_type(&self) -> Option<Ident> {
        if self.plan.metadata.output_type.ends_with("Empty") {
            None
        } else {
            Some(extract_type_ident(&self.plan.metadata.output_type))
        }
    }

    pub(crate) fn input_message(&self) -> Option<MessageMeta<'_>> {
        if self.plan.metadata.input_type == "Empty" {
            None
        } else {
            self.metadata
                .get_message_meta(&self.plan.metadata.input_type)
        }
    }

    pub(crate) fn input_type(&self) -> Option<Ident> {
        if self.plan.metadata.input_type == "Empty" {
            None
        } else {
            Some(extract_type_ident(&self.plan.metadata.input_type))
        }
    }

    pub(crate) fn builder_type(&self) -> Ident {
        format_ident!("{}Builder", self.plan.metadata.method_name)
    }
}

/// Extract the final type name from a fully qualified protobuf type and convert to Ident
pub(crate) fn extract_type_ident(full_type: &str) -> Ident {
    let type_name = full_type.split('.').next_back().unwrap_or(full_type);
    format_ident!("{}", type_name)
}
