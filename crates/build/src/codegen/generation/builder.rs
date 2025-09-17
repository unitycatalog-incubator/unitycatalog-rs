use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Path;

use super::format_tokens;
use crate::analysis::RequestType;
use crate::codegen::{MethodHandler, ServiceHandler, extract_type_ident};
use crate::parsing::types::BaseType;
use crate::parsing::{MessageField, RenderContext};

/// Generate builder code for all request types in a service
pub(crate) fn generate(service: &ServiceHandler<'_>) -> Result<String, Box<dyn std::error::Error>> {
    let builder_impls: Vec<_> = service
        .methods()
        .map(|method| generate_request_builder(method, service))
        .try_collect()?;

    if builder_impls.is_empty() {
        return Ok(String::new());
    }

    let builder_code = generate_builders_module(service, &builder_impls);

    Ok(builder_code)
}

/// Generate the complete builders module
fn generate_builders_module(service: &ServiceHandler<'_>, builders: &[String]) -> String {
    let builder_tokens: Vec<TokenStream> = builders
        .iter()
        .map(|b| syn::parse_str::<TokenStream>(b).unwrap_or_else(|_| quote! {}))
        .collect();
    let mod_path: Path = service.models_path();

    let tokens = quote! {
        #![allow(unused_mut)]
        use futures::{future::BoxFuture, stream::BoxStream, TryStreamExt, StreamExt};
        use std::future::IntoFuture;
        use crate::{error::Result, utils::stream_paginated};
        use #mod_path::*;
        use super::client::*;

        #(#builder_tokens)*
    };

    format_tokens(tokens)
}

/// Generate a builder for a specific request type
fn generate_request_builder(
    method: MethodHandler<'_>,
    service: &ServiceHandler<'_>,
) -> Result<String, Box<dyn std::error::Error>> {
    let builder_ident = method.builder_type();
    let request_type_ident = method.input_type().unwrap();
    let output_type_ident = method.output_type();
    let client_type_ident = service.client_type();
    let method_name = format_ident!("{}", method.plan.handler_function_name);

    // Analyze fields to determine required vs optional
    let (required_fields, optional_fields) = method.analyze_request_fields();

    // Generate constructor
    let constructor = generate_constructor(
        &method,
        &builder_ident,
        &request_type_ident,
        &client_type_ident,
        &required_fields,
    );

    // Generate with_* methods for optional fields
    let with_methods = generate_with_methods(&method, &optional_fields);

    let into_stream_impl = if matches!(method.plan.request_type, RequestType::List) {
        Some(generate_into_stream_impl(method, &method_name))
    } else {
        None
    };

    // Generate IntoFuture implementation
    let into_future_impl = generate_into_future_impl(
        &builder_ident,
        &client_type_ident,
        output_type_ident.as_ref(),
        &method_name,
    );

    let tokens = quote! {
        /// Builder for creating requests
        pub struct #builder_ident {
            client: #client_type_ident,
            request: #request_type_ident,
        }

        impl #builder_ident {
            #constructor

            #(#with_methods)*

            #into_stream_impl
        }

        #into_future_impl
    };

    Ok(format_tokens(tokens))
}

