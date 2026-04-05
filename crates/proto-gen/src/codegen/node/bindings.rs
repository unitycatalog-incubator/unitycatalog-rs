//! NAPI-RS binding generation for Unity Catalog services.

use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::super::format_tokens;
use super::super::python::derive_resource_accessor_params;
use crate::analysis::{RequestParam, RequestType};
use crate::codegen::{MethodHandler, ServiceHandler};
use crate::parsing::types::{BaseType, RenderContext};
use crate::utils::strings;

/// Check if a parameter type is supported across the NAPI boundary.
///
/// NAPI-RS supports: primitives, String, bool, Buffer, HashMap<String, String>,
/// Vec<T> of supported types, Option<T> of supported types.
/// Enums are supported as i32 values. Complex messages/oneofs are not.
fn is_napi_supported(param: &RequestParam) -> bool {
    is_napi_supported_type(&param.field_type().base_type)
}

fn is_napi_supported_type(base_type: &BaseType) -> bool {
    match base_type {
        BaseType::String
        | BaseType::Int32
        | BaseType::Int64
        | BaseType::Bool
        | BaseType::Float32
        | BaseType::Float64
        | BaseType::Bytes
        | BaseType::Unit
        | BaseType::Enum(_) => true,
        BaseType::Map(k, v) => {
            is_napi_supported_type(&k.base_type) && is_napi_supported_type(&v.base_type)
        }
        BaseType::Message(_) | BaseType::OneOf(_) => false,
    }
}

pub fn main_module(services: &[ServiceHandler<'_>]) -> String {
    let service_modules = services.iter().map(|s| {
        let module_name = format_ident!("{}", s.plan.base_path);
        quote! { pub mod #module_name; }
    });
    let uc_client_module = collection_client_struct(services);

    let tokens = quote! {
        #![allow(unused_mut, unused_imports, dead_code, clippy::all)]
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
        .expect("bindings config required for node output");

    let rust_client_ident = service.client_type();
    let napi_client_ident = format_ident!("Napi{}", rust_client_ident);
    let _napi_client_name = rust_client_ident.to_string();

    let client_crate = format_ident!("{}", bindings.aggregate_client_name.to_case(Case::Snake));
    let napi_error_ext_ident = format_ident!("{}", bindings.napi_error_ext_trait);

    let methods = service.methods().filter_map(resource_client_method);
    let mod_path = service.models_path();

    let tokens = quote! {
        #![allow(unused_mut, unused_imports, dead_code, clippy::all)]
        use std::collections::HashMap;
        use napi::bindgen_prelude::Buffer;
        use napi_derive::napi;
        use prost::Message;
        use #client_crate::#rust_client_ident;
        use #mod_path::*;
        use crate::error::#napi_error_ext_ident;

        #[napi]
        pub struct #napi_client_ident {
            pub(crate) client: #rust_client_ident,
        }

        #[napi]
        impl #napi_client_ident {
            #(#methods)*
        }

        impl #napi_client_ident {
            pub fn new(client: #rust_client_ident) -> Self {
                Self { client }
            }
        }
    };

    format_tokens(tokens)
}

