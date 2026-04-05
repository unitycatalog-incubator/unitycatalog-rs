//! PyO3 binding generation for protobuf-defined services.

use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use super::super::format_tokens;
use super::derive_resource_accessor_params;
use crate::analysis::{RequestParam, RequestType};
use crate::codegen::{MethodHandler, ServiceHandler};
use crate::parsing::types::{BaseType, RenderContext};
use crate::utils::strings;

pub fn main_module(services: &[ServiceHandler<'_>]) -> String {
    let service_modules = services.iter().map(|s| {
        let module_name = format_ident!("{}", s.plan.base_path);
        quote! { pub mod #module_name; }
    });
    let uc_client_module = collection_client_struct(services);

    let tokens = quote! {
        #(#service_modules)*

        #uc_client_module
    };

    format_tokens(tokens)
}

pub(crate) fn generate(service: &ServiceHandler<'_>) -> String {
    let bindings = service
        .config
        .bindings
        .as_ref()
        .expect("bindings config required for python output");

    let rust_client_ident = service.client_type();
    let client_ident = format_ident!("{}", format!("Py{}", rust_client_ident));
    let rust_client_name = rust_client_ident.to_string();

    let client_crate = format_ident!("{}", bindings.client_crate_name);

    let py_error_type = format_ident!("{}", bindings.py_error_type);
    let py_result_type = format_ident!("{}", bindings.py_result_type);

    let methods = service
        .methods()
        .filter_map(|m| resource_client_method(m, &py_error_type, &py_result_type));
    let mod_path = service.models_path();

    let tokens = quote! {
        use std::collections::HashMap;
        use pyo3::prelude::*;
        use #client_crate::#rust_client_ident;
        use #mod_path::*;
        use crate::error::{#py_error_type, #py_result_type};
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
    // All services must share the same config; read bindings from the first service.
    let bindings = services
        .first()
        .and_then(|s| s.config.bindings.as_ref())
        .expect("bindings config required for python output");

    let aggregate_client_name = &bindings.aggregate_client_name;
    let client_crate = format_ident!("{}", bindings.client_crate_name);
    let aggregate_client_ident = format_ident!("{}", aggregate_client_name);
    let py_aggregate_client_ident = format_ident!("Py{}", aggregate_client_name);
    let py_error_type = format_ident!("{}", bindings.py_error_type);
    let py_result_type = format_ident!("{}", bindings.py_result_type);

    let mut sorted_services = services.iter().collect_vec();
    sorted_services.sort_by(|a, b| a.plan.service_name.cmp(&b.plan.service_name));

    let mod_paths = sorted_services.iter().map(|s| {
        let mod_path = s.models_path();
        quote! { use #mod_path::*; }
    });

    let codegen_imports = sorted_services.iter().map(|s| {
        let mod_name = format_ident!("{}", s.plan.base_path);
        let client_name = format_ident!("Py{}", s.client_type().to_string());
        quote! { use crate::codegen::#mod_name::#client_name; }
    });

    let methods = sorted_services.iter().flat_map(|s| {
        s.methods()
            .filter_map(|m| collection_client_method(m, &py_error_type, &py_result_type))
    });

    let resource_accessor_methods = sorted_services
        .iter()
        .filter_map(|s| generate_resource_accessor_method(s));

    quote! {
        use std::collections::HashMap;
        use futures::stream::TryStreamExt;
        use pyo3::prelude::*;
        use #client_crate::{#aggregate_client_ident};
        use crate::error::{#py_error_type, #py_result_type};
        use crate::runtime::get_runtime;
        #(#mod_paths)*
        #(#codegen_imports)*

        #[pyclass(name = #aggregate_client_name)]
        pub struct #py_aggregate_client_ident {
            client: #aggregate_client_ident
        }

        #[pymethods]
        impl #py_aggregate_client_ident {
            #[new]
            #[pyo3(signature = (base_url, token = None))]
            pub fn new(base_url: String, token: Option<String>) -> PyResult<Self> {
                let client = if let Some(token) = token {
                    cloud_client::CloudClient::new_with_token(token)
                } else {
                    cloud_client::CloudClient::new_unauthenticated()
                };
                let base_url = base_url.parse().map_err(#py_error_type::from)?;
                Ok(Self { client: #aggregate_client_ident::new(client, base_url) })
            }

            #(#methods)*

            #(#resource_accessor_methods)*
        }
    }
}

fn resource_client_method(
    method: MethodHandler<'_>,
    py_error_type: &Ident,
    py_result_type: &Ident,
) -> Option<TokenStream> {
    let code = match &method.plan.request_type {
        RequestType::Get | RequestType::Update => {
            resource_get_update_method_impl(&method, py_error_type, py_result_type)
        }
        RequestType::Delete => resource_delete_method_impl(&method, py_error_type, py_result_type),
        _ => return None,
    };
    Some(code)
}

fn collection_client_method(
    method: MethodHandler<'_>,
    py_error_type: &Ident,
    py_result_type: &Ident,
) -> Option<TokenStream> {
    if !method.is_collection_method() {
        return None;
    }
    match &method.plan.request_type {
        RequestType::List => Some(collection_list_method_impl(
            &method,
            py_error_type,
            py_result_type,
        )),
        RequestType::Create => Some(collection_create_method_impl(
            &method,
            py_error_type,
            py_result_type,
        )),
        _ => None,
    }
}

fn collection_list_method_impl(
    method: &MethodHandler<'_>,
    py_error_type: &Ident,
    py_result_type: &Ident,
) -> TokenStream {
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
        ) -> #py_result_type<#response_type> {
            let mut request = #client_call;
            #(#builder_calls)*
            let runtime = get_runtime(py)?;
            py.allow_threads(|| {
                let result = runtime.block_on(async move { request.into_stream().try_collect().await })?;
                Ok::<_, #py_error_type>(result)
            })
        }
    }
}

fn collection_create_method_impl(
    method: &MethodHandler<'_>,
    py_error_type: &Ident,
    py_result_type: &Ident,
) -> TokenStream {
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
        ) -> #py_result_type<#response_type> {
            let mut request = #client_call;
            #(#builder_calls)*
            let runtime = get_runtime(py)?;
            py.allow_threads(|| {
                let result = runtime.block_on(request.into_future())?;
                Ok::<_, #py_error_type>(result)
            })
        }
    }
}

fn resource_get_update_method_impl(
    method: &MethodHandler<'_>,
    py_error_type: &Ident,
    py_result_type: &Ident,
) -> TokenStream {
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
        ) -> #py_result_type<#response_type> {
            let mut request = #client_call;
            #(#builder_calls)*
            let runtime = get_runtime(py)?;
            py.allow_threads(|| {
                let result = runtime.block_on(request.into_future())?;
                Ok::<_, #py_error_type>(result)
            })
        }
    }
}

