use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Path;

use super::super::{MethodPlan, PathParam, ServicePlan, templates};
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

    println!(
        "cargo:warning=Generated client {} with {} methods",
        client_name,
        service.methods.len()
    );

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
        use cloud_client::CloudClient;
        use url::Url;

        use #mod_path::*;

        /// HTTP client for service operations
        #[derive(Clone)]
        pub struct #client_ident {
            pub(crate) client: CloudClient,
            pub(crate) base_url: Url,
        }

        impl #client_ident {
            /// Create a new client instance
            pub fn new(client: CloudClient, base_url: Url) -> Self {
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
            pub async fn #method_name(&self, request: &#input_type_ident) -> crate::Result<#output_type_ident> {
                #url_formatting
                let response = self.client.#http_method(url)#body_handling.send().await?;
                response.error_for_status_ref()?;
                let result = response.bytes().await?;
                Ok(serde_json::from_slice(&result)?)
            }
        }
    } else {
        quote! {
            pub async fn #method_name(&self, request: &#input_type_ident) -> crate::Result<()> {
                #url_formatting
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
    if params.is_empty() {
        return quote! {
            let url = self.base_url.join(#path)?;
        };
    }

    let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
    let (format_string, format_args) = crate::utils::paths::format_url_template(path, &param_names);

    if format_args.is_empty() {
        quote! {
            let url = self.base_url.join(#path)?;
        }
    } else {
        let field_idents: Vec<_> = format_args
            .iter()
            .map(|arg| format_ident!("{}", arg))
            .collect();
        quote! {
            let formatted_path = format!(#format_string, #(request.#field_idents),*);
            let url = self.base_url.join(&formatted_path)?;
        }
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
}
