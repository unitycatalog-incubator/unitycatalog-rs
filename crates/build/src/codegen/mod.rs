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
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::analysis::{ManagedResource, MethodPlan, RequestType, ServicePlan, analyze_metadata};
use crate::google::api::http_rule::Pattern;
use crate::output;
use crate::parsing::types::UnifiedType;
use crate::parsing::{CodeGenMetadata, MessageField, MessageInfo, RenderContext, TypeConverter};

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

    pub(crate) fn models_path_crate(&self) -> syn::Path {
        syn::parse_str(&format!("crate::models::{}::v1", self.plan.base_path)).unwrap()
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

    /// Get Rust parameter type for constructor-like arguments (builder methods)
    pub(crate) fn rust_parameter_type(&self, field_type: &str) -> TokenStream {
        let type_converter = TypeConverter::new();
        let unified_type = type_converter.protobuf_to_unified(field_type);
        let param_type_str = type_converter.rust_parameter_type(&unified_type);
        param_type_str.parse().unwrap_or_else(|_| quote! { String })
    }

    pub(crate) fn rust_field_type_unified(
        &self,
        field_type: &UnifiedType,
        ctx: RenderContext,
    ) -> syn::Type {
        let type_converter = TypeConverter::new();
        let rust_type = type_converter.unified_to_rust(field_type, ctx);
        syn::parse_str(&rust_type).expect("proper field type")
    }

    /// Get Python parameter type for a Rust type
    pub(crate) fn python_parameter_type(&self, rust_type: &str, optional: bool) -> TokenStream {
        // This method needs to convert Rust type strings back to Python types
        // Since this is used by existing Python generation code, we keep it simple
        let base_type = if rust_type.starts_with("Option<") {
            rust_type
                .strip_prefix("Option<")
                .and_then(|s| s.strip_suffix(">"))
                .unwrap_or(rust_type)
        } else {
            rust_type
        };

        let converted = match base_type {
            "String" | "str" => quote! { String },
            "i32" => quote! { i32 },
            "i64" => quote! { i64 },
            "bool" => quote! { bool },
            "f32" => quote! { f32 },
            "f64" => quote! { f64 },
            s if s.contains("HashMap") => quote! { HashMap<String, String> },
            _ => {
                // Assume it's a struct type, use as-is
                let type_ident = format_ident!("{}", base_type);
                quote! { #type_ident }
            }
        };

        if optional || rust_type.starts_with("Option<") {
            quote! { Option<#converted> }
        } else {
            converted
        }
    }

    /// Get field assignment TokenStream for constructor
    pub(crate) fn field_assignment(
        &self,
        field_type: &str,
        field_ident: &proc_macro2::Ident,
    ) -> TokenStream {
        let type_converter = TypeConverter::new();
        let unified_type = type_converter.protobuf_to_unified(field_type);
        type_converter.field_assignment(&unified_type, field_ident)
    }

    /// Get flexible field assignment for optional fields using impl Into<Option<T>>
    pub(crate) fn flexible_optional_field_assignment(
        &self,
        field_type: &str,
        field_ident: &proc_macro2::Ident,
    ) -> TokenStream {
        let type_converter = TypeConverter::new();
        let unified_type = type_converter.protobuf_to_unified(field_type);
        type_converter.flexible_optional_field_assignment(&unified_type, field_ident)
    }

    /// Analyze request fields to separate required from optional
    pub(crate) fn analyze_request_fields(&self) -> (Vec<&MessageField>, Vec<&MessageField>) {
        let fields = &self.plan.metadata.input_fields;
        let mut required = Vec::new();
        let mut optional = Vec::new();

        for field in fields {
            if field.optional {
                optional.push(field);
            } else if field.field_type.contains("map<") {
                // Maps are not required in constructor, but are optional with_* methods
                optional.push(field);
            } else if field.field_type.starts_with("TYPE_MESSAGE:")
                || field.field_type.starts_with("TYPE_ONEOF:")
                || field.repeated
            {
                // Complex message types, oneof fields, and repeated fields go to optional with direct setters
                optional.push(field);
            } else {
                required.push(field);
            }
        }

        (required, optional)
    }

    /// Get required fields for constructor
    pub(crate) fn required_constructor_fields(&self) -> Vec<&MessageField> {
        let (required, _) = self.analyze_request_fields();
        required
    }

    /// Get optional fields for builder methods
    pub(crate) fn optional_builder_fields(&self) -> Vec<&MessageField> {
        let (_, optional) = self.analyze_request_fields();
        optional
    }
}

/// Extract the final type name from a fully qualified protobuf type and convert to Ident
pub(crate) fn extract_type_ident(full_type: &str) -> Ident {
    let type_name = full_type.split('.').next_back().unwrap_or(full_type);
    format_ident!("{}", type_name)
}
