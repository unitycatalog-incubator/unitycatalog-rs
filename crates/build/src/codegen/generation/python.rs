//! Python bindings generation module
//!
//! This module generates Python bindings for Unity Catalog services using PyO3.
//! It creates wrapper structs that expose the Rust client functionality to Python
//! while handling async operations and error conversion.

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::format_tokens;
use crate::analysis::RequestType;
use crate::codegen::{MethodHandler, ServiceHandler};
use crate::parsing::RenderContext;
use crate::parsing::types::BaseType;
use crate::utils::strings;

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
    let client_ident = format_ident!("{}", format!("Py{}", service.client_type()));
    let rust_client_ident = service.client_type();
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

    let methods = services.iter().flat_map(|s| {
        s.methods().filter_map(|m| {
            m.is_collection_method()
                .then(|| collection_client_method(m))
                .flatten()
        })
    });

    let resource_accessor_methods = services
        .iter()
        .filter_map(|s| generate_resource_accessor_method(s));

    let mod_paths = services.iter().map(|s| {
        let mod_path = s.models_path();
        quote! { use #mod_path::*; }
    });

    let codegen_imports = services.iter().map(|s| {
        let mod_name = format_ident!("{}", s.plan.base_path);
        let client_name = format_ident!("Py{}", s.client_type().to_string());
        quote! { use crate::codegen::#mod_name::#client_name; }
    });

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
fn resource_client_method(method: MethodHandler<'_>) -> Option<TokenStream> {
    let code = match &method.plan.request_type {
        RequestType::Get | RequestType::Update => resource_get_update_method_impl(&method),
        RequestType::Delete => resource_delete_method_impl(&method),
        _ => return None,
    };
    Some(code)
}

fn collection_client_method(method: MethodHandler<'_>) -> Option<TokenStream> {
    match &method.plan.request_type {
        RequestType::List => Some(collection_list_method_impl(&method)),
        RequestType::Create => Some(collection_create_method_impl(&method)),
        _ => None,
    }
}

/// Generate List method (returns Vec<T> and uses try_collect)
fn collection_list_method_impl(method: &MethodHandler<'_>) -> TokenStream {
    let method_name = method.plan.base_method_ident();

    let param_defs = generate_param_definitions(method, true);
    let pyo3_signature = generate_pyo3_signature(method, true);
    let client_call = inner_resource_client_call(method, true);
    let builder_calls = generate_builder_pattern(method, true);

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
fn collection_create_method_impl(method: &MethodHandler<'_>) -> TokenStream {
    let method_name = method.plan.base_method_ident();
    let response_type = method.output_type().unwrap();
    let param_defs = generate_param_definitions(method, false);
    let pyo3_signature = generate_pyo3_signature_for_resource(method);
    let client_call = inner_resource_client_call(method, false);
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
    let param_defs = resource_method_parameters(method);
    let pyo3_signature = generate_pyo3_signature_for_resource(method);
    let client_call = inner_resource_client_call(method, false);
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
    let param_defs = resource_method_parameters(method);
    let pyo3_signature = generate_pyo3_signature_for_resource(method);
    let client_call = inner_resource_client_call(method, false);
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
fn resource_method_parameters(method: &MethodHandler<'_>) -> Vec<TokenStream> {
    let mut params = Vec::new();

    // Add required body fields (non-optional)
    for body_field in method.plan.body_fields() {
        if !body_field.optional {
            let param_name = format_ident!("{}", body_field.name);
            let rust_type =
                method.field_type(&body_field.field_type, RenderContext::PythonParameter);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    // Add required query parameters
    for query_param in method.plan.query_parameters() {
        if !query_param.optional {
            let param_name = format_ident!("{}", query_param.name);
            let rust_type =
                method.field_type(&query_param.field_type, RenderContext::PythonParameter);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    // Add optional body fields
    for body_field in method.plan.body_fields() {
        if body_field.optional {
            let param_name = format_ident!("{}", body_field.name);
            let rust_type =
                method.field_type(&body_field.field_type, RenderContext::PythonParameter);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    // Add optional query parameters
    for query_param in method.plan.query_parameters() {
        if query_param.optional {
            let param_name = format_ident!("{}", query_param.name);
            let rust_type =
                method.field_type(&query_param.field_type, RenderContext::PythonParameter);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    params
}

/// Generate parameter definitions for method signature
fn generate_param_definitions(method: &MethodHandler<'_>, is_list: bool) -> Vec<TokenStream> {
    let mut params = Vec::new();

    // Add required path parameters first (these don't have Option wrapper)
    for path_param in method.plan.path_parameters() {
        let param_name = format_ident!("{}", path_param.field_name);
        let rust_type = method.field_type(&path_param.field_type, RenderContext::PythonParameter);
        params.push(quote! { #param_name: #rust_type });
    }

    // Add required body fields (non-optional)
    for body_field in method.plan.body_fields() {
        if !body_field.optional {
            let param_name = format_ident!("{}", body_field.name);
            let rust_type =
                method.field_type(&body_field.field_type, RenderContext::PythonParameter);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    // Add required query parameters (non-optional)
    for query_param in method.plan.query_parameters() {
        if !query_param.optional && !(is_list && query_param.name.as_str() == "page_token") {
            let param_name = format_ident!("{}", query_param.name);
            let rust_type =
                method.field_type(&query_param.field_type, RenderContext::PythonParameter);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    // Add optional query parameters
    for query_param in method.plan.query_parameters() {
        if query_param.optional && !(is_list && query_param.name.as_str() == "page_token") {
            let param_name = format_ident!("{}", query_param.name);
            let rust_type =
                method.field_type(&query_param.field_type, RenderContext::PythonParameter);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    // Add optional body fields
    for body_field in method.plan.body_fields() {
        if body_field.optional {
            let param_name = format_ident!("{}", body_field.name);
            let rust_type =
                method.field_type(&body_field.field_type, RenderContext::PythonParameter);
            params.push(quote! { #param_name: #rust_type });
        }
    }

    params
}

/// Generate PyO3 signature annotation for collection methods (includes path parameters)
fn generate_pyo3_signature(method: &MethodHandler<'_>, is_list: bool) -> TokenStream {
    let mut signature_parts = Vec::new();

    // Add path parameters first
    for path_param in method.plan.path_parameters() {
        signature_parts.push(path_param.field_name.clone());
    }

    // Required body fields (no default values)
    for body_field in method.plan.body_fields() {
        if !body_field.optional {
            signature_parts.push(body_field.name.clone());
        }
    }

    // Required query parameters (no default values)
    for query_param in method.plan.query_parameters() {
        if !query_param.optional && !(is_list && query_param.name == "page_token") {
            signature_parts.push(query_param.name.clone());
        }
    }

    // Optional body fields with defaults
    for body_field in method.plan.body_fields() {
        if body_field.optional {
            signature_parts.push(format!("{} = None", body_field.name));
        }
    }

    // Optional query parameters with defaults - exclude page_token for list methods
    for query_param in method.plan.query_parameters() {
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
fn generate_pyo3_signature_for_resource(method: &MethodHandler<'_>) -> TokenStream {
    let mut signature_parts = Vec::new();

    // Don't add path parameters for resource methods - they're part of the client instance

    // Required body fields (no default values)
    for body_field in method.plan.body_fields() {
        if !body_field.optional {
            signature_parts.push(body_field.name.clone());
        }
    }

    // Required query parameters (no default values)
    for query_param in method.plan.query_parameters() {
        if !query_param.optional {
            signature_parts.push(query_param.name.clone());
        }
    }

    // Optional body fields with defaults
    for body_field in method.plan.body_fields() {
        if body_field.optional {
            signature_parts.push(format!("{} = None", body_field.name));
        }
    }

    // Optional query parameters with defaults
    for query_param in method.plan.query_parameters() {
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
fn inner_resource_client_call(method: &MethodHandler<'_>, _is_list: bool) -> TokenStream {
    let method_name = method.plan.resource_client_method();
    let mut args = Vec::new();

    // Add required body fields as direct arguments
    for body_field in method.plan.body_fields() {
        if !body_field.optional {
            let param_name = format_ident!("{}", body_field.name);
            args.push(quote! { #param_name });
        }
    }

    // Add required query parameters
    for query_param in method.plan.query_parameters() {
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
fn generate_builder_pattern(method: &MethodHandler<'_>, is_list: bool) -> Vec<TokenStream> {
    let mut builder_calls = Vec::new();

    // Optional parameters become builder calls
    for query_param in method.plan.query_parameters() {
        if query_param.optional && !(is_list && query_param.name == "page_token") {
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
