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
use std::path::PathBuf;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::format_ident;
use syn::Ident;

use crate::analysis::{
    ManagedResource, MethodPlan, RequestParam, RequestType, ServicePlan, analyze_metadata,
};
use crate::google::api::http_rule::Pattern;
use crate::output;
use crate::parsing::types::{self, UnifiedType};
use crate::parsing::{CodeGenMetadata, MessageField, MessageInfo, RenderContext};

pub mod generation;

/// Configuration for code generation, including import paths and output directories.
///
/// Use [`CodeGenConfig::unitycatalog_defaults`] when generating code for the Unity Catalog
/// workspace. To generate code for an external crate with different runtime types, construct
/// this struct directly and override the fields you need.
#[derive(Debug, Clone)]
pub struct CodeGenConfig {
    /// Fully-qualified path to the request context type used in handler methods.
    ///
    /// Default: `"crate::api::RequestContext"`
    pub context_type_path: String,

    /// Fully-qualified path to the `Result` alias used in generated handler and client code.
    ///
    /// Default: `"crate::Result"`
    pub result_type_path: String,

    /// Template for the external model import path. `{service}` is replaced with the service's
    /// base path (e.g. `"catalogs"`).
    ///
    /// Default: `"unitycatalog_common::models::{service}::v1"`
    pub models_path_template: String,

    /// Template for crate-local model import path. `{service}` is replaced with the service's
    /// base path.
    ///
    /// Default: `"crate::models::{service}::v1"`
    pub models_path_crate_template: String,

    /// Output directory configuration.
    pub output: CodeGenOutput,
}

impl CodeGenConfig {
    /// Create a config with the default Unity Catalog import paths.
    pub fn unitycatalog_defaults(output: CodeGenOutput) -> Self {
        Self {
            context_type_path: "crate::api::RequestContext".to_string(),
            result_type_path: "crate::Result".to_string(),
            models_path_template: "unitycatalog_common::models::{service}::v1".to_string(),
            models_path_crate_template: "crate::models::{service}::v1".to_string(),
            output,
        }
    }
}

/// Output directory configuration for code generation.
#[derive(Debug, Clone)]
pub struct CodeGenOutput {
    /// Output directory for common (shared extractor) code.
    pub common: PathBuf,
    /// Output directory for server-side handler and route code.
    pub server: PathBuf,
    /// Output directory for HTTP client code.
    pub client: PathBuf,
    /// Output directory for Python bindings. Generation is skipped when `None`.
    pub python: Option<PathBuf>,
    /// Output directory for Node.js NAPI bindings. Generation is skipped when `None`.
    pub node: Option<PathBuf>,
    /// Output directory for Node.js TypeScript client. Generation is skipped when `None`.
    pub node_ts: Option<PathBuf>,
    /// Filename for the generated Python typings stub.
    ///
    /// Default: `"unitycatalog_client.pyi"`
    pub python_typings_filename: String,
}

