//! Python bindings generation module
//!
//! This module generates Python bindings for Unity Catalog services using PyO3.
//! It creates wrapper structs that expose the Rust client functionality to Python
//! while handling async operations and error conversion.

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use textwrap::{Options, dedent, fill, indent, refill};

use super::format_tokens;
use crate::analysis::{RequestParam, RequestType};
use crate::codegen::{MethodHandler, ServiceHandler};
use crate::parsing::types::{BaseType, UnifiedType};
use crate::parsing::{CodeGenMetadata, EnumInfo, MessageField, MessageInfo, RenderContext};
use crate::utils::strings;

static DOCS_TARGET_WIDTH: usize = 100;

pub fn main_module(services: &[ServiceHandler<'_>]) -> String {
    let service_modules = services.iter().map(|s| {
        let module_name = format_ident!("{}", s.plan.base_path);
        quote! { pub mod #module_name; }
    });
    let uc_client_module = collection_client_struct(services);

    let tokens = quote! {
        // Service modules
        #(#service_modules)*

        #uc_client_module
    };

    format_tokens(tokens)
}

/// Generate Python bindings for a service
pub(crate) fn generate(service: &ServiceHandler<'_>) -> String {
    let rust_client_ident = service.client_type();
    let client_ident = format_ident!("{}", format!("Py{}", rust_client_ident));
    let rust_client_name = rust_client_ident.to_string();

    let methods = service.methods().filter_map(resource_client_method);
    let mod_path = service.models_path();

    let tokens = quote! {
        use std::collections::HashMap;
        use pyo3::prelude::*;
        use unitycatalog_client::#rust_client_ident;
        use #mod_path::*;
        use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
        use crate::runtime::get_runtime;

        #[pyclass(name = #rust_client_name)]
        pub struct #client_ident {
            pub(crate) client: #rust_client_ident,
        }

        #[pymethods]
        impl #client_ident {
            #(#methods)*
        }

        impl #client_ident {
            pub fn new(client: #rust_client_ident) -> Self {
                Self { client }
            }
        }
    };

    format_tokens(tokens)
}

fn collection_client_struct(services: &[ServiceHandler<'_>]) -> TokenStream {
    let mut services = services.iter().collect_vec();
    services.sort_by(|a, b| a.plan.service_name.cmp(&b.plan.service_name));

    let mod_paths = services.iter().map(|s| {
        let mod_path = s.models_path();
        quote! { use #mod_path::*; }
    });

    let codegen_imports = services.iter().map(|s| {
        let mod_name = format_ident!("{}", s.plan.base_path);
        let client_name = format_ident!("Py{}", s.client_type().to_string());
        quote! { use crate::codegen::#mod_name::#client_name; }
    });

    let methods = services
        .iter()
        .flat_map(|s| s.methods().filter_map(collection_client_method));

    let resource_accessor_methods = services
        .iter()
        .filter_map(|s| generate_resource_accessor_method(s));

    quote! {
        use std::collections::HashMap;
        use futures::stream::TryStreamExt;
        use pyo3::prelude::*;
        use unitycatalog_client::{UnityCatalogClient};
        use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
        use crate::runtime::get_runtime;
        #(#mod_paths)*
        #(#codegen_imports)*

        #[pyclass(name = "UnityCatalogClient")]
        pub struct PyUnityCatalogClient {
            client: UnityCatalogClient
        }

        #[pymethods]
        impl PyUnityCatalogClient {
            #[new]
            #[pyo3(signature = (base_url, token = None))]
            pub fn new(base_url: String, token: Option<String>) -> PyResult<Self> {
                let client = if let Some(token) = token {
                    cloud_client::CloudClient::new_with_token(token)
                } else {
                    cloud_client::CloudClient::new_unauthenticated()
                };
                let base_url = base_url.parse().map_err(PyUnityCatalogError::from)?;
                Ok(Self { client: UnityCatalogClient::new(client, base_url) })
            }

            #(#methods)*

            #(#resource_accessor_methods)*
        }
    }
}

/// Generate Python method wrapper
fn resource_client_method(method: MethodHandler<'_>) -> Option<TokenStream> {
    let code = match &method.plan.request_type {
        RequestType::Get | RequestType::Update => resource_get_update_method_impl(&method),
        RequestType::Delete => resource_delete_method_impl(&method),
        _ => return None,
    };
    Some(code)
}

fn collection_client_method(method: MethodHandler<'_>) -> Option<TokenStream> {
    if !method.is_collection_method() {
        return None;
    }
    match &method.plan.request_type {
        RequestType::List => Some(collection_list_method_impl(&method)),
        RequestType::Create => Some(collection_create_method_impl(&method)),
        _ => None,
    }
}

/// Generate List method (returns Vec<T> and uses try_collect)
fn collection_list_method_impl(method: &MethodHandler<'_>) -> TokenStream {
    let method_name = method.plan.base_method_ident();

    let (param_defs, pyo3_signature) = collection_method_parameters(method, true);
    let client_call = inner_resource_client_call(method);
    let builder_calls = generate_builder_pattern(method, true);

    let items_field = method.list_output_field().unwrap();
    let response_type = method.field_type(&items_field.unified_type, RenderContext::ReturnType);

    quote! {
        #pyo3_signature
        pub fn #method_name(
            &self,
            py: Python,
            #(#param_defs,)*
        ) -> PyUnityCatalogResult<#response_type> {
            let mut request = #client_call;
            #(#builder_calls)*
            let runtime = get_runtime(py)?;
            py.allow_threads(|| {
                let result = runtime.block_on(async move { request.into_stream().try_collect().await })?;
                Ok::<_, PyUnityCatalogError>(result)
            })
        }
    }
}

/// Generate Create method (uses builder pattern)
fn collection_create_method_impl(method: &MethodHandler<'_>) -> TokenStream {
    let method_name = method.plan.base_method_ident();
    let response_type = method.output_type().unwrap();
    let (param_defs, pyo3_signature) = collection_method_parameters(method, false);
    let client_call = inner_resource_client_call(method);
    let builder_calls = generate_builder_pattern(method, false);

    quote! {
        #pyo3_signature
        pub fn #method_name(
            &self,
            py: Python,
            #(#param_defs,)*
        ) -> PyUnityCatalogResult<#response_type> {
            let mut request = #client_call;
            #(#builder_calls)*
            let runtime = get_runtime(py)?;
            py.allow_threads(|| {
                let result = runtime.block_on(request.into_future())?;
                Ok::<_, PyUnityCatalogError>(result)
            })
        }
    }
}