/// Generate the constructor for the builder
fn generate_constructor(
    method: &MethodHandler<'_>,
    _builder_ident: &proc_macro2::Ident,
    request_type_ident: &proc_macro2::Ident,
    client_type_ident: &proc_macro2::Ident,
    required_fields: &[&MessageField],
) -> TokenStream {
    let param_list: Vec<TokenStream> = required_fields
        .iter()
        .map(|field| {
            let field_ident = format_ident!("{}", field.name);
            let param_type = method.rust_parameter_type(&field.field_type);
            quote! { #field_ident: #param_type }
        })
        .collect();

    let field_assignments: Vec<TokenStream> = required_fields
        .iter()
        .map(|field| {
            let field_ident = format_ident!("{}", field.name);
            let assignment = method.field_assignment(&field.field_type, &field_ident);
            quote! { #field_ident: #assignment }
        })
        .collect();

    quote! {
        /// Create a new builder instance
        pub(crate) fn new(client: #client_type_ident, #(#param_list),*) -> Self {
            let request = #request_type_ident {
                #(#field_assignments,)*
                ..Default::default()
            };
            Self { client, request }
        }
    }
}

/// Generate with_* methods for optional fields
fn generate_with_methods(
    method: &MethodHandler<'_>,
    optional_fields: &[&MessageField],
) -> Vec<TokenStream> {
    let mut methods = Vec::new();

    // First, generate individual methods for oneof variants
    for field in optional_fields {
        if matches!(field.unified_type.base_type, BaseType::OneOf(_))
            && field.oneof_variants.is_some()
        {
            methods.extend(generate_oneof_variant_methods(method, field));
        }
    }

    // Then generate regular methods for non-oneof fields
    let regular_methods: Vec<TokenStream> = optional_fields
        .iter()
        .filter(|field| !matches!(field.unified_type.base_type, BaseType::OneOf(_)))
        .map(|field| builder_with_impl(method, field))
        .collect();

    methods.extend(regular_methods);
    methods
}

fn builder_with_impl(method: &MethodHandler<'_>, field: &MessageField) -> TokenStream {
    let field_ident = format_ident!("{}", field.name);
    let method_name = format_ident!("with_{}", field.name);
    let field_name = &field.name;

    // Generate appropriate documentation for the method
    let doc_attr = if let Some(ref doc) = field.documentation {
        quote! { #[doc = #doc] }
    } else {
        quote! { #[doc = concat!("Set ", #field_name)] }
    };

    if matches!(field.unified_type.base_type, BaseType::Map(_, _)) {
        quote! {
            #doc_attr
            pub fn #method_name<I, K, V>(mut self, #field_ident: I) -> Self
            where
                I: IntoIterator<Item = (K, V)>,
                K: Into<String>,
                V: Into<String>,
            {
                self.request.#field_ident = #field_ident
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect();
                self
            }
        }
    } else if matches!(field.unified_type.base_type, BaseType::Enum(_)) {
        let enum_ident =
            method.rust_field_type_unified(&field.unified_type, RenderContext::BuilderMethod);
        let assignment = method.flexible_optional_field_assignment(&field.field_type, &field_ident);
        quote! {
            #doc_attr
            pub fn #method_name(mut self, #field_ident: impl Into<Option<#enum_ident>>) -> Self {
                self.request.#field_ident = #assignment;
                self
            }
        }
    } else {
        let field_type =
            method.rust_field_type_unified(&field.unified_type, RenderContext::BuilderMethod);
        if field.repeated {
            let assignment = quote! { #field_ident.into_iter().collect() };
            quote! {
                #doc_attr
                pub fn #method_name<I>(mut self, #field_ident: I) -> Self
                where
                    I: IntoIterator<Item = #field_type>,
                {
                    self.request.#field_ident = #assignment;
                    self
                }
            }
        } else if field.optional {
            quote! {
                #doc_attr
                pub fn #method_name(mut self, #field_ident: impl Into<Option<#field_type>>) -> Self {
                    self.request.#field_ident = #field_ident.into();
                    self
                }
            }
        } else {
            quote! {
                #doc_attr
                pub fn #method_name(mut self, #field_ident: #field_type) -> Self {
                    self.request.#field_ident = #field_ident;
                    self
                }
            }
        }
    }
}

/// Generate individual methods for each variant of a oneof field
fn generate_oneof_variant_methods(
    method: &MethodHandler<'_>,
    field: &MessageField,
) -> Vec<TokenStream> {
    let variants = field.oneof_variants.as_ref().unwrap();
    let oneof_field_ident = format_ident!("{}", field.name);

    let enum_type_tokens =
        method.rust_field_type_unified(&field.unified_type, RenderContext::BuilderMethod);

    variants
        .iter()
        .map(|variant| {
            let method_name = format_ident!("with_{}", variant.field_name);
            let param_ident = format_ident!("{}", variant.field_name.replace("_", ""));
            let variant_name = format_ident!("{}", variant.variant_name);

            // Parse the rust type for the parameter
            let param_type: syn::Type = syn::parse_str(&variant.rust_type)
                .unwrap_or_else(|_| syn::parse_str("String").unwrap());

            // Generate documentation
            let doc_attr = if let Some(ref doc) = variant.documentation {
                quote! { #[doc = #doc] }
            } else {
                let field_name = &variant.field_name;
                quote! { #[doc = concat!("Set ", #field_name)] }
            };

            quote! {
                #doc_attr
                pub fn #method_name(mut self, #param_ident: #param_type) -> Self {
                    self.request.#oneof_field_ident = Some(#enum_type_tokens::#variant_name(#param_ident));
                    self
                }
            }
        })
        .collect()
}

/// Generate IntoFuture implementation
fn generate_into_future_impl(
    builder_ident: &proc_macro2::Ident,
    _client_type_ident: &proc_macro2::Ident,
    output_type_ident: Option<&proc_macro2::Ident>,
    method_name: &proc_macro2::Ident,
) -> TokenStream {
    if let Some(out_ident) = output_type_ident {
        quote! {
            impl IntoFuture for #builder_ident {
                type Output = Result<#out_ident>;
                type IntoFuture = BoxFuture<'static, Self::Output>;

                fn into_future(self) -> Self::IntoFuture {
                    let client = self.client;
                    let request = self.request;
                    Box::pin(async move { client.#method_name(&request).await })
                }
            }
        }
    } else {
        quote! {
            impl IntoFuture for #builder_ident {
                type Output = Result<()>;
                type IntoFuture = BoxFuture<'static, Self::Output>;

                fn into_future(self) -> Self::IntoFuture {
                    let client = self.client;
                    let request = self.request;
                    Box::pin(async move { client.#method_name(&request).await })
                }
            }
        }
    }
}

fn generate_into_stream_impl(
    method: MethodHandler<'_>,
    client_method_name: &proc_macro2::Ident,
) -> TokenStream {
    let items_field = method
        .output_message()
        .unwrap()
        .info
        .fields
        .iter()
        .find(|f| !f.name.contains("page_token"))
        .unwrap();
    let item_field_ident = format_ident!("{}", items_field.name);
    let output_type_ident = extract_type_ident(&items_field.field_type);

    quote! {
        /// Convert paginated request into stream of results
        pub fn into_stream(self) -> BoxStream<'static, Result<#output_type_ident>> {
            stream_paginated(self, move |mut builder, page_token| async move {
                builder.request.page_token = page_token;
                let res = builder.client.#client_method_name(&builder.request).await?;
                if let Some(ref mut remaining) = builder.request.max_results {
                    *remaining -= res.#item_field_ident.len() as i32;
                    if *remaining <= 0 {
                        builder.request.max_results = Some(0);
                    }
                }
                let next_page_token = res.next_page_token.clone();
                Ok((res, builder, next_page_token))
            })
            .map_ok(|resp| futures::stream::iter(resp.#item_field_ident.into_iter().map(Ok)))
            .try_flatten()
            .boxed()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parsing::MessageField;
    use crate::parsing::types::{BaseType, UnifiedType};

    // Note: Tests for enum conversion and field analysis are now integrated into the MethodHandler
    // and would be better tested as part of handler integration tests

    #[test]
    fn test_field_analysis_simulation() {
        // Create a temporary method handler to use the centralized method
        // Note: In practice, this test would be refactored to test the handler methods directly
        let fields = vec![
            MessageField {
                name: "name".to_string(),
                field_type: "TYPE_STRING".to_string(),
                unified_type: UnifiedType {
                    base_type: BaseType::String,
                    is_optional: false,
                    is_repeated: false,
                },
                optional: false,
                repeated: false,
                oneof_name: None,
                documentation: None,
                oneof_variants: None,
                field_behavior: vec![],
            },
            MessageField {
                name: "comment".to_string(),
                field_type: "TYPE_STRING".to_string(),
                unified_type: UnifiedType {
                    base_type: BaseType::String,
                    is_optional: true,
                    is_repeated: false,
                },
                optional: true,
                repeated: false,
                oneof_name: None,
                documentation: None,
                oneof_variants: None,
                field_behavior: vec![],
            },
            MessageField {
                name: "properties".to_string(),
                field_type: "map<string, string>".to_string(),
                unified_type: UnifiedType {
                    base_type: BaseType::Map(
                        Box::new(UnifiedType {
                            base_type: BaseType::String,
                            is_optional: false,
                            is_repeated: false,
                        }),
                        Box::new(UnifiedType {
                            base_type: BaseType::String,
                            is_optional: false,
                            is_repeated: false,
                        }),
                    ),
                    is_optional: true,
                    is_repeated: false,
                },
                optional: true,
                repeated: false,
                oneof_name: None,
                documentation: None,
                oneof_variants: None,
                field_behavior: vec![],
            },
        ];

        // Simulate field analysis logic for testing
        let mut required = Vec::new();
        let mut optional = Vec::new();

        for field in &fields {
            if field.optional {
                optional.push(field);
            } else if field.field_type.contains("map<") {
                optional.push(field);
            } else if field.field_type.starts_with("TYPE_MESSAGE:")
                || field.field_type.starts_with("TYPE_ONEOF:")
                || field.repeated
            {
                optional.push(field);
            } else {
                required.push(field);
            }
        }

        let (required, optional) = (required, optional);

        assert_eq!(required.len(), 1);
        assert_eq!(required[0].name, "name");

        assert_eq!(optional.len(), 2);
        assert_eq!(optional[0].name, "comment");
        assert_eq!(optional[1].name, "properties");
    }
}
