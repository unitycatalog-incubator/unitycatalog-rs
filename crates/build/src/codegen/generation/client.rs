use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Path;

use super::super::{MethodPlan, PathParam, QueryParam, ServicePlan, templates};
use crate::RequestType;
use crate::utils::strings;

/// Generate client code for a service
pub(crate) fn generate(service: &ServicePlan) -> Result<String, Box<dyn std::error::Error>> {
    let mut client_methods = Vec::new();

    for method in &service.methods {
        let method_code = client_method(method);
        client_methods.push(method_code);
    }

    let client_name = format!(
        "{}Client",
        service
            .handler_name
            .strip_suffix("Handler")
            .unwrap_or(&service.handler_name)
    );
    let client_code = client_struct(&client_name, &client_methods, &service.base_path);

    Ok(client_code)
}

/// Generate client struct definition
fn client_struct(client_name: &str, methods: &[String], service_namespace: &str) -> String {
    let client_ident = format_ident!("{}", client_name);
    let method_tokens: Vec<TokenStream> = methods
        .iter()
        .map(|m| syn::parse_str::<TokenStream>(m).unwrap_or_else(|_| quote! {}))
        .collect();
    let mod_path: Path =
        syn::parse_str(&format!("crate::models::{}::v1", service_namespace)).unwrap();

    let tokens = quote! {
        #![allow(unused_mut)]
        use cloud_client::CloudClient;
        use url::Url;
        use crate::Result;
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

    templates::format_tokens(tokens)
}

/// Generate client method implementation
pub fn client_method(method: &MethodPlan) -> String {
    let method_name = format_ident!("{}", method.handler_function_name);
    let input_type = strings::extract_simple_type_name(&method.metadata.input_type);
    let input_type_ident = format_ident!("{}", input_type);
    let http_method = format_ident!("{}", method.http_method.to_lowercase());
    let url_formatting = generate_url_formatting(&method.http_path, &method.path_params);
    let query_handling = generate_query_parameters(&method.query_params);

    let body_handling = if matches!(
        method.metadata.request_type(),
        RequestType::Create | RequestType::Update
    ) {
        quote! { .json(request) }
    } else {
        quote! {}
    };

    let tokens = if method.has_response {
        let output_type = strings::extract_simple_type_name(&method.metadata.output_type);
        let output_type_ident = format_ident!("{}", output_type);
        quote! {
            pub async fn #method_name(&self, request: &#input_type_ident) -> Result<#output_type_ident> {
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

    templates::format_tokens(tokens)
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
    let (format_string, format_args) =
        crate::utils::paths::format_url_template(path, &template_param_names);

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
    use super::super::tests::create_test_service_plan;
    use super::*;

    #[test]
    fn test_generate_client_code() {
        let service = create_test_service_plan();
        let result = generate(&service);
        assert!(result.is_ok());
        let code = result.unwrap();

        // Print generated client code to verify format
        println!("Generated client code:\n{}", code);

        // Verify the code contains expected elements
        assert!(code.contains("pub struct CatalogClient"));
        assert!(code.contains("pub async fn list_catalogs"));
        assert!(code.contains("CloudClient"));
        assert!(code.contains("impl CatalogClient"));

        // Verify proper Rust syntax
        assert!(!code.contains("\\n"));
        assert!(!code.contains("\\t"));
        assert!(!code.contains("\\\""));
    }

    #[test]
    fn test_generate_query_parameters() {
        use crate::codegen::QueryParam;

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