/// Main entry point for code generation
///
/// Takes collected metadata and a [`CodeGenConfig`] and generates all necessary code.
pub fn generate_code(
    metadata: &CodeGenMetadata,
    config: &CodeGenConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Analyze metadata and plan generation
    let plan = analyze_metadata(metadata)?;

    // Generate code from plan
    let common_code = generation::generate_common_code(&plan, metadata, config)?;
    output::write_generated_code(&common_code, &config.output.common)?;

    // Generate server
    let server_code = generation::generate_server_code(&plan, metadata, config)?;
    output::write_generated_code(&server_code, &config.output.server)?;

    // Generate client
    let client_code = generation::generate_client_code(&plan, metadata, config)?;
    output::write_generated_code(&client_code, &config.output.client)?;

    // Generate Python bindings
    if let Some(ref python_dir) = config.output.python {
        let python_code = generation::generate_python_code(&plan, metadata, config)?;
        output::write_generated_code(&python_code, python_dir)?;
    }

    // Generate Node.js NAPI bindings
    if let Some(ref node_dir) = config.output.node {
        let node_code = generation::generate_node_code(&plan, metadata, config)?;
        output::write_generated_code(&node_code, node_dir)?;
    }

    // Generate Node.js TypeScript client
    if let Some(ref node_ts_dir) = config.output.node_ts {
        let node_ts_code = generation::generate_node_ts_code(&plan, metadata, config)?;
        output::write_generated_code(&node_ts_code, node_ts_dir)?;
    }

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
    pub(crate) plan: &'a ServicePlan,
    pub(crate) metadata: &'a CodeGenMetadata,
    pub(crate) config: &'a CodeGenConfig,
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
        let path = self
            .config
            .models_path_template
            .replace("{service}", &self.plan.base_path);
        syn::parse_str(&path)
            .unwrap_or_else(|e| panic!("Invalid models_path_template `{path}`: {e}"))
    }

    pub(crate) fn models_path_crate(&self) -> syn::Path {
        let path = self
            .config
            .models_path_crate_template
            .replace("{service}", &self.plan.base_path);
        syn::parse_str(&path)
            .unwrap_or_else(|e| panic!("Invalid models_path_crate_template `{path}`: {e}"))
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
            || (matches!(
                self.plan.request_type,
                RequestType::Custom(Pattern::Post(_))
            ) && self.plan.metadata.method_name.starts_with("Generate"))
    }

    pub(crate) fn output_message(&self) -> Option<MessageMeta<'_>> {
        if self.plan.metadata.output_type.ends_with("Empty") {
            return None;
        }
        self.metadata
            .get_message_meta(&self.plan.metadata.output_type)
    }

    pub(crate) fn output_type(&self) -> Option<Ident> {
        self.output_message()
            .map(|t| extract_type_ident(&t.info.name))
    }

    pub(crate) fn list_output_field(&self) -> Option<&MessageField> {
        self.output_message()?
            .info
            .fields
            .iter()
            .find(|f| !f.name.contains("page_token"))
    }

    pub(crate) fn input_message(&self) -> Option<MessageMeta<'_>> {
        if self.plan.metadata.input_type == "Empty" {
            return None;
        }
        self.metadata
            .get_message_meta(&self.plan.metadata.input_type)
    }

    pub(crate) fn input_type(&self) -> Option<Ident> {
        self.input_message()
            .map(|t| extract_type_ident(&t.info.name))
    }

    pub(crate) fn builder_type(&self) -> Ident {
        format_ident!("{}Builder", self.plan.metadata.method_name)
    }

    /// Get type representation for rust depending on context
    ///
    /// Depending on context we may want concrete types (e.g. 'String') or more flexible types (e.g. 'Into<String d>')
    pub(crate) fn field_type(&self, field_type: &UnifiedType, ctx: RenderContext) -> syn::Type {
        let rust_type = types::unified_to_rust(field_type, ctx);
        syn::parse_str(&rust_type).expect("proper field type")
    }

    /// Get field assignment TokenStream for constructor
    pub(crate) fn field_assignment(
        &self,
        field_type: &UnifiedType,
        field_ident: &proc_macro2::Ident,
        ctx: &RenderContext,
    ) -> TokenStream {
        types::field_assignment(field_type, field_ident, ctx)
    }

    pub(crate) fn required_parameters(&self) -> impl Iterator<Item = &RequestParam> {
        self.plan
            .parameters
            .iter()
            .filter(|param| !param.is_optional())
    }

    pub(crate) fn optional_parameters(&self) -> impl Iterator<Item = &RequestParam> {
        self.plan
            .parameters
            .iter()
            .filter(|param| param.is_optional())
    }

    /// Analyze request fields to separate required from optional
    pub(crate) fn analyze_request_fields(&self) -> (Vec<&MessageField>, Vec<&MessageField>) {
        let fields = &self.plan.metadata.input_fields;
        let mut required = Vec::new();
        let mut optional = Vec::new();

        for field in fields {
            use crate::parsing::types::BaseType;
            if field.optional
                || field.repeated
                || matches!(
                    field.unified_type.base_type,
                    BaseType::Map(_, _) | BaseType::Message(_) | BaseType::OneOf(_)
                )
            {
                optional.push(field);
            } else {
                required.push(field);
            }
        }

        (required, optional)
    }
}

/// Extract the final type name from a fully qualified protobuf type and convert to Ident
pub(crate) fn extract_type_ident(full_type: &str) -> Ident {
    let type_name = full_type.split('.').next_back().unwrap_or(full_type);
    format_ident!("{}", type_name)
}