/// Generate a method call to a resource client.
fn resource_get_update_method_impl(method: &MethodHandler<'_>) -> TokenStream {
    let method_name = method.plan.resource_client_method();
    let response_type = method.output_type();
    let (param_defs, pyo3_signature) = resource_method_parameters(method);
    let client_call = inner_resource_client_call(method);
    let builder_calls = generate_builder_pattern(method, false);

    quote! {
        #pyo3_signature
        pub fn #method_name(
            &self,
            py: Python,
            #(#param_defs,)*
        ) -> PyUnityCatalogResult<#response_type> {
            let mut request = #client_call;
            #(#builder_calls)*
            let runtime = get_runtime(py)?;
            py.allow_threads(|| {
                let result = runtime.block_on(request.into_future())?;
                Ok::<_, PyUnityCatalogError>(result)
            })
        }
    }
}

/// Generate Delete method (returns unit type)
fn resource_delete_method_impl(method: &MethodHandler<'_>) -> TokenStream {
    let method_name = method.plan.resource_client_method();
    let (param_defs, pyo3_signature) = resource_method_parameters(method);
    let client_call = inner_resource_client_call(method);
    let builder_calls = generate_builder_pattern(method, false);

    quote! {
        #pyo3_signature
        pub fn #method_name(
            &self,
            py: Python,
            #(#param_defs,)*
        ) -> PyUnityCatalogResult<()> {
            let mut request = #client_call;
            #(#builder_calls)*
            let runtime = get_runtime(py)?;
            py.allow_threads(|| {
                runtime.block_on(request.into_future())?;
                Ok::<_, PyUnityCatalogError>(())
            })
        }
    }
}

/// Generate parameter definitions for method signature
fn resource_method_parameters(method: &MethodHandler<'_>) -> (Vec<TokenStream>, TokenStream) {
    let map_param = |p: &RequestParam| {
        let param_name = p.field_ident();
        let rust_type = method.field_type(p.field_type(), RenderContext::PythonParameter);
        quote! { #param_name: #rust_type }
    };
    let parameters = method
        .required_parameters()
        .chain(method.optional_parameters())
        // we omit path parameters since these are encoded in the resource client struct.
        .filter(|field| !field.is_path_param())
        .collect_vec();
    let signature = render_pyo3(&parameters);
    (parameters.into_iter().map(map_param).collect(), signature)
}

/// Generate parameter definitions for method signature
fn collection_method_parameters(
    method: &MethodHandler<'_>,
    is_list: bool,
) -> (Vec<TokenStream>, TokenStream) {
    let map_param = |p: &RequestParam| {
        let param_name = p.field_ident();
        let rust_type = method.field_type(p.field_type(), RenderContext::PythonParameter);
        quote! { #param_name: #rust_type }
    };
    let parameters = method
        .required_parameters()
        .chain(method.optional_parameters())
        .filter(|p| !(is_list && p.name() == "page_token"))
        .collect_vec();
    let signature = render_pyo3(&parameters);
    (parameters.into_iter().map(map_param).collect(), signature)
}

fn render_pyo3(signature_parts: &[&RequestParam]) -> TokenStream {
    let signature_parts = signature_parts
        .iter()
        .map(|p| {
            if p.is_optional() {
                format!("{} = None", p.name())
            } else {
                p.name().to_string()
            }
        })
        .collect_vec();
    if signature_parts.is_empty() {
        quote! {}
    } else {
        let signature_string = signature_parts.join(", ");
        let tokens = signature_string
            .parse::<proc_macro2::TokenStream>()
            .unwrap();
        quote! {
            #[pyo3(signature = (#tokens))]
        }
    }
}

/// Generate resource client call
///
/// generated the invocation for a client call on a resource client.
/// Specifically this assumes that all path parameters which identify
/// the resource are encoded in the client and don't need to be passed
/// as arguments.
fn inner_resource_client_call(method: &MethodHandler<'_>) -> TokenStream {
    let method_name = method.plan.resource_client_method();
    let args = method
        .required_parameters()
        .filter(|param| !param.is_path_param())
        .map(|param| {
            let param_name = param.field_ident();
            quote! { #param_name }
        });
    quote! {
        self.client.#method_name(#(#args,)*)
    }
}

/// Generate builder pattern calls for Create/Update methods
fn generate_builder_pattern(method: &MethodHandler<'_>, is_list: bool) -> Vec<TokenStream> {
    let mut builder_calls = Vec::new();

    for query_param in method.plan.query_parameters() {
        if query_param.is_optional() && !(is_list && query_param.name == "page_token") {
            let param_name =
                format_ident!("{}", strings::operation_to_method_name(&query_param.name));
            let with_method = format_ident!("with_{}", query_param.name);
            builder_calls.push(quote! {
                request = request.#with_method(#param_name);
            });
        }
    }

    for body_field in method.plan.body_fields() {
        if body_field.optional {
            let param_name =
                format_ident!("{}", strings::operation_to_method_name(&body_field.name));
            let with_method = format_ident!("with_{}", body_field.name);

            // Special handling for HashMap/properties fields
            if matches!(body_field.field_type.base_type, BaseType::Map(_, _))
                || body_field.field_type.is_repeated
            {
                builder_calls.push(quote! {
                    if let Some(#param_name) = #param_name {
                        request = request.#with_method(#param_name);
                    }
                });
            } else {
                builder_calls.push(quote! {
                    request = request.#with_method(#param_name);
                });
            }
        }
    }

    builder_calls
}

/// Generate resource accessor method for the main client
fn generate_resource_accessor_method(service: &ServiceHandler<'_>) -> Option<TokenStream> {
    // Only generate methods for services that manage resources
    if service.plan.managed_resources.is_empty() {
        return None;
    }

    // For now, assume each service manages exactly one resource type
    let resource = &service.resource().unwrap();
    let method_name = format_ident!("{}", resource.descriptor.singular);
    let client_name = format_ident!("Py{}", service.client_type().to_string());

    // Generate method based on the specific resource patterns
    // Use the singular name from the resource descriptor instead of hardcoded match
    let method_call = match resource.descriptor.singular.as_str() {
        "schema" => {
            quote! {
                pub fn #method_name(&self, catalog_name: String, schema_name: String) -> #client_name {
                    #client_name {
                        client: self.client.schema(&catalog_name, &schema_name),
                    }
                }
            }
        }
        "table" => {
            quote! {
                pub fn #method_name(&self, full_name: String) -> #client_name {
                    #client_name {
                        client: self.client.table(&full_name),
                    }
                }
            }
        }
        "volume" => {
            quote! {
                pub fn #method_name(&self, catalog_name: String, schema_name: String, volume_name: String) -> #client_name {
                    #client_name {
                        client: self.client.volume(catalog_name, schema_name, volume_name),
                    }
                }
            }
        }
        _ => {
            // Default case for simple resources - use single name parameter
            quote! {
                pub fn #method_name(&self, name: String) -> #client_name {
                    #client_name {
                        client: self.client.#method_name(&name),
                    }
                }
            }
        }
    };

    Some(method_call)
}