fn collection_client_struct(services: &[ServiceHandler<'_>]) -> TokenStream {
    let bindings = services
        .first()
        .and_then(|s| s.config.bindings.as_ref())
        .expect("bindings config required for node output");

    let aggregate_client_name = &bindings.aggregate_client_name;
    let client_crate = format_ident!("{}", aggregate_client_name.to_case(Case::Snake));
    let aggregate_client_ident = format_ident!("{}", aggregate_client_name);
    let napi_aggregate_client_ident = format_ident!("Napi{}", aggregate_client_name);
    let napi_error_ext_ident = format_ident!("{}", bindings.napi_error_ext_trait);

    let mut sorted_services = services.iter().collect_vec();
    sorted_services.sort_by(|a, b| a.plan.service_name.cmp(&b.plan.service_name));

    let mod_paths = sorted_services.iter().map(|s| {
        let mod_path = s.models_path();
        quote! { use #mod_path::*; }
    });

    let codegen_imports = sorted_services.iter().map(|s| {
        let mod_name = format_ident!("{}", s.plan.base_path);
        let client_name = format_ident!("Napi{}", s.client_type().to_string());
        quote! { use crate::codegen::#mod_name::#client_name; }
    });

    let methods = sorted_services
        .iter()
        .flat_map(|s| s.methods().filter_map(collection_client_method));

    let resource_accessor_methods = sorted_services
        .iter()
        .filter_map(|s| generate_resource_accessor_method(s));

    quote! {
        use std::collections::HashMap;
        use futures::stream::TryStreamExt;
        use napi::bindgen_prelude::Buffer;
        use napi_derive::napi;
        use prost::Message;
        use #client_crate::#aggregate_client_ident;
        use crate::error::#napi_error_ext_ident;
        #(#mod_paths)*
        #(#codegen_imports)*

        #[napi]
        pub struct #napi_aggregate_client_ident {
            client: #aggregate_client_ident
        }

        #[napi]
        impl #napi_aggregate_client_ident {
            #[napi(factory)]
            pub fn from_url(base_url: String, token: Option<String>) -> napi::Result<Self> {
                let client = if let Some(token) = token {
                    cloud_client::CloudClient::new_with_token(token)
                } else {
                    cloud_client::CloudClient::new_unauthenticated()
                };
                let base_url = base_url.parse().map_err(|e: url::ParseError| {
                    napi::Error::from_reason(e.to_string())
                })?;
                Ok(Self { client: #aggregate_client_ident::new(client, base_url) })
            }

            #(#methods)*

            #(#resource_accessor_methods)*
        }
    }
}

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

fn collection_list_method_impl(method: &MethodHandler<'_>) -> TokenStream {
    let method_name = method.plan.base_method_ident();

    let param_defs = collection_method_parameters(method, true);
    let client_call = inner_resource_client_call(method);
    let builder_calls = generate_builder_pattern(method, true);

    quote! {
        #[napi(catch_unwind)]
        pub async fn #method_name(
            &self,
            #(#param_defs,)*
        ) -> napi::Result<Vec<Buffer>> {
            let mut request = #client_call;
            #(#builder_calls)*
            request
                .into_stream()
                .map_ok(|item| Buffer::from(item.encode_to_vec()))
                .try_collect::<Vec<_>>()
                .await
                .default_error()
        }
    }
}

fn collection_create_method_impl(method: &MethodHandler<'_>) -> TokenStream {
    let method_name = method.plan.base_method_ident();
    let has_response = method.output_type().is_some();
    let param_defs = collection_method_parameters(method, false);
    let client_call = inner_resource_client_call(method);
    let builder_calls = generate_builder_pattern(method, false);

    if has_response {
        quote! {
            #[napi(catch_unwind)]
            pub async fn #method_name(
                &self,
                #(#param_defs,)*
            ) -> napi::Result<Buffer> {
                let mut request = #client_call;
                #(#builder_calls)*
                request
                    .await
                    .map(|item| Buffer::from(item.encode_to_vec()))
                    .default_error()
            }
        }
    } else {
        quote! {
            #[napi(catch_unwind)]
            pub async fn #method_name(
                &self,
                #(#param_defs,)*
            ) -> napi::Result<()> {
                let mut request = #client_call;
                #(#builder_calls)*
                request
                    .await
                    .default_error()
            }
        }
    }
}

fn resource_get_update_method_impl(method: &MethodHandler<'_>) -> TokenStream {
    let method_name = method.plan.resource_client_method();
    let param_defs = resource_method_parameters(method);
    let client_call = inner_resource_client_call(method);
    let builder_calls = generate_builder_pattern(method, false);

    quote! {
        #[napi(catch_unwind)]
        pub async fn #method_name(
            &self,
            #(#param_defs,)*
        ) -> napi::Result<Buffer> {
            let mut request = #client_call;
            #(#builder_calls)*
            request
                .await
                .map(|item| Buffer::from(item.encode_to_vec()))
                .default_error()
        }
    }
}

