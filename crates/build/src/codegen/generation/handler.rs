use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::{extract_type_ident, format_tokens};
use crate::{analysis::MethodPlan, codegen::ServiceHandler};

/// Generate handler trait for a service
pub(super) fn generate(service: &ServiceHandler<'_>) -> Result<String, Box<dyn std::error::Error>> {
    let mut trait_methods = Vec::new();
    for method in &service.plan.methods {
        let method_code = handler_trait_method(method);
        trait_methods.push(method_code);
    }

    let trait_code = handler_trait(service, &service.plan.handler_name, &trait_methods);

    Ok(trait_code)
}

/// Generate handler trait definition
pub fn handler_trait(
    service: &ServiceHandler<'_>,
    trait_name: &str,
    methods: &[TokenStream],
) -> String {
    let trait_ident = format_ident!("{}", trait_name);
    let mod_path = service.models_path();

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
