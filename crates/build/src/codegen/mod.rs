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
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{File, Ident};

use crate::error::{Error, Result};

use crate::analysis::{
    BodyField, GenerationPlan, ManagedResource, MethodPlan, RequestParam, RequestType, ServicePlan,
    analyze_metadata, split_body_fields,
};
use crate::google::api::http_rule::Pattern;
use crate::output;
use crate::parsing::types::{self, UnifiedType};
use crate::parsing::{CodeGenMetadata, MessageField, MessageInfo, RenderContext};

mod builder;
mod client;
mod handler;
pub(crate) mod node;
mod python;
mod resources;
mod server;

impl MethodPlan {
    pub(crate) fn resource_client_method(&self) -> Ident {
        match self.request_type {
            RequestType::Get => format_ident!("get"),
            RequestType::Update => format_ident!("update"),
            RequestType::Delete => format_ident!("delete"),
            _ => format_ident!("{}", self.handler_function_name),
        }
    }

    pub(crate) fn base_method_ident(&self) -> Ident {
        format_ident!("{}", self.handler_function_name)
    }
}

/// Validated model import path derived from a `{service}` template string.
///
/// Constructed once from [`CodeGenConfig`] template fields. `resolve` performs the
/// `{service}` substitution and parses the result as a [`syn::Path`], catching
/// malformed templates at construction time rather than at code-generation time.
#[derive(Debug, Clone)]
pub struct ModelsPath {
    template: String,
}

impl ModelsPath {
    /// Build a `ModelsPath` from a template string containing `{service}`.
    ///
    /// Performs a test substitution at construction to validate the template.
    pub fn new(template: &str) -> Result<Self> {
        let test = template.replace("{service}", "test");
        syn::parse_str::<syn::Path>(&test).map_err(|e| Error::InvalidModelsPathTemplate {
            template: template.to_string(),
            source: e,
        })?;
        Ok(Self {
            template: template.to_string(),
        })
    }

    /// Replace `{service}` with `service` and return the parsed [`syn::Path`].
    ///
    /// # Panics
    ///
    /// Cannot panic in practice: `new` already validates that every possible
    /// `{service}` substitution produces a valid path.
    pub fn resolve(&self, service: &str) -> syn::Path {
        let path = self.template.replace("{service}", service);
        syn::parse_str(&path)
            .unwrap_or_else(|e| panic!("Invalid models path `{path}` after substitution: {e}"))
    }
}

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
///
/// Only `common` is required. All other outputs are optional — set to `None` to skip that
/// output entirely. For example, a server-only crate can omit `client`, and a client-only
/// crate can omit `server`.
#[derive(Debug, Clone)]
pub struct CodeGenOutput {
    /// Output directory for common (shared extractor) code.
    pub common: PathBuf,
    /// Output directory for generated model files (e.g. `unitycatalog.internal.rs`).
    /// When `None`, these files are written alongside `common` output.
    pub models_gen: Option<PathBuf>,
    /// Output directory for server-side handler and route code. Generation is skipped when `None`.
    pub server: Option<PathBuf>,
    /// Output directory for HTTP client code. Generation is skipped when `None`.
    pub client: Option<PathBuf>,
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
pub fn generate_code(metadata: &CodeGenMetadata, config: &CodeGenConfig) -> Result<()> {
    // Validate templates early so callers get a clean error before any generation starts.
    ModelsPath::new(&config.models_path_template)?;
    ModelsPath::new(&config.models_path_crate_template)?;

    let plan = analyze_metadata(metadata)?;

    let (common_code, models_code) = generate_common_code(&plan, metadata, config)?;
    output::write_generated_code(&common_code, &config.output.common)?;
    let models_dir = config
        .output
        .models_gen
        .as_ref()
        .unwrap_or(&config.output.common);
    output::write_generated_code(&models_code, models_dir)?;

    if let Some(ref server_dir) = config.output.server {
        let server_code = generate_server_code(&plan, metadata, config)?;
        output::write_generated_code(&server_code, server_dir)?;
    }

    if let Some(ref client_dir) = config.output.client {
        let client_code = generate_client_code(&plan, metadata, config)?;
        output::write_generated_code(&client_code, client_dir)?;
    }

    if let Some(ref python_dir) = config.output.python {
        let python_code = generate_python_code(&plan, metadata, config)?;
        output::write_generated_code(&python_code, python_dir)?;
    }

    if let Some(ref node_dir) = config.output.node {
        let node_code = generate_node_code(&plan, metadata, config)?;
        output::write_generated_code(&node_code, node_dir)?;
    }

    if let Some(ref node_ts_dir) = config.output.node_ts {
        let node_ts_code = generate_node_ts_code(&plan, metadata, config)?;
        output::write_generated_code(&node_ts_code, node_ts_dir)?;
    }

    Ok(())
}

fn generate_common_code(
    plan: &GenerationPlan,
    metadata: &CodeGenMetadata,
    config: &CodeGenConfig,
) -> Result<(GeneratedCode, GeneratedCode)> {
    let mut files = HashMap::new();

    for service in &plan.services {
        let handler = ServiceHandler {
            plan: service,
            metadata,
            config,
        };
        let server_code = server::generate_common(&handler);
        files.insert(format!("{}/server.rs", service.base_path), server_code);
        let module_code = generate_common_module();
        files.insert(format!("{}/mod.rs", service.base_path), module_code);
    }

    let module_code = main_module(&plan.services);
    files.insert("mod.rs".to_string(), module_code);

    let resource_enum = resources::generate_resource_enum(metadata);
    let mut models_files = HashMap::new();
    models_files.insert("labels.rs".to_string(), resource_enum);

    Ok((
        GeneratedCode { files },
        GeneratedCode {
            files: models_files,
        },
    ))
}

fn generate_server_code(
    plan: &GenerationPlan,
    metadata: &CodeGenMetadata,
    config: &CodeGenConfig,
) -> Result<GeneratedCode> {
    let mut files = HashMap::new();

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
        let module_code = generate_server_module(service);
        files.insert(format!("{}/mod.rs", service.base_path), module_code);
    }