fn resource_delete_method_impl(method: &MethodHandler<'_>) -> TokenStream {
    let method_name = method.plan.resource_client_method();
    let param_defs = resource_method_parameters(method);
    let client_call = inner_resource_client_call(method);
    let builder_calls = generate_builder_pattern(method, false);

    quote! {
        #[napi(catch_unwind)]
        pub async fn #method_name(
            &self,
            #(#param_defs,)*
        ) -> napi::Result<()> {
            let mut request = #client_call;
            #(#builder_calls)*
            request
                .await
                .default_error()
        }
    }
}

fn resource_method_parameters(method: &MethodHandler<'_>) -> Vec<TokenStream> {
    method
        .required_parameters()
        .chain(method.optional_parameters())
        .filter(|field| !field.is_path_param() && is_napi_supported(field))
        .map(|p| {
            let param_name = p.field_ident();
            let rust_type = method.field_type(p.field_type(), RenderContext::NapiParameter);
            quote! { #param_name: #rust_type }
        })
        .collect()
}

fn collection_method_parameters(method: &MethodHandler<'_>, is_list: bool) -> Vec<TokenStream> {
    method
        .required_parameters()
        .chain(method.optional_parameters())
        .filter(|p| !(is_list && p.name() == "page_token") && is_napi_supported(p))
        .map(|p| {
            let param_name = p.field_ident();
            let rust_type = method.field_type(p.field_type(), RenderContext::NapiParameter);
            quote! { #param_name: #rust_type }
        })
        .collect()
}

fn inner_resource_client_call(method: &MethodHandler<'_>) -> TokenStream {
    let method_name = method.plan.resource_client_method();
    let args = method
        .required_parameters()
        .filter(|param| !param.is_path_param() && is_napi_supported(param))
        .map(|param| {
            let param_name = param.field_ident();
            if matches!(param.field_type().base_type, BaseType::Enum(..)) {
                quote! { #param_name.try_into().map_err(|_| napi::Error::from_reason("invalid enum value"))? }
            } else {
                quote! { #param_name }
            }
        });
    quote! {
        self.client.#method_name(#(#args,)*)
    }
}

fn generate_builder_pattern(method: &MethodHandler<'_>, is_list: bool) -> Vec<TokenStream> {
    let mut builder_calls = Vec::new();

    for query_param in method.plan.query_parameters() {
        if query_param.is_optional()
            && !(is_list && query_param.name == "page_token")
            && is_napi_supported_type(&query_param.field_type.base_type)
        {
            let param_name =
                format_ident!("{}", strings::operation_to_method_name(&query_param.name));
            let with_method = format_ident!("with_{}", query_param.name);

            if matches!(query_param.field_type.base_type, BaseType::Enum(_)) {
                builder_calls.push(quote! {
                    request = request.#with_method(
                        #param_name.map(|v| v.try_into().ok()).flatten()
                    );
                });
            } else {
                builder_calls.push(quote! {
                    request = request.#with_method(#param_name);
                });
            }
        }
    }

    for body_field in method.plan.body_fields() {
        if body_field.is_optional() && is_napi_supported_type(&body_field.field_type.base_type) {
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
            } else if matches!(body_field.field_type.base_type, BaseType::Enum(_)) {
                builder_calls.push(quote! {
                    request = request.#with_method(
                        #param_name.map(|v| v.try_into().ok()).flatten()
                    );
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
    let client_name = format_ident!("Napi{}", service.client_type().to_string());

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
        #[napi]
        pub fn #method_name(&self, #(#param_list),*) -> #client_name {
            #client_name {
                client: self.client.#method_name(#(#param_refs),*),
            }
        }
    };

    Some(method_call)
}
