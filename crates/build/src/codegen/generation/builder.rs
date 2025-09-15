use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Path;

use super::format_tokens;
use crate::analysis::RequestType;
use crate::codegen::{MethodHandler, ServiceHandler};
use crate::parsing::MessageField;

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
        use futures::{future::BoxFuture, Stream};
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
    let (required_fields, optional_fields) =
        analyze_request_fields(&method.plan.metadata.input_fields);

    // Generate constructor
    let constructor = generate_constructor(
        &builder_ident,
        &request_type_ident,
        &client_type_ident,
        &required_fields,
    );

    // Generate with_* methods for optional fields
    let with_methods = generate_with_methods(&builder_ident, &optional_fields);

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

/// Analyze request fields to separate required from optional
fn analyze_request_fields(fields: &[MessageField]) -> (Vec<&MessageField>, Vec<&MessageField>) {
    let mut required = Vec::new();
    let mut optional = Vec::new();

    for field in fields {
        if field.optional {
            optional.push(field);
        } else if field.field_type.contains("map<") {
            // Maps are not required in constructor, but are optional with_* methods
            optional.push(field);
        } else if field.field_type.starts_with("TYPE_MESSAGE:")
            || field.field_type.starts_with("TYPE_ONEOF:")
            || field.repeated
        {
            // Complex message types, oneof fields, and repeated fields go to optional with direct setters
            optional.push(field);
        } else {
            required.push(field);
        }
    }

    (required, optional)
}

