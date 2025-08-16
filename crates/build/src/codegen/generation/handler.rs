use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Path;

use crate::codegen::MethodPlan;

use super::{ServicePlan, templates};

/// Generate handler trait for a service
pub(super) fn generate(service: &ServicePlan) -> Result<String, Box<dyn std::error::Error>> {
    let mut trait_methods = Vec::new();

    for method in &service.methods {
        let method_code = handler_trait_method(method);
        trait_methods.push(method_code);
    }

    let trait_code = handler_trait(
        &service.handler_name,
        &trait_methods,
        service.base_path.clone(),
    );

    Ok(trait_code)
}

/// Generate handler trait definition
pub fn handler_trait(trait_name: &str, methods: &[TokenStream], service_base: String) -> String {
    let trait_ident = format_ident!("{}", trait_name);
    let mod_path: Path = syn::parse_str(&format!(
        "unitycatalog_common::models::{}::v1",
        service_base
    ))
    .unwrap();

    let tokens = quote! {
        use async_trait::async_trait;

        use unitycatalog_common::Result;
        use crate::api::RequestContext;
        use #mod_path::*;

        #[async_trait]
        pub trait #trait_ident: Send + Sync + 'static {
            #(#methods)*
        }
    };

    templates::format_tokens(tokens)
}

/// Generate a single handler trait method
pub fn handler_trait_method(method: &MethodPlan) -> TokenStream {
    let input_type = templates::extract_type_ident(&method.metadata.input_type);
    let method_name = format_ident!("{}", method.handler_function_name);

    if method.has_response {
        let output_type = templates::extract_type_ident(&method.metadata.output_type);
        quote! {
            async fn #method_name(
                &self,
                request: #input_type,
                context: RequestContext,
            ) -> Result<#output_type>;
        }
    } else {
        quote! {
            async fn #method_name(
                &self,
                request: #input_type,
                context: RequestContext,
            ) -> Result<()>;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::create_test_service_plan;
    use super::*;

    #[test]
    fn test_generate_handler_trait() {
        let service = create_test_service_plan();
        let result = generate(&service);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("CatalogHandler"));
        assert!(code.contains("list_catalogs"));
    }

    #[test]
    fn test_generated_code_format() {
        let service = create_test_service_plan();
        let result = generate(&service);
        assert!(result.is_ok());
        let code = result.unwrap();

        // Print generated code to verify format
        println!("Generated handler trait:\n{}", code);

        // Verify the code contains expected elements
        assert!(code.contains("pub trait CatalogHandler"));
        assert!(code.contains("async fn list_catalogs"));
        assert!(code.contains("RequestContext"));
        assert!(code.contains("async_trait"));

        // Verify proper Rust syntax (no extra escaping or formatting issues)
        assert!(!code.contains("\\n"));
        assert!(!code.contains("\\t"));
        assert!(!code.contains("\\\""));
    }
}
