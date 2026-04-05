use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Path;

use super::format_tokens;
use crate::analysis::{BodyField, RequestParam, RequestType};
use crate::codegen::{MethodHandler, ServiceHandler};
use crate::parsing::types::{BaseType, RenderContext, unified_to_rust};

/// Generate builder code for all request types in a service
pub(crate) fn generate(service: &ServiceHandler<'_>) -> crate::error::Result<String> {
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
    let result_path: Path =
        syn::parse_str(&service.config.result_type_path).expect("valid result_type_path");

    let tokens = quote! {
        #![allow(unused_mut)]
        use futures::{future::BoxFuture, stream::BoxStream, TryStreamExt, StreamExt};
        use std::future::IntoFuture;
        use #result_path;
        use super::super::stream_paginated;
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
) -> crate::error::Result<String> {
    let builder_ident = method.builder_type();
    let request_type_ident = method.input_type().unwrap();
    let output_type_ident = method.output_type();
    let client_type_ident = service.client_type();
    let method_name = format_ident!("{}", method.plan.handler_function_name);

    // Constructor params: all required parameters (path + required body fields)
    let required_params: Vec<&RequestParam> = method.required_parameters().collect();
    // Optional body fields get with_* setter methods (handles oneof variant expansion)
    let (_, optional_body) = method.split_body_fields();
    // Optional query params also get simple with_* setter methods
    let optional_query: Vec<&RequestParam> = method
        .optional_parameters()
        .filter(|p| matches!(p, RequestParam::Query(_)))
        .collect();

    // Generate constructor
    let constructor = generate_constructor(
        &method,
        &request_type_ident,
        &client_type_ident,
        &required_params,
    );

    // Generate with_* methods: body fields first (handles oneof variants), then query params
    let with_methods_body = generate_with_methods(&method, &optional_body);
    let with_methods_query = optional_query
        .iter()
        .map(|param| generate_simple_with_method(&method, param));
    let with_methods: Vec<_> = with_methods_body
        .into_iter()
        .chain(with_methods_query)
        .collect();

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
    request_type_ident: &proc_macro2::Ident,
    client_type_ident: &proc_macro2::Ident,
    required_params: &[&RequestParam],
) -> TokenStream {
    let param_list = required_params.iter().map(|param| {
        let field_ident = param.field_ident();
        let param_type = method.field_type(param.field_type(), RenderContext::Constructor);
        quote! { #field_ident: #param_type }
    });

    let field_assignments = required_params.iter().map(|param| {
        let field_ident = param.field_ident();
        let assignment = method.field_assignment(
            param.field_type(),
            &field_ident,
            &RenderContext::Constructor,
        );
        quote! { #field_ident: #assignment }
    });

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
    optional_fields: &[&BodyField],
) -> Vec<TokenStream> {
    let mut methods = Vec::new();

    // First, generate individual methods for oneof variants
    for field in optional_fields {
        if matches!(field.field_type.base_type, BaseType::OneOf(_))
            && field.oneof_variants.is_some()
        {
            methods.extend(generate_oneof_variant_methods(method, field));
        }
    }

    // Then generate regular methods for non-oneof fields
    let regular_methods: Vec<TokenStream> = optional_fields
        .iter()
        .filter(|field| !matches!(field.field_type.base_type, BaseType::OneOf(_)))
        .map(|field| builder_with_impl(method, field))
        .collect();

    methods.extend(regular_methods);
    methods
}

fn builder_with_impl(method: &MethodHandler<'_>, field: &BodyField) -> TokenStream {
    let field_ident = format_ident!("{}", field.name);
    let method_name = format_ident!("with_{}", field.name);
    let field_name = &field.name;

    // Generate appropriate documentation for the method
    let doc_attr = if let Some(ref doc) = field.documentation {
        let doc_spaced = format!(" {}", doc.trim_start());
        quote! { #[doc = #doc_spaced] }
    } else {
        let set_msg = format!(" Set {}", field_name);
        quote! { #[doc = #set_msg] }
    };

    if matches!(field.field_type.base_type, BaseType::Map(_, _)) {
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
    } else if matches!(field.field_type.base_type, BaseType::Enum(_)) {
        let enum_ident = method.field_type(&field.field_type, RenderContext::BuilderMethod);
        let assignment = method.field_assignment(
            &field.field_type,
            &field_ident,
            &RenderContext::BuilderMethod,
        );
        quote! {
            #doc_attr
            pub fn #method_name(mut self, #field_ident: impl Into<Option<#enum_ident>>) -> Self {
                self.request.#field_ident = #assignment;
                self
            }
        }
    } else {
        let field_type = method.field_type(&field.field_type, RenderContext::BuilderMethod);
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
        } else {
            quote! {
                #doc_attr
                pub fn #method_name(mut self, #field_ident: impl Into<Option<#field_type>>) -> Self {
                    self.request.#field_ident = #field_ident.into();
                    self
                }
            }
        }
    }
}

/// Generate individual methods for each variant of a oneof field
fn generate_oneof_variant_methods(
    method: &MethodHandler<'_>,
    field: &BodyField,
) -> Vec<TokenStream> {
    let variants = field.oneof_variants.as_ref().unwrap();
    let oneof_field_ident = format_ident!("{}", field.name);

    let enum_type_tokens = method.field_type(&field.field_type, RenderContext::BuilderMethod);

    variants
        .iter()
        .map(|variant| {
            let method_name = format_ident!("with_{}", variant.field_name);
            let param_ident = format_ident!("{}", variant.field_name.replace("_", ""));
            let variant_name = format_ident!("{}", variant.variant_name);

            // Derive the Rust parameter type from the UnifiedType abstraction.
            let rust_type_str = unified_to_rust(&variant.field_type, RenderContext::Parameter);
            let param_type: syn::Type = syn::parse_str(&rust_type_str)
                .unwrap_or_else(|_| syn::parse_str("String").unwrap());

            // Generate documentation
            let doc_attr = if let Some(ref doc) = variant.documentation {
                let doc_spaced = format!(" {}", doc.trim_start());
                quote! { #[doc = #doc_spaced] }
            } else {
                let set_msg = format!(" Set {}", variant.field_name);
                quote! { #[doc = #set_msg] }
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

/// Generate a simple `with_*` setter for a single optional query parameter.
fn generate_simple_with_method(method: &MethodHandler<'_>, param: &RequestParam) -> TokenStream {
    let field_ident = param.field_ident();
    let method_name = format_ident!("with_{}", param.name());
    let field_name = param.name();

    let doc_attr = if let Some(doc) = param.documentation() {
        let doc_spaced = format!(" {}", doc.trim_start());
        quote! { #[doc = #doc_spaced] }
    } else {
        let set_msg = format!(" Set {}", field_name);
        quote! { #[doc = #set_msg] }
    };

    let field_type = method.field_type(param.field_type(), RenderContext::BuilderMethod);

    // Enum fields must be cast to i32 at the FFI boundary
    if matches!(param.field_type().base_type, BaseType::Enum(_)) {
        let assignment = method.field_assignment(
            param.field_type(),
            &field_ident,
            &RenderContext::BuilderMethod,
        );
        quote! {
            #doc_attr
            pub fn #method_name(mut self, #field_ident: impl Into<Option<#field_type>>) -> Self {
                self.request.#field_ident = #assignment;
                self
            }
        }
    } else {
        quote! {
            #doc_attr
            pub fn #method_name(mut self, #field_ident: impl Into<Option<#field_type>>) -> Self {
                self.request.#field_ident = #field_ident.into();
                self
            }
        }
    }
}

fn generate_into_stream_impl(
    method: MethodHandler<'_>,
    client_method_name: &proc_macro2::Ident,
) -> TokenStream {
    let items_field = method.list_output_field().unwrap();
    let item_field_ident = format_ident!("{}", items_field.name);
    let output_type_ident = items_field.unified_type.type_ident();

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
                unified_type: UnifiedType {
                    base_type: BaseType::String,
                    is_optional: false,
                    is_repeated: false,
                },
                documentation: None,
                oneof_variants: None,
                field_behavior: vec![],
            },
            MessageField {
                name: "comment".to_string(),
                unified_type: UnifiedType {
                    base_type: BaseType::String,
                    is_optional: true,
                    is_repeated: false,
                },
                documentation: None,
                oneof_variants: None,
                field_behavior: vec![],
            },
            MessageField {
                name: "properties".to_string(),
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
                documentation: None,
                oneof_variants: None,
                field_behavior: vec![],
            },
        ];

        let mut required = Vec::new();
        let mut optional = Vec::new();

        for field in &fields {
            if field.unified_type.is_optional
                || field.unified_type.is_repeated
                || matches!(
                    field.unified_type.base_type,
                    BaseType::Map(_, _) | BaseType::Message(_) | BaseType::OneOf(_)
                )
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
