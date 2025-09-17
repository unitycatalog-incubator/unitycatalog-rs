//! Python bindings generation module
//!
//! This module generates Python bindings for Unity Catalog services using PyO3.
//! It creates wrapper structs that expose the Rust client functionality to Python
//! while handling async operations and error conversion.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::format_tokens;
use crate::analysis::{MethodPlan, RequestType};
use crate::codegen::{MethodHandler, ServiceHandler};
use crate::utils::strings;

pub fn main_module(services: &[ServiceHandler<'_>]) -> String {
    let service_modules: Vec<TokenStream> = services
        .iter()
        .map(|s| {
            let module_name = format_ident!("{}", s.plan.base_path);
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
pub(crate) fn generate(service: &ServiceHandler<'_>) -> Result<String, Box<dyn std::error::Error>> {
    let client_methods: Vec<_> = service
        .methods()
        .filter_map(|m| resource_client_method(service, &m))
        .collect();

    let client_code = resource_client_struct(service, &client_methods);

    Ok(client_code)
}

/// Generate Python client struct definition
fn resource_client_struct(service: &ServiceHandler<'_>, methods: &[TokenStream]) -> String {
    let client_ident = format_ident!("{}", format!("Py{}", service.client_type()));
    let rust_client_ident = service.client_type();
    let rust_client_name = rust_client_ident.to_string();

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
    let methods: Vec<TokenStream> = services
        .iter()
        .flat_map(|s| {
            s.methods().filter_map(|m| {
                m.is_collection_method()
                    .then_some(collection_client_method(&m, s))
            })
        })
        .collect();

    let resource_accessor_methods: Vec<TokenStream> = services
        .iter()
        .filter_map(generate_resource_accessor_method)
        .collect();

    let mod_paths: Vec<TokenStream> = services
        .iter()
        .map(|s| {
            let mod_path = s.models_path();
            quote! { use #mod_path::*; }
        })
        .collect();

    let codegen_imports: Vec<TokenStream> = services
        .iter()
        .map(|s| {
            let mod_name = format_ident!("{}", s.plan.base_path);
            let client_name = format_ident!("Py{}", s.client_type().to_string());
            quote! { use crate::codegen::#mod_name::#client_name; }
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
        #(#codegen_imports)*

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
                let base_url = base_url.parse().map_err(PyUnityCatalogError::from)?;
                Ok(Self { client: UnityCatalogClient::new(client, base_url) })
            }

            #(#methods)*

            #(#resource_accessor_methods)*
        }
    }
}

/// Generate Python method wrapper
fn resource_client_method(
    _service: &ServiceHandler<'_>,
    method: &MethodHandler<'_>,
) -> Option<TokenStream> {
    let method_name = method.plan.resource_client_method();
    let code = match &method.plan.request_type {
        RequestType::Get | RequestType::Update => {
            let response_type_ident = method.output_type().unwrap();
            generate_resource_method(response_type_ident, method.plan, method)
        }
        RequestType::Delete => generate_delete_method(method_name, method.plan, method),
        _ => return None,
    };
    Some(code)
}

fn collection_client_method(
    method: &MethodHandler<'_>,
    _service: &ServiceHandler<'_>,
) -> TokenStream {
    match &method.plan.request_type {
        RequestType::List => generate_list_method(method),
        RequestType::Create => generate_create_method(method),
        _ => quote! {},
    }
}

/// Generate List method (returns Vec<T> and uses try_collect)
fn generate_list_method(method: &MethodHandler<'_>) -> TokenStream {
    let method_name = method.plan.base_method_ident();

    let param_defs = generate_param_definitions(method.plan, method, true);
    let pyo3_signature = generate_pyo3_signature(method.plan, true);
    let client_call = generate_resource_client_call(method.plan, true);
    let builder_calls = generate_builder_pattern(method.plan, true);

    let response_type = extract_list_inner_type(&method.output_type().unwrap().to_string());
    let response_type = format_ident!("{}", response_type);

    quote! {
        #pyo3_signature
        pub fn #method_name(
            &self,
            py: Python,
            #(#param_defs,)*
        ) -> PyUnityCatalogResult<Vec<#response_type>> {
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
fn generate_create_method(method: &MethodHandler<'_>) -> TokenStream {
    let method_name = method.plan.base_method_ident();
    let response_type = method.output_type().unwrap();
    let param_defs = generate_param_definitions(method.plan, method, false);
    let pyo3_signature = generate_pyo3_signature_for_resource(method.plan);
    let client_call = generate_resource_client_call(method.plan, false);
    let builder_calls = generate_builder_pattern(method.plan, false);

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
fn generate_resource_method(
    response_type: syn::Ident,
    method_plan: &MethodPlan,
    method: &MethodHandler<'_>,
) -> TokenStream {
    let method_name = method_plan.resource_client_method();

    let param_defs = generate_resource_param_definitions(method_plan, method);
    let pyo3_signature = generate_pyo3_signature_for_resource(method_plan);
    let client_call = generate_resource_client_call(method_plan, false);
    let builder_calls = generate_builder_pattern(method_plan, false);

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
fn generate_delete_method(
    method_name: syn::Ident,
    method_plan: &MethodPlan,
    method: &MethodHandler<'_>,
) -> TokenStream {
    let param_defs = generate_resource_param_definitions(method_plan, method);
    let pyo3_signature = generate_pyo3_signature_for_resource(method_plan);
    let client_call = generate_resource_client_call(method_plan, false);
    let builder_calls = generate_builder_pattern(method_plan, false);

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
fn generate_resource_param_definitions(
    method_plan: &MethodPlan,
    method: &MethodHandler<'_>,
) -> Vec<TokenStream> {
    let mut params = Vec::new();

    // Add required body fields (non-optional)
    for body_field in method_plan.body_fields() {
        if !body_field.optional {
            let param_name = format_ident!("{}", body_field.name);
            let rust_type = convert_to_python_type(method, &body_field.rust_type, false);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    // Add required query parameters
    for query_param in method_plan.query_parameters() {
        if !query_param.optional {
            let param_name = format_ident!("{}", query_param.name);
            let rust_type = convert_to_python_type(method, &query_param.rust_type, true);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    // Add optional body fields
    for body_field in method_plan.body_fields() {
        if body_field.optional {
            let param_name = format_ident!("{}", body_field.name);
            let rust_type = convert_to_python_type(method, &body_field.rust_type, true);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    // Add optional query parameters
    for query_param in method_plan.query_parameters() {
        if query_param.optional {
            let param_name = format_ident!("{}", query_param.name);
            let rust_type = convert_to_python_type(method, &query_param.rust_type, true);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    params
}

/// Generate parameter definitions for method signature
fn generate_param_definitions(
    method_plan: &MethodPlan,
    method: &MethodHandler<'_>,
    is_list: bool,
) -> Vec<TokenStream> {
    let mut params = Vec::new();

    // Add required path parameters first (these don't have Option wrapper)
    for path_param in method_plan.path_parameters() {
        let param_name = format_ident!("{}", path_param.field_name);
        let rust_type = convert_to_python_type(method, &path_param.rust_type, false);
        params.push(quote! { #param_name: #rust_type });
    }

    // Add required body fields (non-optional)
    for body_field in method_plan.body_fields() {
        if !body_field.optional {
            let param_name = format_ident!("{}", body_field.name);
            // Use the original field type to get proper enum handling
            let python_type =
                get_python_type_from_method_field(method_plan, &body_field.name, false);
            let rust_type = convert_to_python_type(method, &python_type, false);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    // Add required query parameters (non-optional)
    for query_param in method_plan.query_parameters() {
        if !query_param.optional && !(is_list && query_param.name.as_str() == "page_token") {
            let param_name = format_ident!("{}", query_param.name);
            // Use the original field type to get proper enum handling
            let python_type =
                get_python_type_from_method_field(method_plan, &query_param.name, false);
            let rust_type = convert_to_python_type(method, &python_type, false);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    // Add optional query parameters
    for query_param in method_plan.query_parameters() {
        if query_param.optional && !(is_list && query_param.name.as_str() == "page_token") {
            let param_name = format_ident!("{}", query_param.name);
            // Use the original field type to get proper enum handling
            let python_type =
                get_python_type_from_method_field(method_plan, &query_param.name, true);
            let rust_type = convert_to_python_type(method, &python_type, true);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    // Add optional body fields
    for body_field in method_plan.body_fields() {
        if body_field.optional {
            let param_name = format_ident!("{}", body_field.name);
            // Use the original field type to get proper enum handling
            let python_type =
                get_python_type_from_method_field(method_plan, &body_field.name, true);
            let rust_type = convert_to_python_type(method, &python_type, true);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    params
}

/// Generate PyO3 signature annotation for collection methods (includes path parameters)
fn generate_pyo3_signature(method: &MethodPlan, is_list: bool) -> TokenStream {
    let mut signature_parts = Vec::new();

    // Add path parameters first
    for path_param in method.path_parameters() {
        signature_parts.push(path_param.field_name.clone());
    }

    // Required body fields (no default values)
    for body_field in method.body_fields() {
        if !body_field.optional {
            signature_parts.push(body_field.name.clone());
        }
    }

    // Required query parameters (no default values)
    for query_param in method.query_parameters() {
        if !query_param.optional && !(is_list && query_param.name == "page_token") {
            signature_parts.push(query_param.name.clone());
        }
    }

    // Optional body fields with defaults
    for body_field in method.body_fields() {
        if body_field.optional {
            signature_parts.push(format!("{} = None", body_field.name));
        }
    }

    // Optional query parameters with defaults - exclude page_token for list methods
    for query_param in method.query_parameters() {
        if query_param.optional && !(is_list && query_param.name == "page_token") {
            signature_parts.push(format!("{} = None", query_param.name));
        }
    }

    if signature_parts.is_empty() {
        quote! {}
    } else {
        let signature_string = signature_parts.join(", ");
        let signature_tokens = signature_string
            .parse::<proc_macro2::TokenStream>()
            .unwrap();
        quote! {
            #[pyo3(signature = (#signature_tokens))]
        }
    }
}

/// Generate PyO3 signature annotation for resource methods (excludes path parameters)
fn generate_pyo3_signature_for_resource(method: &MethodPlan) -> TokenStream {
    let mut signature_parts = Vec::new();

    // Don't add path parameters for resource methods - they're part of the client instance

    // Required body fields (no default values)
    for body_field in method.body_fields() {
        if !body_field.optional {
            signature_parts.push(body_field.name.clone());
        }
    }

    // Required query parameters (no default values)
    for query_param in method.query_parameters() {
        if !query_param.optional {
            signature_parts.push(query_param.name.clone());
        }
    }

    // Optional body fields with defaults
    for body_field in method.body_fields() {
        if body_field.optional {
            signature_parts.push(format!("{} = None", body_field.name));
        }
    }

    // Optional query parameters with defaults
    for query_param in method.query_parameters() {
        if query_param.optional {
            signature_parts.push(format!("{} = None", query_param.name));
        }
    }

    if signature_parts.is_empty() {
        quote! {}
    } else {
        let signature_string = signature_parts.join(", ");
        let signature_tokens = signature_string
            .parse::<proc_macro2::TokenStream>()
            .unwrap();
        quote! {
            #[pyo3(signature = (#signature_tokens))]
        }
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
    for body_field in method.body_fields() {
        if !body_field.optional {
            let param_name = format_ident!("{}", body_field.name);
            args.push(quote! { #param_name });
        }
    }

    // Add required query parameters
    for query_param in method.query_parameters() {
        if !query_param.optional {
            let param_name = format_ident!("{}", query_param.name);
            args.push(quote! { #param_name });
        }
    }

    quote! {
        self.client.#method_name(#(#args,)*)
    }
}

/// Generate builder pattern calls for Create/Update methods
fn generate_builder_pattern(method: &MethodPlan, is_list: bool) -> Vec<TokenStream> {
    let mut builder_calls = Vec::new();

    // Optional parameters become builder calls
    for query_param in method.query_parameters() {
        if query_param.optional && !(is_list && query_param.name == "page_token") {
            let param_name =
                format_ident!("{}", strings::operation_to_method_name(&query_param.name));
            let with_method = format_ident!("with_{}", query_param.name);
            builder_calls.push(quote! {
                request = request.#with_method(#param_name);
            });
        }
    }

    for body_field in method.body_fields() {
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

    builder_calls
}

/// Convert Rust type to Python-compatible type annotation using MethodHandler
fn convert_to_python_type(
    method: &MethodHandler<'_>,
    rust_type: &str,
    force_optional: bool,
) -> TokenStream {
    method.python_parameter_type(rust_type, force_optional)
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

/// Get the correct Python type for a method field, handling enums properly
fn get_python_type_from_method_field(
    method: &MethodPlan,
    field_name: &str,
    is_optional: bool,
) -> String {
    // Look for the field in the method metadata to get the original field type
    for field in &method.metadata.input_fields {
        if field.name == field_name {
            // Note: This should be refactored to use MethodHandler directly
            // For now, we'll create a temporary handler or use the existing logic
            return python_field_type_to_rust_type_fallback(&field.field_type, is_optional);
        }
    }

    // Fallback to existing logic if field not found
    if is_optional {
        format!("Option<{}>", "String")
    } else {
        "String".to_string()
    }
}

/// Temporary fallback function until we fully refactor to use MethodHandler
fn python_field_type_to_rust_type_fallback(field_type: &str, is_optional: bool) -> String {
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
