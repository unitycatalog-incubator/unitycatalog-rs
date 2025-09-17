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
use crate::parsing::{CodeGenMetadata, MessageField, MessageInfo};

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
        match field_type {
            "TYPE_STRING" => quote! { impl Into<String> },
            "TYPE_INT32" => quote! { i32 },
            "TYPE_INT64" => quote! { i64 },
            "TYPE_BOOL" => quote! { bool },
            "TYPE_DOUBLE" => quote! { f64 },
            "TYPE_FLOAT" => quote! { f32 },
            _ if field_type.starts_with("TYPE_ENUM:") => {
                let enum_type = self.convert_protobuf_enum_to_rust_type(field_type);
                let enum_ident: syn::Type =
                    syn::parse_str(&enum_type).unwrap_or_else(|_| syn::parse_str("i32").unwrap());
                quote! { #enum_ident }
            }
            _ if field_type.contains("map<") => {
                quote! { impl IntoIterator<Item = (impl Into<String>, impl Into<String>)> }
            }
            _ => quote! { impl Into<String> },
        }
    }

    /// Get Rust field type for a protobuf field type
    pub(crate) fn rust_field_type(&self, field_type: &str) -> String {
        match field_type {
            "TYPE_STRING" => "String".to_string(),
            "TYPE_INT32" => "i32".to_string(),
            "TYPE_INT64" => "i64".to_string(),
            "TYPE_BOOL" => "bool".to_string(),
            "TYPE_DOUBLE" => "f64".to_string(),
            "TYPE_FLOAT" => "f32".to_string(),
            "TYPE_BYTES" => "Vec<u8>".to_string(),
            _ if field_type.starts_with("TYPE_ENUM:") => {
                self.convert_protobuf_enum_to_rust_type(field_type)
            }
            _ if field_type.ends_with("PropertiesEntry") => "HashMap<String, String>".to_string(),
            _ if field_type.starts_with("TYPE_MESSAGE:") => {
                self.extract_simple_type_name(&field_type[13..])
            }
            _ if field_type.starts_with("TYPE_ONEOF:") => {
                self.extract_simple_type_name(&field_type[11..])
            }
            _ => "String".to_string(),
        }
    }

    /// Get Python parameter type for a Rust type
    pub(crate) fn python_parameter_type(&self, rust_type: &str, optional: bool) -> TokenStream {
        let base_type = if rust_type.starts_with("Option<") {
            // Extract inner type from Option<T>
            rust_type
                .strip_prefix("Option<")
                .and_then(|s| s.strip_suffix(">"))
                .unwrap_or(rust_type)
        } else {
            rust_type
        };

        let converted = self.convert_basic_type_to_python(base_type);

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
        match field_type {
            "TYPE_STRING" => quote! { #field_ident.into() },
            "TYPE_INT32" | "TYPE_INT64" | "TYPE_BOOL" | "TYPE_DOUBLE" | "TYPE_FLOAT" => {
                quote! { #field_ident }
            }
            _ if field_type.starts_with("TYPE_ENUM:") => quote! { #field_ident as i32 },
            _ if field_type.contains("map<") => quote! {
                #field_ident.into_iter().map(|(k, v)| (k.into(), v.into())).collect()
            },
            _ => quote! { #field_ident.into() },
        }
    }

    /// Get flexible field assignment for optional fields using impl Into<Option<T>>
    pub(crate) fn flexible_optional_field_assignment(
        &self,
        field_type: &str,
        field_ident: &proc_macro2::Ident,
    ) -> TokenStream {
        match field_type {
            "TYPE_STRING" => quote! { #field_ident.into() },
            "TYPE_INT32" | "TYPE_INT64" | "TYPE_BOOL" | "TYPE_DOUBLE" | "TYPE_FLOAT" => {
                quote! { #field_ident.into() }
            }
            _ if field_type.starts_with("TYPE_ENUM:") => {
                quote! { #field_ident.into().map(|e| e as i32) }
            }
            _ => quote! { #field_ident.into().map(|s| s.to_string()) },
        }
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

    /// Convert protobuf enum type to Rust enum type
    fn convert_protobuf_enum_to_rust_type(&self, field_type: &str) -> String {
        if let Some(enum_name) = field_type.strip_prefix("TYPE_ENUM:") {
            // Remove leading dot if present
            let enum_name = enum_name.trim_start_matches('.');

            // Parse the enum name parts
            let parts: Vec<&str> = enum_name.split('.').collect();

            match parts.as_slice() {
                // unitycatalog.tables.v1.TableType -> TableType
                ["unitycatalog", "tables", "v1", enum_type] => enum_type.to_string(),
                // unitycatalog.credentials.v1.Purpose -> Purpose
                ["unitycatalog", "credentials", "v1", enum_type] => enum_type.to_string(),
                // unitycatalog.recipients.v1.AuthenticationType -> AuthenticationType
                ["unitycatalog", "recipients", "v1", enum_type] => enum_type.to_string(),
                // unitycatalog.volumes.v1.VolumeType -> VolumeType
                ["unitycatalog", "volumes", "v1", enum_type] => enum_type.to_string(),
                // unitycatalog.temporary_credentials.v1.generate_temporary_table_credentials_request.Operation
                [
                    "unitycatalog",
                    "temporary_credentials",
                    "v1",
                    nested_type,
                    enum_type,
                ] => {
                    // Convert to snake_case module name
                    let snake_case_module =
                        nested_type.chars().fold(String::new(), |mut acc, c| {
                            if c.is_uppercase() && !acc.is_empty() {
                                acc.push('_');
                            }
                            acc.push(c.to_lowercase().next().unwrap());
                            acc
                        });
                    format!("{}::{}", snake_case_module, enum_type)
                }
                // Fallback: use the last part as the enum name
                _ => parts.last().map_or("i32", |v| v).to_string(),
            }
        } else {
            // Not an enum type, return as-is (fallback to i32)
            "i32".to_string()
        }
    }

    /// Extract simple type name from fully qualified protobuf type
    fn extract_simple_type_name(&self, full_type: &str) -> String {
        // Remove leading dots and extract the last component
        let trimmed = full_type.trim_start_matches('.');
        trimmed
            .split('.')
            .next_back()
            .unwrap_or(trimmed)
            .to_string()
    }

    /// Convert basic Rust types to Python-compatible types
    fn convert_basic_type_to_python(&self, rust_type: &str) -> TokenStream {
        match rust_type {
            "String" | "str" => quote! { String },
            "i32" => quote! { i32 },
            "i64" => quote! { i64 },
            "bool" => quote! { bool },
            "f32" => quote! { f32 },
            "f64" => quote! { f64 },
            s if s.contains("HashMap") => quote! { HashMap<String, String> },
            _ => {
                // Assume it's a struct type, use as-is
                let type_ident = format_ident!("{}", rust_type);
                quote! { #type_ident }
            }
        }
    }
}

/// Extract the final type name from a fully qualified protobuf type and convert to Ident
pub(crate) fn extract_type_ident(full_type: &str) -> Ident {
    let type_name = full_type.split('.').next_back().unwrap_or(full_type);
    format_ident!("{}", type_name)
}
