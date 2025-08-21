use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Path;

use super::super::{MethodPlan, ServicePlan, templates};
use crate::utils::strings;
use crate::{MessageField, RequestType};

/// Generate builder code for all request types in a service
pub(crate) fn generate(service: &ServicePlan) -> Result<String, Box<dyn std::error::Error>> {
    let mut builder_impls = Vec::new();

    for method in &service.methods {
        // Generate builders for Create, Update, and Get operations
        if matches!(
            method.metadata.request_type(),
            RequestType::Create | RequestType::Update | RequestType::Get
        ) {
            let builder_code = generate_request_builder(method, service)?;
            builder_impls.push(builder_code);
        }
    }

    if builder_impls.is_empty() {
        return Ok(String::new());
    }

    let service_namespace = &service.base_path;
    let client_name = format!(
        "{}Client",
        service
            .handler_name
            .strip_suffix("Handler")
            .unwrap_or(&service.handler_name)
    );

    let builder_code = generate_builders_module(&client_name, &builder_impls, service_namespace);

    Ok(builder_code)
}

/// Generate the complete builders module
fn generate_builders_module(
    _client_name: &str,
    builders: &[String],
    service_namespace: &str,
) -> String {
    let builder_tokens: Vec<TokenStream> = builders
        .iter()
        .map(|b| syn::parse_str::<TokenStream>(b).unwrap_or_else(|_| quote! {}))
        .collect();

    let mod_path: Path = syn::parse_str(&format!(
        "unitycatalog_common::models::{}::v1",
        service_namespace
    ))
    .unwrap();

    let tokens = quote! {
        #![allow(unused_mut)]
        use futures::future::BoxFuture;
        use std::future::IntoFuture;
        use crate::error::Result;
        use #mod_path::*;
        use super::client::*;

        #(#builder_tokens)*
    };

    templates::format_tokens(tokens)
}