/// Generate the constructor for the builder
fn generate_constructor(
    _builder_ident: &proc_macro2::Ident,
    request_type_ident: &proc_macro2::Ident,
    client_type_ident: &proc_macro2::Ident,
    required_fields: &[&MessageField],
) -> TokenStream {
    let param_list: Vec<TokenStream> = required_fields
        .iter()
        .map(|field| {
            let field_ident = format_ident!("{}", field.name);
            let param_type = get_constructor_param_type(&field.field_type);
            quote! { #field_ident: #param_type }
        })
        .collect();

    let field_assignments: Vec<TokenStream> = required_fields
        .iter()
        .map(|field| {
            let field_ident = format_ident!("{}", field.name);
            let assignment = get_field_assignment(&field.field_type, &field_ident);
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
    _builder_ident: &proc_macro2::Ident,
    optional_fields: &[&MessageField],
) -> Vec<TokenStream> {
    let mut methods = Vec::new();

    // First, generate individual methods for oneof variants
    for field in optional_fields {
        if field.field_type.starts_with("TYPE_ONEOF:") && field.oneof_variants.is_some() {
            methods.extend(generate_oneof_variant_methods(field));
            continue; // Skip the regular oneof method generation
        }
    }

    // Then generate regular methods for non-oneof fields
    let regular_methods: Vec<TokenStream> = optional_fields
        .iter()
        .filter(|field| !(field.field_type.starts_with("TYPE_ONEOF:") && field.oneof_variants.is_some()))
        .map(|field| {
            let field_ident = format_ident!("{}", field.name);
            let method_name = format_ident!("with_{}", field.name);
            let field_name = &field.name;

            // Generate appropriate documentation for the method
            let doc_attr = if let Some(ref doc) = field.documentation {
                quote! { #[doc = #doc] }
            } else {
                quote! { #[doc = concat!("Set ", #field_name)] }
            };

            if field.field_type.contains("map<") || field.name == "properties" {
                // Handle HashMap properties with generic method
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
            } else if field.field_type.starts_with("TYPE_MESSAGE:")
                || field.field_type.starts_with("TYPE_ONEOF:")
                || field.repeated
            {
                // Handle complex types with direct assignment - no trait bounds needed
                let (field_type_str, is_repeated) = if field.repeated {
                    if field.field_type.starts_with("TYPE_MESSAGE:") {
                        // For repeated message fields, extract the inner type
                        let inner_type = field
                            .field_type
                            .strip_prefix("TYPE_MESSAGE:")
                            .unwrap_or(&field.field_type)
                            .trim_start_matches('.');

                        // Convert protobuf message names to Rust types
                        let parts: Vec<&str> = inner_type.split('.').collect();
                        let rust_type = if parts.len() >= 2 {
                            let parent_message = parts[parts.len() - 2];
                            let nested_type = parts[parts.len() - 1];

                            // For direct types like "v1::ColumnInfo", just use the type name
                            if parent_message == "v1" {
                                nested_type.to_string()
                            } else {
                                // For nested messages like "CreateCatalogRequest.PropertiesEntry"
                                let snake_case_parent = parent_message.to_case(Case::Snake);
                                format!("{}::{}", snake_case_parent, nested_type)
                            }
                        } else {
                            inner_type
                                .split('.')
                                .next_back()
                                .unwrap_or(inner_type)
                                .to_string()
                        };

                        (rust_type, true)
                    } else {
                        // For other repeated fields, determine the correct base type
                        let base_type = if field.field_type.starts_with("TYPE_STRING") {
                            "String".to_string()
                        } else {
                            get_constructor_param_type(&field.field_type).to_string()
                        };
                        (base_type, true)
                    }
                } else if field.field_type.starts_with("TYPE_MESSAGE:") {
                    let inner_type = field
                        .field_type
                        .strip_prefix("TYPE_MESSAGE:")
                        .unwrap_or(&field.field_type)
                        .trim_start_matches('.');

                    // Convert protobuf message names to Rust types
                    let rust_type = inner_type
                        .split('.')
                        .next_back()
                        .unwrap_or(inner_type)
                        .to_string();

                    (rust_type, false)
                } else if field.field_type.starts_with("TYPE_ONEOF:") {
                    let inner_type = field
                        .field_type
                        .strip_prefix("TYPE_ONEOF:")
                        .unwrap_or(&field.field_type);

                    // The field type is already in the format "createcredentialrequest::Credential"
                    // We need to convert it to snake_case: "create_credential_request::Credential"

                    let rust_type = if inner_type.contains("::") {
                        let parts: Vec<&str> = inner_type.split("::").collect();
                        if parts.len() == 2 {
                            let module_name = parts[0];
                            let type_name = parts[1];

                            // Convert module name to snake_case
                            // Handle specific known cases like "createcredentialrequest" -> "create_credential_request"
                            let snake_case_module = match module_name {
                                "createcredentialrequest" => {
                                    "create_credential_request".to_string()
                                }
                                "updatecredentialrequest" => {
                                    "update_credential_request".to_string()
                                }
                                "createcataloguestrequest" => "create_catalogs_request".to_string(),
                                "updatecataloguestrequest" => "update_catalogs_request".to_string(),
                                _ => {
                                    if module_name.chars().any(|c| c.is_uppercase()) {
                                        // If there are uppercase letters, use the standard conversion
                                        module_name.chars().fold(String::new(), |mut acc, c| {
                                            if c.is_uppercase() && !acc.is_empty() {
                                                acc.push('_');
                                            }
                                            acc.push(c.to_lowercase().next().unwrap());
                                            acc
                                        })
                                    } else {
                                        // For other cases, just use as-is
                                        module_name.to_string()
                                    }
                                }
                            };

                            format!("{}::{}", snake_case_module, type_name)
                        } else {
                            inner_type.to_string()
                        }
                    } else {
                        inner_type.to_string()
                    };

                    (rust_type, false)
                } else {
                    (field.field_type.clone(), false)
                };

                if is_repeated {
                    // For repeated fields, use generic IntoIterator
                    let inner_type: syn::Type = syn::parse_str(&field_type_str)
                        .unwrap_or_else(|_| syn::parse_str("String").unwrap());

                    // For repeated fields, assignment is always collecting the iterator
                    let assignment = quote! { #field_ident.into_iter().collect() };

                    quote! {
                        #doc_attr
                        pub fn #method_name<I>(mut self, #field_ident: I) -> Self
                        where
                            I: IntoIterator<Item = #inner_type>,
                        {
                            self.request.#field_ident = #assignment;
                            self
                        }
                    }
                } else {
                    // Parse the type string as a proper Type token for non-repeated fields
                    let field_type: syn::Type = syn::parse_str(&field_type_str)
                        .unwrap_or_else(|_| syn::parse_str("String").unwrap());

                    if field.optional {
                        // For optional complex types, use impl Into<Option<T>> pattern
                        quote! {
                            #doc_attr
                            pub fn #method_name(mut self, #field_ident: impl Into<Option<#field_type>>) -> Self {
                                self.request.#field_ident = #field_ident.into();
                                self
                            }
                        }
                    } else {
                        // For required complex types, use direct assignment
                        quote! {
                            #doc_attr
                            pub fn #method_name(mut self, #field_ident: #field_type) -> Self {
                                self.request.#field_ident = #field_ident;
                                self
                            }
                        }
                    }
                }
            } else {
                // Handle all other fields with appropriate type conversion
                if field.optional {
                    // Use flexible impl Into<Option<T>> pattern for optional fields
                    let assignment = get_flexible_optional_field_assignment(&field.field_type, &field_ident);

                    match field.field_type.as_str() {
                        "TYPE_STRING" => {
                            quote! {
                                #doc_attr
                                pub fn #method_name(mut self, #field_ident: impl Into<Option<String>>) -> Self {
                                    self.request.#field_ident = #field_ident.into();
                                    self
                                }
                            }
                        }
                        "TYPE_INT32" => {
                            quote! {
                                #doc_attr
                                pub fn #method_name(mut self, #field_ident: impl Into<Option<i32>>) -> Self {
                                    self.request.#field_ident = #assignment;
                                    self
                                }
                            }
                        }
                        "TYPE_INT64" => {
                            quote! {
                                #doc_attr
                                pub fn #method_name(mut self, #field_ident: impl Into<Option<i64>>) -> Self {
                                    self.request.#field_ident = #assignment;
                                    self
                                }
                            }
                        }
                        "TYPE_BOOL" => {
                            quote! {
                                #doc_attr
                                pub fn #method_name(mut self, #field_ident: impl Into<Option<bool>>) -> Self {
                                    self.request.#field_ident = #assignment;
                                    self
                                }
                            }
                        }
                        "TYPE_DOUBLE" => {
                            quote! {
                                #doc_attr
                                pub fn #method_name(mut self, #field_ident: impl Into<Option<f64>>) -> Self {
                                    self.request.#field_ident = #assignment;
                                    self
                                }
                            }
                        }
                        "TYPE_FLOAT" => {
                            quote! {
                                #doc_attr
                                pub fn #method_name(mut self, #field_ident: impl Into<Option<f32>>) -> Self {
                                    self.request.#field_ident = #assignment;
                                    self
                                }
                            }
                        }
                        _ if field.field_type.starts_with("TYPE_ENUM:") => {
                            let enum_type = convert_protobuf_enum_to_rust_type(&field.field_type);
                            let enum_ident: syn::Type =
                                syn::parse_str(&enum_type).unwrap_or_else(|_| syn::parse_str("i32").unwrap());
                            quote! {
                                #doc_attr
                                pub fn #method_name(mut self, #field_ident: impl Into<Option<#enum_ident>>) -> Self {
                                    self.request.#field_ident = #assignment;
                                    self
                                }
                            }
                        }
                        _ => {
                            quote! {
                                #doc_attr
                                pub fn #method_name(mut self, #field_ident: impl Into<Option<String>>) -> Self {
                                    self.request.#field_ident = #field_ident.into();
                                    self
                                }
                            }
                        }
                    }
                } else {
                    // Use the original pattern for required fields
                    let param_type = get_constructor_param_type(&field.field_type);
                    let assignment = get_field_assignment(&field.field_type, &field_ident);

                    quote! {
                        #doc_attr
                        pub fn #method_name(mut self, #field_ident: #param_type) -> Self {
                            self.request.#field_ident = #assignment;
                            self
                        }
                    }
                }
            }
        })
        .collect();

    methods.extend(regular_methods);
    methods
}