fn resource_delete_method_impl(
    method: &MethodHandler<'_>,
    py_error_type: &Ident,
    py_result_type: &Ident,
) -> TokenStream {
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
        ) -> #py_result_type<()> {
            let mut request = #client_call;
            #(#builder_calls)*
            let runtime = get_runtime(py)?;
            py.allow_threads(|| {
                runtime.block_on(request.into_future())?;
                Ok::<_, #py_error_type>(())
            })
        }
    }
}

fn resource_method_parameters(method: &MethodHandler<'_>) -> (Vec<TokenStream>, TokenStream) {
    let map_param = |p: &RequestParam| {
        let param_name = p.field_ident();
        let rust_type = method.field_type(p.field_type(), RenderContext::PythonParameter);
        quote! { #param_name: #rust_type }
    };
    let parameters = method
        .required_parameters()
        .chain(method.optional_parameters())
        .filter(|field| !field.is_path_param())
        .collect_vec();
    let signature = render_pyo3(&parameters);
    (parameters.into_iter().map(map_param).collect(), signature)
}

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
        if body_field.is_optional() {
            let param_name =
                format_ident!("{}", strings::operation_to_method_name(&body_field.name));
            let with_method = format_ident!("with_{}", body_field.name);

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

fn generate_resource_accessor_method(service: &ServiceHandler<'_>) -> Option<TokenStream> {
    if service.plan.managed_resources.is_empty() {
        return None;
    }

    let resource = service.resource().unwrap();
    let method_name = format_ident!("{}", resource.descriptor.singular);
    let client_name = format_ident!("Py{}", service.client_type().to_string());

    let params = derive_resource_accessor_params(service);

    let param_idents: Vec<_> = params.iter().map(|p| format_ident!("{}", p)).collect();
    let param_list = param_idents
        .iter()
        .map(|id| quote! { #id: String })
        .collect::<Vec<_>>();
    let param_refs = param_idents
        .iter()
        .map(|id| quote! { #id })
        .collect::<Vec<_>>();

    let method_call = quote! {
        pub fn #method_name(&self, #(#param_list),*) -> #client_name {
            #client_name {
                client: self.client.#method_name(#(#param_refs),*),
            }
        }
    };

    Some(method_call)
}