    let module_code = main_module(&plan.services);
    files.insert("mod.rs".to_string(), module_code);

    Ok(GeneratedCode { files })
}

fn generate_python_code(
    plan: &GenerationPlan,
    metadata: &CodeGenMetadata,
    config: &CodeGenConfig,
) -> Result<GeneratedCode> {
    let mut files = HashMap::new();

    let handlers = plan
        .services
        .iter()
        .map(|service| ServiceHandler {
            plan: service,
            metadata,
            config,
        })
        .collect_vec();

    for service in &handlers {
        let python_code = python::generate(service);
        files.insert(format!("{}.rs", service.plan.base_path), python_code);
    }

    let module_code = python::main_module(&handlers);
    files.insert("mod.rs".to_string(), module_code);

    let python_typings_code = python::generate_typings(&handlers);
    files.insert(
        config.output.python_typings_filename.clone(),
        python_typings_code,
    );

    Ok(GeneratedCode { files })
}

fn generate_node_code(
    plan: &GenerationPlan,
    metadata: &CodeGenMetadata,
    config: &CodeGenConfig,
) -> Result<GeneratedCode> {
    let mut files = HashMap::new();

    let handlers = plan
        .services
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

fn generate_node_ts_code(
    plan: &GenerationPlan,
    metadata: &CodeGenMetadata,
    config: &CodeGenConfig,
) -> Result<GeneratedCode> {
    let handlers = plan
        .services
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

fn generate_client_code(
    plan: &GenerationPlan,
    metadata: &CodeGenMetadata,
    config: &CodeGenConfig,
) -> Result<GeneratedCode> {
    let mut files = HashMap::new();

    for service in &plan.services {
        let handler = ServiceHandler {
            plan: service,
            metadata,
            config,
        };
        let client_code = client::generate(&handler)?;
        files.insert(format!("{}/client.rs", service.base_path), client_code);
        let builder_code = builder::generate(&handler)?;
        files.insert(format!("{}/builders.rs", service.base_path), builder_code);
        let module_code = generate_client_module();
        files.insert(format!("{}/mod.rs", service.base_path), module_code);
    }

    let module_code = generate_client_main_module(&plan.services);
    files.insert("mod.rs".to_string(), module_code);

    Ok(GeneratedCode { files })
}

fn generate_common_module() -> String {
    let tokens = quote! {
        #[cfg(feature = "axum")]
        pub mod server;
    };
    format_tokens(tokens)
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

pub fn main_module(services: &[ServicePlan]) -> String {
    let service_modules: Vec<TokenStream> = services
        .iter()
        .map(|s| {
            let module_name = format_ident!("{}", s.base_path);
            quote! { pub mod #module_name; }
        })
        .collect();

    let tokens = quote! {
        #(#service_modules)*
    };
    format_tokens(tokens)
}

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
pub(crate) fn doc_tokens(documentation: Option<&str>) -> TokenStream {
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

pub(crate) fn format_tokens(tokens: TokenStream) -> String {
    let tokens_string = tokens.to_string();
    let syntax_tree = syn::parse2::<File>(tokens).unwrap_or_else(|_| {
        syn::parse_str::<File>(&tokens_string).unwrap_or_else(|_| {
            syn::parse_quote! {
                // Failed to parse generated code
            }
        })
    });
    prettyplease::unparse(&syntax_tree)
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
        // Templates are validated by `generate_code` before any `ServiceHandler` is used,
        // so this substitution is guaranteed to succeed.
        ModelsPath::new(&self.config.models_path_template)
            .expect("models_path_template already validated by generate_code")
            .resolve(&self.plan.base_path)
    }

    pub(crate) fn models_path_crate(&self) -> syn::Path {
        ModelsPath::new(&self.config.models_path_crate_template)
            .expect("models_path_crate_template already validated by generate_code")
            .resolve(&self.plan.base_path)
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

    /// Split body fields into required and optional subsets.
    pub(crate) fn split_body_fields(&self) -> (Vec<&BodyField>, Vec<&BodyField>) {
        split_body_fields(self.plan)
    }
}

/// Extract the final type name from a fully qualified protobuf type and convert to Ident
pub(crate) fn extract_type_ident(full_type: &str) -> Ident {
    let type_name = full_type.split('.').next_back().unwrap_or(full_type);
    format_ident!("{}", type_name)
}