/// Generate individual methods for each variant of a oneof field
fn generate_oneof_variant_methods(field: &MessageField) -> Vec<TokenStream> {
    let variants = field.oneof_variants.as_ref().unwrap();
    let oneof_field_ident = format_ident!("{}", field.name);

    // Extract the enum type name from the field type
    let enum_type = field
        .field_type
        .strip_prefix("TYPE_ONEOF:")
        .unwrap_or(&field.field_type);

    let enum_type_rust = if enum_type.contains("::") {
        let parts: Vec<&str> = enum_type.split("::").collect();
        if parts.len() == 2 {
            let module_name = parts[0];
            let type_name = parts[1];

            // Convert module name to snake_case for known patterns
            let snake_case_module = match module_name {
                "createcredentialrequest" => "create_credential_request".to_string(),
                "updatecredentialrequest" => "update_credential_request".to_string(),
                "createcataloguestrequest" => "create_catalogs_request".to_string(),
                "updatecataloguestrequest" => "update_catalogs_request".to_string(),
                _ => {
                    if module_name.chars().any(|c| c.is_uppercase()) {
                        module_name.chars().fold(String::new(), |mut acc, c| {
                            if c.is_uppercase() && !acc.is_empty() {
                                acc.push('_');
                            }
                            acc.push(c.to_lowercase().next().unwrap());
                            acc
                        })
                    } else {
                        module_name.to_string()
                    }
                }
            };

            format!("{}::{}", snake_case_module, type_name)
        } else {
            enum_type.to_string()
        }
    } else {
        enum_type.to_string()
    };

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

            // Parse the enum type for the assignment
            let enum_type_tokens: syn::Type = syn::parse_str(&enum_type_rust)
                .unwrap_or_else(|_| syn::parse_str("String").unwrap());

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
    let output_type_ident = method.output_type();

    quote! {
        /// Convert paginated request into stream of results
        pub(crate) fn into_stream(&self) -> impl Stream<Item = Result<#output_type_ident>> {
            let request = self.request.clone();
            stream_paginated(request, move |mut request, page_token| async move {
                request.page_token = page_token;
                let res = self.client.#client_method_name(&request).await?;
                if let Some(ref mut remaining) = request.max_results {
                    *remaining -= res.#item_field_ident.len() as i32;
                    if *remaining <= 0 {
                        request.max_results = Some(0);
                    }
                }
                let next_page_token = res.next_page_token.clone();
                Ok((res, request, next_page_token))
            })
        }
    }
}