/// Helper function to check if a method is a list method
fn is_list_method(method: &MethodHandler<'_>) -> bool {
    matches!(method.plan.request_type, RequestType::List)
}

/// Generate method signature for Python typings
fn generate_method_typings_signature(method: &MethodHandler<'_>) -> Option<String> {
    let method_name = match &method.plan.request_type {
        RequestType::Get | RequestType::Update => method.plan.resource_client_method().to_string(),
        RequestType::Delete => method.plan.resource_client_method().to_string(),
        RequestType::List | RequestType::Create => method.plan.base_method_ident().to_string(),
        _ => return None,
    };

    let return_type = match &method.plan.request_type {
        RequestType::Delete => "None".to_string(),
        RequestType::List => {
            if let Some(items_field) = method.list_output_field() {
                let item_type = python_type_annotation(&items_field.unified_type);
                format!(
                    "List[{}]",
                    item_type.trim_start_matches("List[").trim_end_matches("]")
                )
            } else {
                "Any".to_string()
            }
        }
        _ => {
            if let Some(output_type) = method.output_type() {
                python_type_annotation_from_ident(&output_type)
            } else {
                "Any".to_string()
            }
        }
    };

    let parameters = generate_method_parameters_for_typings(method);
    let params_str = if parameters.is_empty() {
        "self".to_string()
    } else {
        format!("self, {}", parameters.join(", "))
    };

    let docstring = format_method_docstring_with_params(method);

    let params_vec = if params_str == "self" {
        vec!["self".to_string()]
    } else {
        params_str.split(", ").map(|s| s.to_string()).collect()
    };

    let docstring_text = if docstring.trim().is_empty() {
        None
    } else {
        Some(docstring.trim())
    };
    let method_stub =
        generate_method_template(&method_name, &params_vec, &return_type, docstring_text, 1);

    Some(method_stub)
}

/// Generate method parameters for Python typings
fn generate_method_parameters_for_typings(method: &MethodHandler<'_>) -> Vec<String> {
    let mut parameters = Vec::new();

    // Add required parameters (excluding path parameters for resource methods)
    for param in method.required_parameters() {
        if !param.is_path_param() {
            let param_type = python_type_annotation(param.field_type());
            parameters.push(format!("{}: {}", param.name(), param_type));
        }
    }

    // Add optional parameters
    for param in method.optional_parameters() {
        if !(is_list_method(method) && param.name() == "page_token") {
            let param_type = python_type_annotation(param.field_type());
            let param_type = if param_type.starts_with("Optional[") {
                param_type
            } else {
                format!("Optional[{}]", param_type)
            };
            parameters.push(format!("{}: {} = None", param.name(), param_type));
        }
    }

    parameters
}

/// Convert a unified type to Python type annotation
fn python_type_annotation(unified_type: &UnifiedType) -> String {
    use crate::parsing::CONVERTER;
    CONVERTER.unified_to_python_type(unified_type)
}

/// Convert an Ident to Python type annotation (fallback for complex types)
fn python_type_annotation_from_ident(ident: &syn::Ident) -> String {
    let type_str = ident.to_string();
    // Simple conversion for common Rust types to Python types
    match type_str.as_str() {
        "String" => "str".to_string(),
        "i32" | "i64" => "int".to_string(),
        "f32" | "f64" => "float".to_string(),
        "bool" => "bool".to_string(),
        "Vec < u8 >" => "bytes".to_string(),
        "()" => "None".to_string(),
        _ => {
            // Extract simple type name for complex types
            if let Some(simple_name) = type_str.split("::").last() {
                simple_name.trim().to_string()
            } else {
                type_str
            }
        }
    }
}

/// Format method docstring with Google-style parameter documentation
/// Format method docstring with parameters for documentation
fn format_method_docstring_with_params(method: &MethodHandler<'_>) -> String {
    let mut docstring_parts = Vec::new();

    // Add method description if available
    if let Some(doc) = method.plan.metadata.documentation.as_ref() {
        let cleaned_doc = clean_and_format_description(doc);
        if !cleaned_doc.is_empty() {
            // Apply refill to main description for better line distribution
            let refilled_doc = refill(&cleaned_doc, DOCS_TARGET_WIDTH);
            docstring_parts.push(refilled_doc);
        }
    }

    // Collect parameter documentation
    let param_docs = collect_parameter_documentation(method);
    if !param_docs.is_empty() {
        if !docstring_parts.is_empty() {
            docstring_parts.push("".to_string()); // Empty line before Args
        }
        docstring_parts.push("Args:".to_string());
        for (param_name, param_doc) in param_docs {
            let cleaned_param_doc = clean_and_format_description(&param_doc);
            // Use improved parameter description wrapping with hanging indentation
            let wrapped_param = format_parameter_description(&param_name, &cleaned_param_doc);
            docstring_parts.push(wrapped_param);
        }
    }

    // Add return type documentation if relevant
    let return_doc = get_return_type_documentation(method);
    if !return_doc.is_empty() {
        if !docstring_parts.is_empty() {
            docstring_parts.push("".to_string()); // Empty line before Returns
        }
        docstring_parts.push("Returns:".to_string());
        let cleaned_return_doc = clean_and_format_description(&return_doc);
        let wrapped_return = format_return_description(&cleaned_return_doc);
        docstring_parts.push(wrapped_return);
    }

    if docstring_parts.is_empty() {
        return String::new();
    }

    // Join sections with proper spacing - empty line only between main description and Args/Returns
    let mut result: Vec<String> = Vec::new();
    for (i, part) in docstring_parts.iter().enumerate() {
        if i > 0 && !part.is_empty() && !result.is_empty() {
            // Add single empty line before Args: and Returns: sections if there's a main description
            if (part.starts_with("Args:") || part.starts_with("Returns:"))
                && !result.is_empty()
                && !result.last().unwrap().is_empty()
            {
                result.push(String::new());
            }
        }
        result.push(part.clone());
    }
    result.join("\n")
}

