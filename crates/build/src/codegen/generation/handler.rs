use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Path;

use super::{extract_type_ident, format_tokens};
use crate::analysis::{MethodPlan, ServicePlan};

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

        use crate::Result;
        use crate::api::RequestContext;
        use #mod_path::*;

        #[async_trait]
        pub trait #trait_ident: Send + Sync + 'static {
            #(#methods)*
        }
    };

    format_tokens(tokens)
}

/// Generate a single handler trait method
pub fn handler_trait_method(method: &MethodPlan) -> TokenStream {
    let input_type = extract_type_ident(&method.metadata.input_type);
    let method_name = format_ident!("{}", method.handler_function_name);

    if method.has_response {
        let output_type = extract_type_ident(&method.metadata.output_type);
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
