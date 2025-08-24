//! Python bindings generation module
//!
//! This module generates Python bindings for Unity Catalog services using PyO3.
//! It creates wrapper structs that expose the Rust client functionality to Python
//! while handling async operations and error conversion.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Path;

use super::format_tokens;
use crate::analysis::{MethodPlan, RequestType, ServicePlan};
use crate::utils::strings;

pub fn main_module(services: &[ServicePlan]) -> String {
    let service_modules: Vec<TokenStream> = services
        .iter()
        .map(|s| {
            let module_name = format_ident!("{}", s.base_path);
            quote! { pub mod #module_name; }
        })
        .collect();
    let uc_client_module = collection_client_struct(services);

    let tokens = quote! {
        // Service modules
        #(#service_modules)*

        #uc_client_module
    };

    format_tokens(tokens)
}

/// Generate Python bindings for a service
pub(crate) fn generate(service: &ServicePlan) -> Result<String, Box<dyn std::error::Error>> {
    let client_methods: Vec<_> = service
        .methods
        .iter()
        .filter_map(|m| resource_client_method(m, service))
        .collect();

    let client_code = resource_client_struct(&client_methods, service);

    Ok(client_code)
}

/// Generate Python client struct definition
fn resource_client_struct(methods: &[TokenStream], service: &ServicePlan) -> String {
    let base_name = service
        .handler_name
        .strip_suffix("Handler")
        .unwrap_or(&service.handler_name);

    let client_name = format!("Py{}Client", base_name);
    let client_ident = format_ident!("{}", client_name);

    let rust_client_name = format!("{}Client", base_name);
    let rust_client_ident = format_ident!("{}", rust_client_name);

    let mod_path: Path = syn::parse_str(&format!(
        "unitycatalog_common::models::{}::v1",
        service.base_path
    ))
    .unwrap();

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

fn collection_client_struct(services: &[ServicePlan]) -> TokenStream {
    let methods: Vec<TokenStream> = services
        .iter()
        .flat_map(|s| {
            s.methods
                .iter()
                .flat_map(|m| collection_client_method(m, s))
        })
        .collect();

    let mod_paths: Vec<TokenStream> = services
        .iter()
        .map(|s| {
            let mod_path: Path =
                syn::parse_str(&format!("unitycatalog_common::models::{}::v1", s.base_path))
                    .unwrap();
            quote! { use #mod_path::*; }
        })
        .collect();

    quote! {
        use std::collections::HashMap;
        use futures::stream::TryStreamExt;
        use pyo3::prelude::*;
        use unitycatalog_client::{UnityCatalogClient};
        use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
        use crate::runtime::get_runtime;
        #(#mod_paths)*

        #[pyclass(name = "UnityCatalogClient")]
        pub struct PyUnityCatalogClientABC {
            client: UnityCatalogClient
        }

        #[pymethods]
        impl PyUnityCatalogClientABC {
            #[new]
            #[pyo3(signature = (base_url, token = None))]
            pub fn new(base_url: String, token: Option<String>) -> PyResult<Self> {
                let client = if let Some(token) = token {
                    cloud_client::CloudClient::new_with_token(token)
                } else {
                    cloud_client::CloudClient::new_unauthenticated()
                };
                let base_url = base_url.parse().unwrap();
                Ok(Self { client: UnityCatalogClient::new(client, base_url) })
            }
            #(#methods)*
        }
    }
}

/// Generate Python method wrapper
fn resource_client_method(method: &MethodPlan, _service: &ServicePlan) -> Option<TokenStream> {
    let method_name = format_ident!("{}", method.handler_function_name);
    let response_type = extract_response_type(&method.metadata.output_type);
    let response_type_ident = format_ident!("{}", response_type);

    let code = match &method.request_type {
        RequestType::Get => generate_resource_method(response_type_ident, method),
        RequestType::Update => generate_resource_method(response_type_ident, method),
        RequestType::Delete => generate_delete_method(method_name, method),
        _ => return None,
    };
    Some(code)
}

fn collection_client_method(method: &MethodPlan, _service: &ServicePlan) -> Option<TokenStream> {
    if !method.is_collection_client_method {
        return None;
    }

    let response_type = extract_response_type(&method.metadata.output_type);
    let response_type_ident = format_ident!("{}", response_type);

    let code = match &method.request_type {
        // RequestType::List => generate_list_method(response_type_ident, method),
        // RequestType::Get => generate_resource_method(response_type_ident, method),
        RequestType::Create => generate_create_method(response_type_ident, method),
        // RequestType::Update => generate_resource_method(response_type_ident, method),
        // RequestType::Delete => generate_delete_method(method_name, method),
        _ => return None,
    };
    Some(code)
}

/// Generate List method (returns Vec<T> and uses try_collect)
fn generate_list_method(response_type: syn::Ident, method: &MethodPlan) -> TokenStream {
    let method_name = method.collection_client_method();

    let param_defs = generate_param_definitions(method);
    let pyo3_signature = generate_pyo3_signature(method);
    let client_call = generate_client_call(method, true); // true for list methods

    // Extract inner type from response (e.g., ListCatalogsResponse -> CatalogInfo)
    let inner_type = extract_list_inner_type(&response_type.to_string());
    let inner_type_ident = format_ident!("{}", inner_type);

    quote! {
        #pyo3_signature
        pub fn #method_name(
            &self,
            py: Python,
            #(#param_defs,)*
        ) -> PyUnityCatalogResult<Vec<#inner_type_ident>> {
            let runtime = get_runtime(py)?;
            py.allow_threads(|| {
                let result = runtime.block_on(async move {
                    #client_call.try_collect::<Vec<_>>().await
                })?;
                Ok::<_, PyUnityCatalogError>(result)
            })
        }
    }
}

/// Generate a method call to a resource client.
fn generate_resource_method(response_type: syn::Ident, method: &MethodPlan) -> TokenStream {
    let method_name = method.resource_client_method();

    let param_defs = generate_resource_param_definitions(method);
    let pyo3_signature = generate_pyo3_signature(method);
    let client_call = generate_resource_client_call(method, false);
    let (_required_params, builder_calls) = generate_builder_pattern(method);

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

/// Generate Create method (uses builder pattern)
fn generate_create_method(response_type: syn::Ident, method: &MethodPlan) -> TokenStream {
    let method_name = method.collection_client_method();

    let param_defs = generate_param_definitions(method);
    let pyo3_signature = generate_pyo3_signature(method);
    let client_call = generate_resource_client_call(method, false);
    let (_required_params, builder_calls) = generate_builder_pattern(method);

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
fn generate_delete_method(method_name: syn::Ident, method: &MethodPlan) -> TokenStream {
    let param_defs = generate_resource_param_definitions(method);
    let pyo3_signature = generate_pyo3_signature(method);
    let client_call = generate_resource_client_call(method, false);
    let (_required_params, builder_calls) = generate_builder_pattern(method);

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
fn generate_resource_param_definitions(method: &MethodPlan) -> Vec<TokenStream> {
    let mut params = Vec::new();

    // Add required body fields (non-optional)
    for body_field in &method.body_fields {
        if !body_field.optional {
            let param_name = format_ident!("{}", body_field.name);
            let rust_type = convert_to_python_type(&body_field.rust_type, false);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    // Add required query parameters
    for query_param in &method.query_params {
        if !query_param.optional {
            let param_name = format_ident!("{}", query_param.name);
            let rust_type = convert_to_python_type(&query_param.rust_type, true);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    // Add optional body fields
    for body_field in &method.body_fields {
        if body_field.optional {
            let param_name = format_ident!("{}", body_field.name);
            let rust_type = convert_to_python_type(&body_field.rust_type, true);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    // Add optional query parameters
    for query_param in &method.query_params {
        if query_param.optional {
            let param_name = format_ident!("{}", query_param.name);
            let rust_type = convert_to_python_type(&query_param.rust_type, true);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    params
}

/// Generate parameter definitions for method signature
fn generate_param_definitions(method: &MethodPlan) -> Vec<TokenStream> {
    let mut params = Vec::new();

    // Add required path parameters first (these don't have Option wrapper)
    for path_param in &method.path_params {
        let param_name = format_ident!("{}", path_param.field_name);
        let rust_type = convert_to_python_type(&path_param.rust_type, false);
        params.push(quote! { #param_name: #rust_type });
    }

    // Add required body fields (non-optional)
    for body_field in &method.body_fields {
        if !body_field.optional {
            let param_name = format_ident!("{}", body_field.name);
            // Use the original field type to get proper enum handling
            let python_type = get_python_type_from_method_field(method, &body_field.name, false);
            let rust_type = convert_to_python_type(&python_type, false);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    // Add optional query parameters
    for query_param in &method.query_params {
        let param_name = format_ident!("{}", query_param.name);
        let rust_type = convert_to_python_type(&query_param.rust_type, true);
        params.push(quote! { #param_name: #rust_type });
    }

    // Add optional body fields
    for body_field in &method.body_fields {
        if body_field.optional {
            let param_name = format_ident!("{}", body_field.name);
            // Use the original field type to get proper enum handling
            let python_type = get_python_type_from_method_field(method, &body_field.name, true);
            let rust_type = convert_to_python_type(&python_type, true);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    params
}

/// Generate PyO3 signature annotation
fn generate_pyo3_signature(method: &MethodPlan) -> TokenStream {
    let mut signature_parts = Vec::new();

    for body_field in &method.body_fields {
        if !body_field.optional {
            signature_parts.push(body_field.name.clone());
        }
    }

    // Optional parameters with defaults
    for query_param in &method.query_params {
        if !query_param.optional {
            signature_parts.push(format!("{} = None", query_param.name));
        }
    }

    for body_field in &method.body_fields {
        if body_field.optional {
            signature_parts.push(format!("{} = None", body_field.name));
        }
    }

    // Optional parameters with defaults
    for query_param in &method.query_params {
        if query_param.optional {
            signature_parts.push(format!("{} = None", query_param.name));
        }
    }

    if signature_parts.is_empty() {
        quote! {}
    } else {
        // Simplified approach - just generate the signature string as a comment for now
        // The actual PyO3 signature generation can be added later
        quote! {}
    }
}

/// Generate resource client call
///
/// generated the invocation for a client call on a resource client.
/// Specifically this assumes that all path parameters which identify
/// the resource are encoded in the client and don't need to be passed
/// as arguments.
fn generate_resource_client_call(method: &MethodPlan, _is_list: bool) -> TokenStream {
    let method_name = method.resource_client_method();
    let mut args = Vec::new();

    // Add required body fields as direct arguments
    for body_field in &method.body_fields {
        if !body_field.optional {
            let param_name = format_ident!("{}", body_field.name);
            args.push(quote! { #param_name });
        }
    }

    // Add required query parameters
    for query_param in &method.query_params {
        if !query_param.optional {
            let param_name = format_ident!("{}", query_param.name);
            args.push(quote! { #param_name });
        }
    }

    quote! {
        self.client.#method_name(#(#args,)*)
    }
}

/// Generate client method call
fn generate_client_call(method: &MethodPlan, _is_list: bool) -> TokenStream {
    let method_name = format_ident!("{}", method.handler_function_name);
    let mut args = Vec::new();

    // Add path parameters as direct arguments
    for path_param in &method.path_params {
        let param_name = format_ident!(
            "{}",
            strings::operation_to_method_name(&path_param.field_name)
        );
        args.push(quote! { #param_name });
    }

    // Add required body fields as direct arguments
    for body_field in &method.body_fields {
        if !body_field.optional {
            let param_name =
                format_ident!("{}", strings::operation_to_method_name(&body_field.name));
            args.push(quote! { #param_name });
        }
    }

    // Add optional query parameters
    for query_param in &method.query_params {
        let param_name = format_ident!("{}", strings::operation_to_method_name(&query_param.name));
        args.push(quote! { #param_name });
    }

    quote! {
        self.client.#method_name(#(#args,)*)
    }
}

/// Generate builder pattern calls for Create/Update methods
fn generate_builder_pattern(method: &MethodPlan) -> (Vec<TokenStream>, Vec<TokenStream>) {
    let mut required_params = Vec::new();
    let mut builder_calls = Vec::new();

    for body_field in &method.body_fields {
        if !body_field.optional {
            let param_name =
                format_ident!("{}", strings::operation_to_method_name(&body_field.name));
            required_params.push(quote! { #param_name });
        }
    }

    // Optional parameters become builder calls
    for query_param in &method.query_params {
        let param_name = format_ident!("{}", strings::operation_to_method_name(&query_param.name));
        let with_method = format_ident!("with_{}", query_param.name);
        builder_calls.push(quote! {
            request = request.#with_method(#param_name);
        });
    }

    for body_field in &method.body_fields {
        if body_field.optional {
            let param_name =
                format_ident!("{}", strings::operation_to_method_name(&body_field.name));
            let with_method = format_ident!("with_{}", body_field.name);

            // Special handling for HashMap/properties fields
            if body_field.rust_type.contains("HashMap") {
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

    (required_params, builder_calls)
}

/// Convert Rust type to Python-compatible type annotation
fn convert_to_python_type(rust_type: &str, force_optional: bool) -> TokenStream {
    let base_type = if rust_type.starts_with("Option<") {
        // Extract inner type from Option<T>
        rust_type
            .strip_prefix("Option<")
            .and_then(|s| s.strip_suffix(">"))
            .unwrap_or(rust_type)
    } else {
        rust_type
    };

    let converted = convert_basic_type(base_type);

    if force_optional || rust_type.starts_with("Option<") {
        quote! { Option<#converted> }
    } else {
        converted
    }
}

/// Convert basic Rust types to Python-compatible types
fn convert_basic_type(rust_type: &str) -> TokenStream {
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

/// Extract inner type from List response types
fn extract_list_inner_type(response_type: &str) -> String {
    // Convert "ListCatalogsResponse" -> "CatalogInfo"
    // Convert "ListSchemasResponse" -> "SchemaInfo"
    if let Some(stripped) = response_type.strip_prefix("List") {
        if let Some(base) = stripped.strip_suffix("Response") {
            // Handle plural -> singular conversion
            let singular = if base.ends_with("s") {
                &base[..base.len() - 1]
            } else {
                base
            };
            format!("{}Info", singular)
        } else {
            response_type.to_string()
        }
    } else {
        response_type.to_string()
    }
}

/// Extract response type name from full type path
fn extract_response_type(type_path: &str) -> String {
    type_path
        .split('.')
        .next_back()
        .unwrap_or(type_path)
        .to_string()
}

/// Get the correct Python type for a method field, handling enums properly
fn get_python_type_from_method_field(
    method: &MethodPlan,
    field_name: &str,
    is_optional: bool,
) -> String {
    // Look for the field in the method metadata to get the original field type
    for field in &method.metadata.input_fields {
        if field.name == field_name {
            return python_field_type_to_rust_type(&field.field_type, is_optional);
        }
    }

    // Fallback to existing logic if field not found
    if is_optional {
        format!("Option<{}>", "String")
    } else {
        "String".to_string()
    }
}

/// Convert protobuf field type to Python-compatible Rust type
/// This is a Python-specific version that handles enums properly
fn python_field_type_to_rust_type(field_type: &str, is_optional: bool) -> String {
    let base_type = match field_type {
        "TYPE_STRING" => "String".to_string(),
        "TYPE_INT32" => "i32".to_string(),
        "TYPE_INT64" => "i64".to_string(),
        "TYPE_BOOL" => "bool".to_string(),
        "TYPE_DOUBLE" => "f64".to_string(),
        "TYPE_FLOAT" => "f32".to_string(),
        "TYPE_BYTES" => "Vec<u8>".to_string(),
        _ => {
            // For message types, extract the simple name
            if field_type.ends_with("PropertiesEntry") {
                "HashMap<String, String>".to_string()
            } else if field_type.starts_with("TYPE_MESSAGE:") {
                strings::extract_simple_type_name(&field_type[13..])
            } else if field_type.starts_with("TYPE_ENUM:") {
                // For Python generation, use the actual enum type name instead of i32
                strings::extract_simple_type_name(&field_type[10..])
            } else if let Some(oneof_name) = field_type.strip_prefix("TYPE_ONEOF:") {
                // Extract the oneof name and create an enum type
                strings::extract_simple_type_name(oneof_name)
            } else {
                // Default to String for unknown types
                "String".to_string()
            }
        }
    };

    if is_optional {
        format!("Option<{}>", base_type)
    } else {
        base_type
    }
}