/// Format a parameter description with proper hanging indentation and optimal line filling
fn format_parameter_description(param_name: &str, description: &str) -> String {
    let first_line_prefix = format!("    {}: ", param_name);
    let continuation_prefix = " ".repeat(first_line_prefix.len());

    // Calculate available widths (with reasonable margins)
    let first_line_width = DOCS_TARGET_WIDTH - first_line_prefix.len(); // Allow up to DOCS_TARGET_WIDTH chars total
    let continuation_width = DOCS_TARGET_WIDTH - continuation_prefix.len();

    // If the parameter name is very long, put description on next line
    if first_line_prefix.len() >= 90 {
        // Use refill to optimize line distribution for continuation-only format
        let optimally_filled = refill(description, continuation_width);
        let options = Options::new(continuation_width)
            .initial_indent(&continuation_prefix)
            .subsequent_indent(&continuation_prefix);
        return format!(
            "{}\n{}",
            first_line_prefix,
            fill(&optimally_filled, &options)
        );
    }

    // First try to create the description with hanging indent
    let options = Options::new(first_line_width)
        .initial_indent("")
        .subsequent_indent(&continuation_prefix);

    let initial_wrapped = fill(description, &options);

    // If we have multiple lines, use refill to optimize line distribution
    if initial_wrapped.contains('\n') {
        // Use refill to get better line distribution, then reapply hanging indent
        let refilled_desc = refill(description, continuation_width);

        // Now apply hanging indent to the refilled text
        let lines: Vec<&str> = refilled_desc.lines().collect();
        if lines.is_empty() {
            return first_line_prefix;
        }

        let mut result = format!("{}{}", first_line_prefix, lines[0]);
        for line in &lines[1..] {
            result.push_str(&format!("\n{}{}", continuation_prefix, line));
        }
        result
    } else {
        // Single line, use as-is
        format!("{}{}", first_line_prefix, initial_wrapped)
    }
}

/// Format a return description with proper indentation and optimal line filling
fn format_return_description(description: &str) -> String {
    let prefix = "    ";

    // First use refill to optimize line distribution
    let available_width = DOCS_TARGET_WIDTH - prefix.len();
    let refilled_desc = refill(description, available_width);

    // Then apply indentation
    let options = Options::new(available_width)
        .initial_indent(prefix)
        .subsequent_indent(prefix);

    fill(&refilled_desc, &options)
}

/// Clean and format a description text for docstring usage
fn clean_and_format_description(text: &str) -> String {
    let cleaned = text.trim();
    if cleaned.is_empty() {
        return String::new();
    }

    // Split into lines and clean each line
    let lines: Vec<&str> = cleaned.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    // Join lines with proper spacing, ensuring no leading/trailing whitespace issues
    let joined_text = lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    // Use textwrap to wrap long lines to DOCS_TARGET_WIDTH characters for better filling
    fill(&joined_text, DOCS_TARGET_WIDTH)
}

/// Collect parameter documentation for a method
fn collect_parameter_documentation(method: &MethodHandler<'_>) -> Vec<(String, String)> {
    let mut param_docs = Vec::new();

    // Add required parameters (excluding path parameters for resource methods)
    for param in method.required_parameters() {
        if !param.is_path_param() {
            if let Some(doc) = param.documentation() {
                let cleaned_doc = clean_and_format_description(doc);
                if !cleaned_doc.is_empty() {
                    param_docs.push((param.name().to_string(), cleaned_doc));
                }
            }
        }
    }

    // Add optional parameters
    for param in method.optional_parameters() {
        if !(is_list_method(method) && param.name() == "page_token") {
            if let Some(doc) = param.documentation() {
                let cleaned_doc = clean_and_format_description(doc);
                if !cleaned_doc.is_empty() {
                    param_docs.push((param.name().to_string(), cleaned_doc));
                }
            }
        }
    }

    param_docs
}

/// Get return type documentation for a method
fn get_return_type_documentation(method: &MethodHandler<'_>) -> String {
    match &method.plan.request_type {
        RequestType::Delete => "None".to_string(),
        RequestType::List => {
            if let Some(items_field) = method.list_output_field() {
                if let Some(doc) = items_field.documentation.as_ref() {
                    let cleaned_doc = clean_and_format_description(doc);
                    if cleaned_doc.is_empty() {
                        "List of items".to_string()
                    } else {
                        format!("List of {}", cleaned_doc)
                    }
                } else {
                    "List of items".to_string()
                }
            } else {
                "List of items".to_string()
            }
        }
        _ => {
            if let Some(output_message) = method.output_message() {
                if let Some(doc) = output_message.info.documentation.as_ref() {
                    let cleaned_doc = clean_and_format_description(doc);
                    if cleaned_doc.is_empty() {
                        "The requested resource".to_string()
                    } else {
                        cleaned_doc
                    }
                } else {
                    "The requested resource".to_string()
                }
            } else {
                "The requested resource".to_string()
            }
        }
    }
}

/// Generate resource accessor method for individual service client typings
fn generate_resource_accessor_method_for_typings(service: &ServiceHandler<'_>) -> Option<String> {
    let _resource = service.resource()?;
    let client_name = service.client_type().to_string();

    // Generate method based on the service type, not the resource type
    let (method_name, params, return_type) = match client_name.as_str() {
        "SchemaClient" => (
            "table",
            vec!["self".to_string(), "name: str".to_string()],
            "TableClient",
        ),
        "CatalogClient" => (
            "schema",
            vec!["self".to_string(), "name: str".to_string()],
            "SchemaClient",
        ),
        _ => return None,
    };

    let method_stub = generate_method_template(method_name, &params, return_type, None, 1);
    Some(method_stub)
}

