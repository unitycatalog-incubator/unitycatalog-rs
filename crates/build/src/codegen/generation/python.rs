//! Python bindings generation module
//!
//! This module generates Python bindings for Unity Catalog services using PyO3.
//! It creates wrapper structs that expose the Rust client functionality to Python
//! while handling async operations and error conversion.

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::format_tokens;
use crate::analysis::{RequestParam, RequestType};
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
