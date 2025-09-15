use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Path;

use super::format_tokens;
use crate::analysis::{PathParam, QueryParam, RequestType};
use crate::codegen::{MethodHandler, ServiceHandler};
use crate::google::api::http_rule::Pattern;
use crate::parsing::format_url_template;

/// Generate client code for a service
pub(crate) fn generate(service: &ServiceHandler<'_>) -> Result<String, Box<dyn std::error::Error>> {
    let mut client_methods = Vec::new();

    for method in service.methods() {
        let method_code = client_method(method);
        client_methods.push(method_code);
    }

    let client_code = client_struct(service, &client_methods);

    Ok(client_code)
}

/// Generate client struct definition
fn client_struct(service: &ServiceHandler<'_>, methods: &[String]) -> String {
    let client_ident = service.client_type();
    let method_tokens: Vec<TokenStream> = methods
        .iter()
        .map(|m| syn::parse_str::<TokenStream>(m).unwrap_or_else(|_| quote! {}))
        .collect();
    let mod_path: Path = service.models_path();

    let tokens = quote! {
        #![allow(unused_mut)]
        use cloud_client::CloudClient;
        use url::Url;
        use crate::error::Result;
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

    format_tokens(tokens)
}

/// Generate client method implementation
pub fn client_method(method: MethodHandler<'_>) -> String {
    let method_name = format_ident!("{}", method.plan.handler_function_name);
    let input_type_ident = method.input_type();
    let http_method = format_ident!("{}", method.plan.http_method.to_lowercase());
    let url_formatting = generate_url_formatting(&method.plan.http_path, &method.plan.path_params);
    let query_handling = generate_query_parameters(&method.plan.query_params);

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

    let tokens = if method.plan.has_response {
        let output_type = method.output_type();
        quote! {
            pub async fn #method_name(&self, request: &#input_type_ident) -> Result<#output_type> {
                #url_formatting
                #query_handling
                let response = self.client.#http_method(url)#body_handling.send().await?;
                response.error_for_status_ref()?;
                let result = response.bytes().await?;
                Ok(serde_json::from_slice(&result)?)
            }
        }
    } else {
        quote! {
            pub async fn #method_name(&self, request: &#input_type_ident) -> Result<()> {
                #url_formatting
                #query_handling
                let response = self.client.#http_method(url)#body_handling.send().await?;
                response.error_for_status()?;
                Ok(())
            }
        }
    };

    format_tokens(tokens)
}

/// Generate URL formatting code that properly substitutes path parameters
fn generate_url_formatting(path: &str, params: &[PathParam]) -> proc_macro2::TokenStream {
    let path = path.trim_start_matches('/');

    if params.is_empty() {
        return quote! {
            let mut url = self.base_url.join(#path)?;
        };
    }

    let template_param_names: Vec<String> =
        params.iter().map(|p| p.template_param.clone()).collect();
    let (format_string, format_args) = format_url_template(path, &template_param_names);

    if format_args.is_empty() {
        quote! {
            let mut url = self.base_url.join(#path)?;
        }
    } else {
        // Map template parameter names back to field names for request access
        let field_idents: Vec<_> = format_args
            .iter()
            .map(|template_param| {
                // Find the corresponding field name for this template parameter
                let field_name = params
                    .iter()
                    .find(|p| p.template_param == *template_param)
                    .map(|p| &p.field_name)
                    .unwrap_or(template_param);
                format_ident!("{}", field_name)
            })
            .collect();
        quote! {
            let formatted_path = format!(#format_string, #(request.#field_idents),*);
            let mut url = self.base_url.join(&formatted_path)?;
        }
    }
}

/// Generate query parameter handling code
fn generate_query_parameters(query_params: &[QueryParam]) -> proc_macro2::TokenStream {
    if query_params.is_empty() {
        return quote! {};
    }

    let mut param_assignments = Vec::new();

    for param in query_params {
        let field_ident = format_ident!("{}", param.name);
        let param_name = &param.name;

        if param.optional {
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

    quote! {
        #(#param_assignments)*
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_query_parameters() {
        // Test with no query parameters
        let empty_params = vec![];
        let result = generate_query_parameters(&empty_params);
        assert_eq!(result.to_string(), "");

        // Test with optional query parameters
        let params = vec![
            QueryParam {
                name: "max_results".to_string(),
                rust_type: "Option<i32>".to_string(),
                optional: true,
            },
            QueryParam {
                name: "page_token".to_string(),
                rust_type: "Option<String>".to_string(),
                optional: true,
            },
        ];
        let result = generate_query_parameters(&params);
        let code = result.to_string();
        assert!(code.contains("url . query_pairs_mut () . append_pair"));
        assert!(code.contains("if let Some (ref value) = request . max_results"));
        assert!(code.contains("if let Some (ref value) = request . page_token"));

        // Test with required query parameters
        let required_params = vec![QueryParam {
            name: "filter".to_string(),
            rust_type: "String".to_string(),
            optional: false,
        }];
        let result = generate_query_parameters(&required_params);
        let code = result.to_string();
        assert!(code.contains(
            "url . query_pairs_mut () . append_pair (\"filter\" , & request . filter . to_string ())"
        ));
    }
}
