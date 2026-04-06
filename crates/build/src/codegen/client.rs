use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::{doc_tokens, extract_type_ident, format_tokens};
use crate::Result;
use crate::analysis::RequestType;
use crate::codegen::{MethodHandler, ServiceHandler};
use crate::google::api::http_rule::Pattern;
use crate::parsing::types::BaseType;

/// Generate client code for a service
pub(crate) fn generate(service: &ServiceHandler<'_>) -> Result<String> {
    let mut method_tokens = Vec::new();

    for method in service.methods() {
        let method_code = client_method(method);
        method_tokens.push(method_code);
    }

    let client_ident = service.client_type();
    let mod_path = service.models_path();
    let result_path: syn::Path =
        syn::parse_str(&service.config.result_type_path).expect("valid result_type_path");

    let tokens = quote! {
        #![allow(unused_mut)]
        // TODO: make configurable
        use cloud_client::CloudClient;
        use url::Url;
        use #result_path;
        use #mod_path::*;

        /// HTTP client for service operations
        #[derive(Clone)]
        pub struct #client_ident {
            pub(crate) client: CloudClient,
            pub(crate) base_url: Url,
        }

        impl #client_ident {
            /// Create a new client instance
            pub fn new(client: CloudClient, mut base_url: Url) -> Self {
                if !base_url.path().ends_with('/') {
                    base_url.set_path(&format!("{}/", base_url.path()));
                }
                Self { client, base_url }
            }

            #(#method_tokens)*
        }
    };

    Ok(format_tokens(tokens))
}

/// Generate client method implementation
pub fn client_method(method: MethodHandler<'_>) -> TokenStream {
    let doc_attrs = doc_tokens(method.plan.metadata.documentation.as_deref());
    let method_name = method.plan.base_method_ident();
    let input_type_ident = method.input_type();
    let http_method = format_ident!("{}", method.plan.http_method.to_lowercase());
    let url_formatting = generate_url_formatting(&method);
    let query_handling = generate_query_parameters(&method);

    let body_handling = if matches!(
        method.plan.request_type,
        RequestType::Create
            | RequestType::Update
            | RequestType::Custom(Pattern::Post(_))
            | RequestType::Custom(Pattern::Patch(_))
    ) {
        quote! { .json(request) }
    } else {
        quote! {}
    };

    if let Some(output_type) = method.output_type() {
        quote! {
            #doc_attrs
            pub async fn #method_name(&self, request: &#input_type_ident) -> Result<#output_type> {
                #url_formatting
                #query_handling
                let response = self.client.#http_method(url)#body_handling.send().await?;
                if !response.status().is_success() {
                    return Err(crate::error::parse_error_response(response).await);
                }
                let result = response.bytes().await?;
                Ok(serde_json::from_slice(&result)?)
            }
        }
    } else {
        quote! {
            #doc_attrs
            pub async fn #method_name(&self, request: &#input_type_ident) -> Result<()> {
                #url_formatting
                #query_handling
                let response = self.client.#http_method(url)#body_handling.send().await?;
                if !response.status().is_success() {
                    return Err(crate::error::parse_error_response(response).await);
                }
                Ok(())
            }
        }
    }
}

/// Generate URL formatting code that properly substitutes path parameters
fn generate_url_formatting(method: &MethodHandler<'_>) -> proc_macro2::TokenStream {
    let path = method.plan.http_pattern.base_path();
    let path = path.trim_start_matches('/');
    let params = method.plan.path_parameters().collect_vec();

    if params.is_empty() {
        return quote! {
            let mut url = self.base_url.join(#path)?;
        };
    }

    let (format_string, format_args) = method.plan.http_pattern.to_format_string();

    if format_args.is_empty() {
        quote! {
            let mut url = self.base_url.join(#path)?;
        }
    } else {
        let field_idents: Vec<_> = format_args
            .iter()
            .map(|template_param| format_ident!("{}", template_param))
            .collect();
        quote! {
            let formatted_path = format!(#format_string, #(request.#field_idents),*);
            let mut url = self.base_url.join(&formatted_path)?;
        }
    }
}

/// Generate query parameter handling code
fn generate_query_parameters(method: &MethodHandler<'_>) -> proc_macro2::TokenStream {
    let mut param_assignments = Vec::new();
    for param in method.plan.query_parameters() {
        let field_ident = format_ident!("{}", param.name);
        let param_name = &param.name;
        let field_type = &param.field_type;

        if field_type.is_repeated {
            if let BaseType::Enum(enum_name) = &field_type.base_type {
                let enum_type_ident = extract_type_ident(enum_name);
                param_assignments.push(quote! {
                    for &raw in &request.#field_ident {
                        if let Some(v) = #enum_type_ident::from_i32(raw) {
                            url.query_pairs_mut().append_pair(#param_name, v.as_str_name());
                        }
                    }
                });
            } else {
                param_assignments.push(quote! {
                    for value in &request.#field_ident {
                        url.query_pairs_mut().append_pair(#param_name, &value.to_string());
                    }
                });
            }
        } else if param.is_optional() {
            param_assignments.push(quote! {
                if let Some(ref value) = request.#field_ident {
                    url.query_pairs_mut().append_pair(#param_name, &value.to_string());
                }
            });
        } else {
            param_assignments.push(quote! {
                url.query_pairs_mut().append_pair(#param_name, &request.#field_ident.to_string());
            });
        }
    }

    if param_assignments.is_empty() {
        return quote! {};
    }

    quote! {
        #(#param_assignments)*
    }
}