/// Generate unified Python typings (.pyi file) for all services in a single file
pub(crate) fn generate_typings(services: &[ServiceHandler<'_>]) -> String {
    let metadata = services[0].metadata; // Access metadata
    let mut content = Vec::new();

    // Add imports (including enum)
    content.push("from __future__ import annotations".to_string());
    content.push("from typing import Optional, List, Dict, Any, Literal".to_string());
    content.push("import enum".to_string());
    content.push("".to_string());

    // Generate model classes
    let model_classes = generate_model_classes(metadata);
    content.extend(model_classes);

    // Generate enum classes
    let enum_classes = generate_enum_classes(metadata);
    content.extend(enum_classes);

    // Sort services for stable ordering by creating sorted indices
    let mut service_indices: Vec<usize> = (0..services.len()).collect();
    service_indices.sort_by_key(|&i| &services[i].plan.service_name);

    // Generate individual service client classes
    for &i in &service_indices {
        let service = &services[i];
        let service_class = generate_service_class_typings(service);
        content.push(service_class);
        content.push("".to_string());
    }

    // Generate main client class with sorted services
    let sorted_services: Vec<_> = service_indices.iter().map(|&i| &services[i]).collect();
    let main_client_class = generate_main_client_class_typings(&sorted_services);
    content.push(main_client_class);

    content.join("\n")
}

/// Generate typings for a single service client class
fn generate_service_class_typings(service: &ServiceHandler<'_>) -> String {
    let rust_client_ident = service.client_type();
    let client_ident = format!("{}", rust_client_ident);

    // For resource clients, only include resource-specific methods (not collection methods)
    // Sort methods for stable ordering
    let mut method_signatures: Vec<_> = service
        .methods()
        .filter(|method| !method.is_collection_method()) // Exclude collection methods from resource clients
        .filter_map(|method| {
            generate_method_typings_signature(&method)
                .map(|sig| (method.plan.base_method_ident().to_string(), sig))
        })
        .collect();

    method_signatures.sort_by(|a, b| a.0.cmp(&b.0));
    let mut methods: Vec<_> = method_signatures.into_iter().map(|(_, sig)| sig).collect();

    // Add resource accessor methods for individual service clients
    if let Some(_resource) = service.resource() {
        let resource_method = generate_resource_accessor_method_for_typings(service);
        if let Some(method) = resource_method {
            methods.push(method);
        }
    }

    let body_content = if methods.is_empty() {
        indent("...", "    ")
    } else {
        methods.join("\n")
    };

    generate_class_from_template(&client_ident, "", None, &body_content)
}

/// Generate typings for the main PyUnityCatalogClient class
fn generate_main_client_class_typings(services: &[&ServiceHandler<'_>]) -> String {
    // Services are already sorted by the caller, no need to sort again
    let mut collection_methods = services
        .iter()
        .flat_map(|service| {
            service
                .methods()
                .filter(|m| m.is_collection_method())
                .filter_map(|method| {
                    let method_name = method.plan.base_method_ident().to_string();
                    let return_type = match &method.plan.request_type {
                        RequestType::List => {
                            if let Some(items_field) = method.list_output_field() {
                                let item_type = python_type_annotation(&items_field.unified_type);
                                format!(
                                    "List[{}]",
                                    item_type.trim_start_matches("List[").trim_end_matches("]")
                                )
                            } else {
                                "Any".to_string()
                            }
                        }
                        RequestType::Create => {
                            if let Some(output_type) = method.output_type() {
                                python_type_annotation_from_ident(&output_type)
                            } else {
                                "Any".to_string()
                            }
                        }
                        _ => return None,
                    };

                    let parameters = generate_method_parameters_for_typings(&method);
                    let mut params = vec!["self".to_string()];
                    params.extend(parameters);

                    let docstring = format_method_docstring_with_params(&method);
                    let docstring_text = if docstring.trim().is_empty() {
                        None
                    } else {
                        Some(docstring.trim())
                    };

                    Some((
                        method_name.clone(),
                        generate_method_template(
                            &method_name,
                            &params,
                            &return_type,
                            docstring_text,
                            1,
                        ),
                    ))
                })
        })
        .collect::<Vec<_>>();

    // Sort collection methods for stable ordering
    collection_methods.sort_by(|a, b| a.0.cmp(&b.0));
    let main_collection_methods = collection_methods
        .into_iter()
        .map(|(_, method)| method)
        .collect::<Vec<_>>()
        .join("\n");

    let mut resource_methods = services
        .iter()
        .filter_map(|service| {
            if let Some(resource) = service.resource() {
                let method_name = resource.descriptor.singular.clone();
                let client_name = format!("{}", service.client_type());
                let params = match method_name.as_str() {
                    "schema" => vec![
                        "self".to_string(),
                        "catalog_name: str".to_string(),
                        "schema_name: str".to_string(),
                    ],
                    "table" => vec!["self".to_string(), "full_name: str".to_string()],
                    "volume" => vec![
                        "self".to_string(),
                        "catalog_name: str".to_string(),
                        "schema_name: str".to_string(),
                        "volume_name: str".to_string(),
                    ],
                    _ => vec!["self".to_string(), "name: str".to_string()],
                };

                Some((
                    method_name.clone(),
                    generate_method_template(&method_name, &params, &client_name, None, 1),
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // Sort resource accessor methods for stable ordering
    resource_methods.sort_by(|a, b| a.0.cmp(&b.0));
    let main_client_methods = resource_methods
        .into_iter()
        .map(|(_, method)| method)
        .collect::<Vec<_>>()
        .join("\n");

    let init_method = generate_method_template(
        "__init__",
        &[
            "self".to_string(),
            "base_url: str".to_string(),
            "token: Optional[str] = None".to_string(),
        ],
        "None",
        None,
        1,
    );

    let main_client_all_methods =
        if main_client_methods.is_empty() && main_collection_methods.is_empty() {
            format!("{}\n    ...", init_method)
        } else {
            [init_method, main_collection_methods, main_client_methods]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        };

    generate_class_from_template("PyUnityCatalogClient", "", None, &main_client_all_methods)
}

/// Generate model classes from metadata
fn generate_model_classes(metadata: &CodeGenMetadata) -> Vec<String> {
    let mut classes = Vec::new();

    // Only include Unity Catalog specific model classes that are exported to Python
    let unity_catalog_models = [
        "Catalog",
        "Schema",
        "Table",
        "Column",
        "Credential",
        "ExternalLocation",
        "Recipient",
        "Share",
        "DataObject",
        "DataObjectUpdate",
        "Volume",
        "AzureServicePrincipal",
        "AzureManagedIdentity",
        "AzureStorageKey",
        "RecipientToken",
    ];

    // Collect and sort message infos for stable ordering
    let mut sorted_messages: Vec<_> = metadata.messages.values().collect();
    sorted_messages.sort_by_key(|msg| &msg.name);

    // First collect matching messages, then sort them by simple name for stable ordering
    let mut matching_messages: Vec<_> = sorted_messages
        .into_iter()
        .filter(|message_info| {
            message_info.name.contains("unitycatalog") && {
                let simple_name = extract_simple_type_name(&message_info.name);
                unity_catalog_models.contains(&simple_name.as_str())
            }
        })
        .collect();

    matching_messages.sort_by_key(|msg| extract_simple_type_name(&msg.name));

    for message_info in matching_messages {
        let class_def = generate_model_class_definition(message_info);
        classes.push(class_def);
        classes.push("".to_string()); // Add blank line between classes
    }

    classes
}

/// Generate enum classes from metadata
fn generate_enum_classes(metadata: &CodeGenMetadata) -> Vec<String> {
    let mut enums = Vec::new();

    // Only include Unity Catalog specific enum classes that are exported to Python
    let unity_catalog_enums = [
        "CatalogType",
        "Purpose",
        "TableType",
        "ColumnTypeName",
        "DataSourceFormat",
        "VolumeType",
        "DataObjectType",
        "HistoryStatus",
        "Action",
        "AuthenticationType",
    ];

    // Collect and sort enum infos for stable ordering
    let mut sorted_enums: Vec<_> = metadata.enums.values().collect();
    sorted_enums.sort_by_key(|enum_info| &enum_info.name);

    // First collect matching enums, then sort them by simple name for stable ordering
    let mut matching_enums: Vec<_> = sorted_enums
        .into_iter()
        .filter(|enum_info| {
            let simple_name = extract_simple_type_name(&enum_info.name);
            unity_catalog_enums.contains(&simple_name.as_str())
        })
        .collect();

    matching_enums.sort_by_key(|enum_info| extract_simple_type_name(&enum_info.name));

    for enum_info in matching_enums {
        let enum_def = generate_enum_class_definition(enum_info);
        enums.push(enum_def);
        enums.push("".to_string()); // Add blank line between enums
    }

    enums
}

/// Generate single model class definition
fn generate_model_class_definition(message: &MessageInfo) -> String {
    let class_name = extract_simple_type_name(&message.name);
    let docstring = message
        .documentation
        .as_ref()
        .map(|doc| clean_and_format_description(doc))
        .filter(|doc| !doc.is_empty());

    // Generate field definitions with stable ordering
    let mut field_indices: Vec<usize> = (0..message.fields.len()).collect();
    field_indices.sort_by_key(|&i| &message.fields[i].name);

    let field_definitions = field_indices
        .iter()
        .map(|&i| generate_field_definition(&message.fields[i]))
        .filter(|def| !def.is_empty())
        .collect::<Vec<_>>();

    // Generate oneof variant field definitions with stable ordering
    let mut oneof_field_definitions = Vec::new();
    for &i in &field_indices {
        let field = &message.fields[i];
        if let Some(variants) = &field.oneof_variants {
            let mut variant_indices: Vec<usize> = (0..variants.len()).collect();
            variant_indices.sort_by_key(|&j| &variants[j].field_name);
            for &j in &variant_indices {
                let variant = &variants[j];
                let safe_field_name = sanitize_python_field_name(&variant.field_name);
                // Get the type from the variant's rust_type, convert to Python type
                let python_type = match variant.rust_type.as_str() {
                    "String" => "str",
                    "i32" | "i64" => "int",
                    "f32" | "f64" => "float",
                    "bool" => "bool",
                    "Vec<u8>" => "bytes",
                    _ => {
                        // For complex types, extract simple name
                        variant
                            .rust_type
                            .split("::")
                            .last()
                            .unwrap_or(&variant.rust_type)
                    }
                };

                let mut field_def = format!("    {}: Optional[{}]", safe_field_name, python_type);
                if let Some(doc) = &variant.documentation {
                    let cleaned_doc = clean_and_format_description(doc);
                    if !cleaned_doc.is_empty() {
                        field_def.push_str(&format!("\n    \"\"\"{}\"\"\"", cleaned_doc));
                    }
                }
                oneof_field_definitions.push(field_def);
            }
        }
    }

    // Combine all field definitions
    let mut all_field_definitions = field_definitions;
    all_field_definitions.extend(oneof_field_definitions);

    let body_content = if all_field_definitions.is_empty() {
        indent("...", "    ")
    } else {
        let mut content = all_field_definitions.join("\n");
        content.push_str("\n\n");
        content.push_str(&generate_constructor_definition(message));
        content
    };

    generate_class_from_template(&class_name, "", docstring.as_deref(), &body_content)
}

/// Generate single enum class definition
fn generate_enum_class_definition(enum_info: &EnumInfo) -> String {
    let enum_name = extract_simple_type_name(&enum_info.name);
    let docstring = enum_info
        .documentation
        .as_ref()
        .map(|doc| clean_and_format_description(doc))
        .filter(|doc| !doc.is_empty());

    let body_content = if enum_info.values.is_empty() {
        indent("...", "    ")
    } else {
        // Sort enum values for stable ordering
        let mut value_indices: Vec<usize> = (0..enum_info.values.len()).collect();
        value_indices.sort_by_key(|&i| &enum_info.values[i].name);

        let mut enum_values = Vec::new();
        for &i in &value_indices {
            let value = &enum_info.values[i];
            enum_values.push(format!("{} = \"{}\"", value.name, value.name));

            // Add value docstring if available
            if let Some(doc) = &value.documentation {
                let cleaned_doc = clean_and_format_description(doc);
                if !cleaned_doc.is_empty() {
                    enum_values.push(format!("\"\"\"{}\"\"\"", cleaned_doc));
                }
            }
        }
        indent(&enum_values.join("\n"), "    ")
    };

    generate_class_from_template(&enum_name, "enum.Enum", docstring.as_deref(), &body_content)
}

/// Generate single field definition with type annotation and docstring
fn generate_field_definition(field: &MessageField) -> String {
    // Skip oneof fields - they're handled as individual variant parameters
    if field.oneof_variants.is_some() {
        return String::new();
    }

    // Extract simple field name from fully qualified protobuf names
    let field_name = if field.name.contains('.') {
        field.name.split('.').next_back().unwrap_or(&field.name)
    } else {
        &field.name
    };

    // Handle Python reserved keywords by adding underscore suffix
    let safe_field_name = sanitize_python_field_name(field_name);
    let mut type_annotation = python_type_annotation(&field.unified_type);

    // Handle repeated fields first (they can also be optional)
    if field.repeated {
        // Remove any existing List wrapper to avoid nested List types
        if type_annotation.starts_with("List[") && type_annotation.ends_with("]") {
            type_annotation = type_annotation[5..type_annotation.len() - 1].to_string();
        }
        type_annotation = format!("List[{}]", type_annotation);
    }

    // Handle optional fields
    if field.optional && !type_annotation.starts_with("Optional[") {
        type_annotation = format!("Optional[{}]", type_annotation);
    }

    let mut lines = Vec::new();
    lines.push(format!("    {}: {}", safe_field_name, type_annotation));

    // Add field docstring if available
    if let Some(doc) = &field.documentation {
        let cleaned_doc = clean_and_format_description(doc);
        if !cleaned_doc.is_empty() {
            let formatted_docstring = format_field_docstring(&cleaned_doc);
            lines.push(formatted_docstring);
        }
    }

    lines.join("\n")
}

/// Generate constructor definition for a message class
fn generate_constructor_definition(message: &MessageInfo) -> String {
    let mut params = vec!["self".to_string()];

    // Separate required and optional fields with stable ordering
    let mut field_indices: Vec<usize> = (0..message.fields.len()).collect();
    field_indices.sort_by_key(|&i| &message.fields[i].name);

    let mut required_fields = Vec::new();
    let mut optional_fields = Vec::new();

    for &i in &field_indices {
        let field = &message.fields[i];
        // Skip oneof fields - we'll handle them as individual variant parameters
        if field.oneof_variants.is_some() {
            continue;
        }

        if !field.optional && !field.repeated {
            required_fields.push(field);
        } else {
            optional_fields.push(field);
        }
    }

    // Add required parameters
    for field in &required_fields {
        let mut type_annotation = python_type_annotation(&field.unified_type);

        // Handle repeated fields for required parameters
        if field.repeated {
            if type_annotation.starts_with("List[") && type_annotation.ends_with("]") {
                type_annotation = type_annotation[5..type_annotation.len() - 1].to_string();
            }
            type_annotation = format!("List[{}]", type_annotation);
        }

        // Extract simple field name from fully qualified protobuf names
        let simple_field_name = if field.name.contains('.') {
            field.name.split('.').next_back().unwrap_or(&field.name)
        } else {
            &field.name
        };

        let safe_field_name = sanitize_python_field_name(simple_field_name);
        params.push(format!("{}: {}", safe_field_name, type_annotation));
    }

    // Add optional parameters with defaults
    for field in &optional_fields {
        let mut type_annotation = python_type_annotation(&field.unified_type);

        // Extract simple field name from fully qualified protobuf names
        let simple_field_name = if field.name.contains('.') {
            field.name.split('.').next_back().unwrap_or(&field.name)
        } else {
            &field.name
        };

        let safe_field_name = sanitize_python_field_name(simple_field_name);

        if field.repeated {
            // Remove any existing List wrapper to avoid nested List types
            if type_annotation.starts_with("List[") && type_annotation.ends_with("]") {
                type_annotation = type_annotation[5..type_annotation.len() - 1].to_string();
            }
            type_annotation = format!("Optional[List[{}]]", type_annotation);
            params.push(format!("{}: {} = None", safe_field_name, type_annotation));
        } else if field.optional {
            if !type_annotation.starts_with("Optional[") {
                type_annotation = format!("Optional[{}]", type_annotation);
            }
            params.push(format!("{}: {} = None", safe_field_name, type_annotation));
        } else {
            params.push(format!("{}: {} = None", safe_field_name, type_annotation));
        }
    }

    // Add oneof variant parameters (all optional since only one can be set)
    // Sort fields for stable ordering
    for &i in &field_indices {
        let field = &message.fields[i];
        if let Some(variants) = &field.oneof_variants {
            let mut variant_indices: Vec<usize> = (0..variants.len()).collect();
            variant_indices.sort_by_key(|&j| &variants[j].field_name);
            for &j in &variant_indices {
                let variant = &variants[j];
                let safe_field_name = sanitize_python_field_name(&variant.field_name);
                // Get the type from the variant's rust_type, convert to Python type
                let python_type = match variant.rust_type.as_str() {
                    "String" => "str",
                    "i32" | "i64" => "int",
                    "f32" | "f64" => "float",
                    "bool" => "bool",
                    "Vec<u8>" => "bytes",
                    _ => {
                        // For complex types, extract simple name
                        variant
                            .rust_type
                            .split("::")
                            .last()
                            .unwrap_or(&variant.rust_type)
                    }
                };
                params.push(format!(
                    "{}: Optional[{}] = None",
                    safe_field_name, python_type
                ));
            }
        }
    }

    generate_method_template("__init__", &params, "None", None, 1)
}

/// Extract simple type name from fully qualified protobuf type
fn extract_simple_type_name(full_type: &str) -> String {
    full_type
        .split('.')
        .next_back()
        .unwrap_or(full_type)
        .to_string()
}

/// Helper function to create clean multiline strings without leading indentation
fn clean_text(text: &str) -> String {
    dedent(text).trim().to_string()
}

/// Template-based Python class generator using textwrap for cleaner code
fn generate_class_from_template(
    class_name: &str,
    class_type: &str, // "class" or "enum"
    docstring: Option<&str>,
    body_content: &str,
) -> String {
    let template = if docstring.is_some() {
        format!(
            "class {}({}):\n    \"\"\"{}\"\"\"\n{}",
            class_name,
            class_type,
            docstring.unwrap_or(""),
            body_content
        )
    } else {
        format!("class {}({}):\n{}", class_name, class_type, body_content)
    };

    clean_text(&template)
}

/// Generate method signature with proper indentation and docstring
fn generate_method_template(
    method_name: &str,
    params: &[String],
    return_type: &str,
    docstring: Option<&str>,
    indent_level: usize,
) -> String {
    let indent_str = "    ".repeat(indent_level);

    // Improved parameter formatting with better alignment
    let params_str = if params.len() <= 3 {
        params.join(", ")
    } else {
        // For long parameter lists, align parameters properly
        let param_indent = format!("{}    ", indent_str);
        let formatted_params: Vec<String> = params
            .iter()
            .map(|p| format!("{}{},", param_indent, p))
            .collect();
        format!("\n{}", formatted_params.join("\n").trim_end_matches(','))
    };

    let signature = if params.len() <= 3 {
        format!("def {}({}) -> {}:", method_name, params_str, return_type)
    } else {
        format!(
            "def {}(\n{}\n{}) -> {}:",
            method_name, params_str, indent_str, return_type
        )
    };

    let mut result = indent(&signature, &indent_str);

    if let Some(doc) = docstring {
        result.push('\n');
        let formatted_doc = format_method_docstring_for_template(doc, indent_level);
        result.push_str(&formatted_doc);
    }

    result.push_str(&format!("\n{}    ...", indent_str));
    result
}

/// Format method docstring with proper indentation for template usage
fn format_method_docstring_for_template(docstring: &str, indent_level: usize) -> String {
    let base_indent = "    ".repeat(indent_level + 1); // +1 for docstring indentation

    // Split docstring into sections: main description vs Args/Returns sections
    let mut sections = Vec::new();
    let mut current_section = Vec::new();
    let mut in_args_or_returns = false;

    for line in docstring.lines() {
        let trimmed = line.trim();
        if trimmed == "Args:" || trimmed == "Returns:" {
            // Finish current section before starting Args/Returns
            if !current_section.is_empty() {
                sections.push((false, current_section.join("\n")));
                current_section.clear();
            }
            in_args_or_returns = true;
            current_section.push(line.to_string());
        } else if in_args_or_returns {
            // Continue collecting Args/Returns section
            current_section.push(line.to_string());
        } else {
            // Main description section
            current_section.push(line.to_string());
        }
    }

    // Add final section
    if !current_section.is_empty() {
        sections.push((in_args_or_returns, current_section.join("\n")));
    }

    // Process each section appropriately
    let processed_sections: Vec<String> = sections
        .into_iter()
        .map(|(is_args_returns, content)| {
            if is_args_returns {
                // Args/Returns sections - preserve structure and formatting
                content
            } else {
                // Main description - already refilled, just ensure proper line breaks
                content
            }
        })
        .collect();

    let full_content = processed_sections.join("\n\n");

    let formatted_doc = if full_content.lines().count() == 1 {
        format!("\"\"\"{}\"\"\"", full_content)
    } else {
        let mut lines = vec!["\"\"\"".to_string()];
        for line in full_content.lines() {
            lines.push(line.to_string());
        }
        lines.push("\"\"\"".to_string());
        lines.join("\n")
    };

    indent(&formatted_doc, &base_indent)
}

/// Format field docstring with proper indentation and line wrapping
fn format_field_docstring(description: &str) -> String {
    let base_indent = "    "; // Field-level indentation
    let available_width = DOCS_TARGET_WIDTH - base_indent.len() - 6; // Account for """ quotes, allow up to DOCS_TARGET_WIDTH chars

    // First, use refill to optimize the text wrapping for the available space
    let optimally_filled = refill(description, available_width);

    // Check if it fits on a single line
    let single_line_test = format!(
        "{}\"\"\"{}\"\"\"",
        base_indent,
        optimally_filled.replace('\n', " ")
    );
    if !optimally_filled.contains('\n') && single_line_test.len() <= DOCS_TARGET_WIDTH {
        return single_line_test;
    }

    // Multi-line docstring with proper indentation
    let mut result = format!("{}\"\"\"", base_indent);
    for line in optimally_filled.lines() {
        if line.trim().is_empty() {
            result.push('\n');
        } else {
            result.push_str(&format!("\n{}{}", base_indent, line));
        }
    }
    result.push_str(&format!("\n{}\"\"\"", base_indent));
    result
}

/// Sanitize field names to avoid Python reserved keywords
fn sanitize_python_field_name(field_name: &str) -> String {
    match field_name {
        "not" => "not_".to_string(),
        "and" => "and_".to_string(),
        "or" => "or_".to_string(),
        "is" => "is_".to_string(),
        "in" => "in_".to_string(),
        "def" => "def_".to_string(),
        "class" => "class_".to_string(),
        "if" => "if_".to_string(),
        "else" => "else_".to_string(),
        "for" => "for_".to_string(),
        "while" => "while_".to_string(),
        "try" => "try_".to_string(),
        "except" => "except_".to_string(),
        "finally" => "finally_".to_string(),
        "with" => "with_".to_string(),
        "as" => "as_".to_string(),
        "import" => "import_".to_string(),
        "from" => "from_".to_string(),
        "pass" => "pass_".to_string(),
        "break" => "break_".to_string(),
        "continue" => "continue_".to_string(),
        "return" => "return_".to_string(),
        "yield" => "yield_".to_string(),
        "raise" => "raise_".to_string(),
        "assert" => "assert_".to_string(),
        "del" => "del_".to_string(),
        "global" => "global_".to_string(),
        "nonlocal" => "nonlocal_".to_string(),
        "lambda" => "lambda_".to_string(),
        "None" => "None_".to_string(),
        "True" => "True_".to_string(),
        "False" => "False_".to_string(),
        "async" => "async_".to_string(),
        "await" => "await_".to_string(),
        _ => field_name.to_string(),
    }
}