/// Get the appropriate parameter type for constructor-like arguments (also for builder methods)
fn get_constructor_param_type(field_type: &str) -> TokenStream {
    match field_type {
        "TYPE_STRING" => quote! { impl Into<String> },
        "TYPE_INT32" => quote! { i32 },
        "TYPE_INT64" => quote! { i64 },
        "TYPE_BOOL" => quote! { bool },
        "TYPE_DOUBLE" => quote! { f64 },
        "TYPE_FLOAT" => quote! { f32 },
        _ if field_type.starts_with("TYPE_ENUM:") => {
            let enum_type = convert_protobuf_enum_to_rust_type(field_type);
            let enum_ident: syn::Type =
                syn::parse_str(&enum_type).unwrap_or_else(|_| syn::parse_str("i32").unwrap());
            quote! { #enum_ident }
        }
        _ if field_type.contains("map<") => {
            quote! { impl IntoIterator<Item = (impl Into<String>, impl Into<String>)> }
        }
        _ => quote! { impl Into<String> },
    }
}

/// Get the appropriate field assignment for constructor
fn get_field_assignment(field_type: &str, field_ident: &proc_macro2::Ident) -> TokenStream {
    match field_type {
        "TYPE_STRING" => quote! { #field_ident.into() },
        "TYPE_INT32" | "TYPE_INT64" | "TYPE_BOOL" | "TYPE_DOUBLE" | "TYPE_FLOAT" => {
            quote! { #field_ident }
        }
        _ if field_type.starts_with("TYPE_ENUM:") => quote! { #field_ident as i32 },
        _ if field_type.contains("map<") => quote! {
            #field_ident.into_iter().map(|(k, v)| (k.into(), v.into())).collect()
        },
        _ => quote! { #field_ident.into() },
    }
}

/// Get the flexible field assignment for optional fields using impl Into<Option<T>>
fn get_flexible_optional_field_assignment(
    field_type: &str,
    field_ident: &proc_macro2::Ident,
) -> TokenStream {
    match field_type {
        "TYPE_STRING" => quote! { #field_ident.into().map(|s| s.into()) },
        "TYPE_INT32" | "TYPE_INT64" | "TYPE_BOOL" | "TYPE_DOUBLE" | "TYPE_FLOAT" => {
            quote! { #field_ident.into() }
        }
        _ if field_type.starts_with("TYPE_ENUM:") => {
            quote! { #field_ident.into().map(|e| e as i32) }
        }
        _ => quote! { #field_ident.into().map(|s| s.to_string()) },
    }
}

/// Convert protobuf enum type to Rust enum type
fn convert_protobuf_enum_to_rust_type(field_type: &str) -> String {
    if let Some(enum_name) = field_type.strip_prefix("TYPE_ENUM:") {
        // Remove leading dot if present
        let enum_name = enum_name.trim_start_matches('.');

        // Parse the enum name parts
        let parts: Vec<&str> = enum_name.split('.').collect();

        match parts.as_slice() {
            // unitycatalog.tables.v1.TableType -> TableType
            ["unitycatalog", "tables", "v1", enum_type] => enum_type.to_string(),
            // unitycatalog.credentials.v1.Purpose -> Purpose
            ["unitycatalog", "credentials", "v1", enum_type] => enum_type.to_string(),
            // unitycatalog.recipients.v1.AuthenticationType -> AuthenticationType
            ["unitycatalog", "recipients", "v1", enum_type] => enum_type.to_string(),
            // unitycatalog.volumes.v1.VolumeType -> VolumeType
            ["unitycatalog", "volumes", "v1", enum_type] => enum_type.to_string(),
            // unitycatalog.temporary_credentials.v1.generate_temporary_table_credentials_request.Operation
            [
                "unitycatalog",
                "temporary_credentials",
                "v1",
                nested_type,
                enum_type,
            ] => {
                // Convert to snake_case module name
                let snake_case_module = nested_type.chars().fold(String::new(), |mut acc, c| {
                    if c.is_uppercase() && !acc.is_empty() {
                        acc.push('_');
                    }
                    acc.push(c.to_lowercase().next().unwrap());
                    acc
                });
                format!("{}::{}", snake_case_module, enum_type)
            }
            // Fallback: use the last part as the enum name
            _ => parts.last().map_or("i32", |v| v).to_string(),
        }
    } else {
        // Not an enum type, return as-is (fallback to i32)
        "i32".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::MessageField;

    #[test]
    fn test_convert_protobuf_enum_to_rust_type() {
        // Test table types
        assert_eq!(
            convert_protobuf_enum_to_rust_type("TYPE_ENUM:.unitycatalog.tables.v1.TableType"),
            "TableType"
        );
        assert_eq!(
            convert_protobuf_enum_to_rust_type(
                "TYPE_ENUM:.unitycatalog.tables.v1.DataSourceFormat"
            ),
            "DataSourceFormat"
        );

        // Test credentials
        assert_eq!(
            convert_protobuf_enum_to_rust_type("TYPE_ENUM:.unitycatalog.credentials.v1.Purpose"),
            "Purpose"
        );

        // Test recipients
        assert_eq!(
            convert_protobuf_enum_to_rust_type(
                "TYPE_ENUM:.unitycatalog.recipients.v1.AuthenticationType"
            ),
            "AuthenticationType"
        );

        // Test volumes
        assert_eq!(
            convert_protobuf_enum_to_rust_type("TYPE_ENUM:.unitycatalog.volumes.v1.VolumeType"),
            "VolumeType"
        );

        // Test temporary credentials with nested types
        assert_eq!(
            convert_protobuf_enum_to_rust_type(
                "TYPE_ENUM:.unitycatalog.temporary_credentials.v1.GenerateTemporaryTableCredentialsRequest.Operation"
            ),
            "generate_temporary_table_credentials_request::Operation"
        );
        assert_eq!(
            convert_protobuf_enum_to_rust_type(
                "TYPE_ENUM:.unitycatalog.temporary_credentials.v1.GenerateTemporaryPathCredentialsRequest.Operation"
            ),
            "generate_temporary_path_credentials_request::Operation"
        );

        // Test fallback cases
        assert_eq!(
            convert_protobuf_enum_to_rust_type("TYPE_ENUM:.unknown.SomeEnum"),
            "SomeEnum"
        );
        assert_eq!(convert_protobuf_enum_to_rust_type("TYPE_STRING"), "i32");
        assert_eq!(convert_protobuf_enum_to_rust_type("not_enum_type"), "i32");
    }

    #[test]
    fn test_analyze_request_fields() {
        let fields = vec![
            MessageField {
                name: "name".to_string(),
                field_type: "TYPE_STRING".to_string(),
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
                optional: true,
                repeated: false,
                oneof_name: None,
                documentation: None,
                oneof_variants: None,
                field_behavior: vec![],
            },
        ];

        let (required, optional) = analyze_request_fields(&fields);

        assert_eq!(required.len(), 1);
        assert_eq!(required[0].name, "name");

        assert_eq!(optional.len(), 2);
        assert_eq!(optional[0].name, "comment");
        assert_eq!(optional[1].name, "properties");
    }
}