/// Generate a builder for a specific request type
fn generate_request_builder(
    method: &MethodPlan,
    service: &ServicePlan,
) -> Result<String, Box<dyn std::error::Error>> {
    let input_type = strings::extract_simple_type_name(&method.metadata.input_type);
    let output_type = strings::extract_simple_type_name(&method.metadata.output_type);

    let builder_name = format!(
        "{}Builder",
        input_type.strip_suffix("Request").unwrap_or(&input_type)
    );
    let builder_ident = format_ident!("{}", builder_name);
    let request_type_ident = format_ident!("{}", input_type);
    let output_type_ident = format_ident!("{}", output_type);
    let method_name = format_ident!("{}", method.handler_function_name);

    let client_name = format!(
        "{}Client",
        service
            .handler_name
            .strip_suffix("Handler")
            .unwrap_or(&service.handler_name)
    );
    let client_type_ident = format_ident!("{}", client_name);

    // Analyze fields to determine required vs optional
    let (required_fields, optional_fields) = analyze_request_fields(&method.metadata.input_fields);

    // Generate constructor
    let constructor = generate_constructor(
        &builder_ident,
        &request_type_ident,
        &client_type_ident,
        &required_fields,
    );

    // Generate with_* methods for optional fields
    let with_methods = generate_with_methods(&builder_ident, &optional_fields);

    // Generate IntoFuture implementation
    let into_future_impl = generate_into_future_impl(
        &builder_ident,
        &client_type_ident,
        &output_type_ident,
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
        }

        #into_future_impl
    };

    Ok(templates::format_tokens(tokens))
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
        pub fn new(client: #client_type_ident, #(#param_list),*) -> Self {
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
    optional_fields
        .iter()
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
                            get_with_method_param_type(&field.field_type).to_string()
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

                    // For non-repeated fields, wrap in Some() if optional
                    let assignment = if field.optional {
                        quote! { Some(#field_ident) }
                    } else {
                        quote! { #field_ident }
                    };

                    quote! {
                        #doc_attr
                        pub fn #method_name(mut self, #field_ident: #field_type) -> Self {
                            self.request.#field_ident = #assignment;
                            self
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
                    let param_type = get_with_method_param_type(&field.field_type);
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
        .collect()
}

/// Generate IntoFuture implementation
fn generate_into_future_impl(
    builder_ident: &proc_macro2::Ident,
    _client_type_ident: &proc_macro2::Ident,
    output_type_ident: &proc_macro2::Ident,
    method_name: &proc_macro2::Ident,
) -> TokenStream {
    quote! {
        impl IntoFuture for #builder_ident {
            type Output = Result<#output_type_ident>;
            type IntoFuture = BoxFuture<'static, Self::Output>;

            fn into_future(self) -> Self::IntoFuture {
                let client = self.client;
                let request = self.request;
                Box::pin(async move { client.#method_name(&request).await })
            }
        }
    }
}

/// Get the appropriate parameter type for constructor arguments
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

/// Get the appropriate parameter type for with_* methods
fn get_with_method_param_type(field_type: &str) -> TokenStream {
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
    use super::super::tests::create_test_service_plan;
    use super::*;

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
    use crate::{
        MessageField, MethodMetadata, gnostic::openapi::v3::Operation, google::api::HttpRule,
    };

    fn create_test_create_method() -> MethodPlan {
        let operation = Operation {
            operation_id: "CreateCatalog".to_string(),
            ..Default::default()
        };

        let http_rule = HttpRule {
            pattern: Some(crate::google::api::http_rule::Pattern::Post(
                "/catalogs".to_string(),
            )),
            body: "*".to_string(),
            ..Default::default()
        };

        let metadata = MethodMetadata {
            service_name: "CatalogsService".to_string(),
            method_name: "CreateCatalog".to_string(),
            input_type: ".unitycatalog.catalogs.v1.CreateCatalogRequest".to_string(),
            output_type: ".unitycatalog.catalogs.v1.CatalogInfo".to_string(),
            operation: Some(operation),
            http_rule: Some(http_rule),
            input_fields: vec![
                MessageField {
                    name: "name".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: false,
                    repeated: false,
                    oneof_name: None,
                    documentation: None,
                },
                MessageField {
                    name: "comment".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: true,
                    repeated: false,
                    oneof_name: None,
                    documentation: None,
                },
                MessageField {
                    name: "properties".to_string(),
                    field_type: "map<string, string>".to_string(),
                    optional: true,
                    repeated: false,
                    oneof_name: None,
                    documentation: None,
                },
                MessageField {
                    name: "storage_root".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: true,
                    repeated: false,
                    oneof_name: None,
                    documentation: None,
                },
            ],
        };

        MethodPlan {
            metadata,
            handler_function_name: "create_catalog".to_string(),
            route_function_name: "create_catalog_handler".to_string(),
            http_method: "POST".to_string(),
            http_path: "/catalogs".to_string(),
            path_params: vec![],
            query_params: vec![],
            body_fields: vec![],
            has_response: true,
        }
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
            },
            MessageField {
                name: "comment".to_string(),
                field_type: "TYPE_STRING".to_string(),
                optional: true,
                repeated: false,
                oneof_name: None,
                documentation: None,
            },
            MessageField {
                name: "properties".to_string(),
                field_type: "map<string, string>".to_string(),
                optional: true,
                repeated: false,
                oneof_name: None,
                documentation: None,
            },
        ];

        let (required, optional) = analyze_request_fields(&fields);

        assert_eq!(required.len(), 1);
        assert_eq!(required[0].name, "name");

        assert_eq!(optional.len(), 2);
        assert_eq!(optional[0].name, "comment");
        assert_eq!(optional[1].name, "properties");
    }

    #[test]
    fn test_generate_request_builder() {
        let service = create_test_service_plan();
        let method = create_test_create_method();

        let result = generate_request_builder(&method, &service);
        assert!(result.is_ok());

        let code = result.unwrap();
        println!("Generated builder code:\n{}", code);

        // Verify the code contains expected elements
        assert!(code.contains("pub struct CreateCatalogBuilder"));
        assert!(code.contains("impl CreateCatalogBuilder"));
        assert!(code.contains("pub fn new"));
        assert!(code.contains("pub fn with_comment"));
        assert!(code.contains("pub fn with_properties"));
        assert!(code.contains("impl IntoFuture"));
    }

    fn create_test_update_method() -> MethodPlan {
        let operation = Operation {
            operation_id: "UpdateCatalog".to_string(),
            ..Default::default()
        };

        let http_rule = HttpRule {
            pattern: Some(crate::google::api::http_rule::Pattern::Patch(
                "/catalogs/{name}".to_string(),
            )),
            body: "*".to_string(),
            ..Default::default()
        };

        let metadata = MethodMetadata {
            service_name: "CatalogsService".to_string(),
            method_name: "UpdateCatalog".to_string(),
            input_type: ".unitycatalog.catalogs.v1.UpdateCatalogRequest".to_string(),
            output_type: ".unitycatalog.catalogs.v1.CatalogInfo".to_string(),
            operation: Some(operation),
            http_rule: Some(http_rule),
            input_fields: vec![
                MessageField {
                    name: "name".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: false,
                    repeated: false,
                    oneof_name: None,
                    documentation: None,
                },
                MessageField {
                    name: "new_name".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: true,
                    repeated: false,
                    oneof_name: None,
                    documentation: None,
                },
                MessageField {
                    name: "comment".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: true,
                    repeated: false,
                    oneof_name: None,
                    documentation: None,
                },
                MessageField {
                    name: "owner".to_string(),
                    field_type: "TYPE_STRING".to_string(),
                    optional: true,
                    repeated: false,
                    oneof_name: None,
                    documentation: None,
                },
                MessageField {
                    name: "properties".to_string(),
                    field_type: "map<string, string>".to_string(),
                    optional: true,
                    repeated: false,
                    oneof_name: None,
                    documentation: None,
                },
            ],
        };

        MethodPlan {
            metadata,
            handler_function_name: "update_catalog".to_string(),
            route_function_name: "update_catalog_handler".to_string(),
            http_method: "PATCH".to_string(),
            http_path: "/catalogs/{name}".to_string(),
            path_params: vec![],
            query_params: vec![],
            body_fields: vec![],
            has_response: true,
        }
    }

    #[test]
    fn test_generate_update_builder() {
        let service = create_test_service_plan();
        let method = create_test_update_method();

        let result = generate_request_builder(&method, &service);
        assert!(result.is_ok());

        let code = result.unwrap();
        println!("Generated update builder code:\n{}", code);

        // Verify the code contains expected elements
        assert!(code.contains("pub struct UpdateCatalogBuilder"));
        assert!(code.contains("impl UpdateCatalogBuilder"));
        assert!(code.contains("pub fn new(client: CatalogClient, name: impl Into<String>)"));
        assert!(code.contains("pub fn with_new_name"));
        assert!(code.contains("pub fn with_comment"));
        assert!(code.contains("pub fn with_owner"));
        assert!(code.contains("pub fn with_properties"));
        assert!(code.contains("impl IntoFuture for UpdateCatalogBuilder"));
        assert!(code.contains("client.update_catalog(&request).await"));
    }

    #[test]
    fn test_generate_builders_module() {
        let service = create_test_service_plan();
        let create_method = create_test_create_method();
        let update_method = create_test_update_method();

        // Create a service plan with both create and update methods
        let mut service_with_builders = service.clone();
        service_with_builders.methods = vec![create_method, update_method];

        let result = generate(&service_with_builders);
        assert!(result.is_ok());

        let code = result.unwrap();

        if !code.is_empty() {
            println!("Generated builders module:\n{}", code);

            // Verify the code contains expected elements
            assert!(code.contains("with_properties<I, K, V>"));
            assert!(code.contains("use futures::future::BoxFuture"));
            assert!(code.contains("use std::future::IntoFuture"));
            assert!(code.contains("pub struct CreateCatalogBuilder"));
            assert!(code.contains("pub struct UpdateCatalogBuilder"));
            assert!(code.contains("impl CreateCatalogBuilder"));
            assert!(code.contains("impl UpdateCatalogBuilder"));
        }
    }
}
