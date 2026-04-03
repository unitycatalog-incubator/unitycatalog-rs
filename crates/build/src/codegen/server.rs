use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Path, Type};

use super::format_tokens;
use crate::{
    analysis::{MethodPlan, RequestParam, RequestType},
    codegen::{MethodHandler, ServiceHandler},
    google::api::http_rule::Pattern,
    parsing::{RenderContext, types::BaseType},
};

/// Generate server side code for axum servers
///
/// This generates:
/// - FromRequestParts extractor implementations for path/query parameters
/// - FromRequest extractor implementations for JSON body
pub(super) fn generate_common(service: &ServiceHandler<'_>) -> String {
    let extractor_impls = service
        .methods()
        .map(|method| from_request_extractor(&method))
        .collect_vec();
    let mod_path = service.models_path_crate();
    let result_path: Path =
        syn::parse_str(&service.config.result_type_path).expect("valid result_type_path");

    // Only import RequestPartsExt when there are FromRequestParts impls (path/query params).
    let has_parts_extractors = service.methods().any(|m| {
        matches!(
            m.plan.request_type,
            RequestType::List | RequestType::Get | RequestType::Delete
        ) || matches!(
            m.plan.request_type,
            RequestType::Custom(Pattern::Get(_) | Pattern::Delete(_))
        )
    });

    let axum_imports = if has_parts_extractors {
        quote! { use axum::{RequestExt, RequestPartsExt}; }
    } else {
        quote! { use axum::RequestExt; }
    };

    let tokens = quote! {
        #![allow(unused_mut)]
        use #result_path;
        use #mod_path::*;
        #axum_imports

        #(#extractor_impls)*
    };

    format_tokens(tokens)
}

pub(super) fn generate_server(service: &ServiceHandler<'_>) -> String {
    let handler_function_impls = service
        .methods()
        .map(|method| axum_route_handler_impl(&method, &service.plan.handler_name))
        .collect_vec();

    let mod_path = service.models_path();
    let trait_path: Path =
        syn::parse_str(&format!("super::handler::{}", &service.plan.handler_name)).unwrap();
    let result_path: Path =
        syn::parse_str(&service.config.result_type_path).expect("valid result_type_path");

    let tokens = quote! {
        #![allow(unused_mut)]
        use #result_path;
        use #mod_path::*;
        use #trait_path;
        use axum::extract::State;

        #(#handler_function_impls)*

    };

    format_tokens(tokens)
}

/// Generate extractor implementation for a specific method
fn from_request_extractor(method: &MethodHandler<'_>) -> TokenStream {
    match &method.plan.request_type {
        RequestType::List | RequestType::Get | RequestType::Delete => {
            from_request_parts_impl(method)
        }
        RequestType::Create | RequestType::Update => from_request_impl(method),
        RequestType::Custom(pattern) => match pattern {
            Pattern::Get(_) | Pattern::Delete(_) => from_request_parts_impl(method),
            Pattern::Post(_) | Pattern::Patch(_) | Pattern::Put(_) => from_request_impl(method),
            Pattern::Custom(_) => from_request_impl(method),
        },
    }
}

/// Generate route handler function
fn axum_route_handler_impl(method: &MethodHandler<'_>, handler_trait: &str) -> TokenStream {
    let handler_method = format_ident!("{}", method.plan.handler_function_name);
    let input_type = method.input_type();
    let handler_trait_ident = format_ident!("{}", handler_trait);

    if method.plan.has_response {
        let output_type = method.output_type();
        quote! {
            pub async fn #handler_method<T, Cx>(
                State(handler): State<T>,
                context: Cx,
                request: #input_type,
            ) -> Result<::axum::Json<#output_type>>
            where
                T: #handler_trait_ident<Cx> + Clone + Send + Sync + 'static,
                Cx: axum::extract::FromRequestParts<T> + Send,
            {
                let result = handler.#handler_method(request, context).await?;
                Ok(axum::Json(result))
            }
        }
    } else {
        quote! {
            pub async fn #handler_method<T, Cx>(
                State(handler): State<T>,
                context: Cx,
                request: #input_type,
            ) -> Result<()>
            where
                T: #handler_trait_ident<Cx> + Clone + Send + Sync + 'static,
                Cx: axum::extract::FromRequestParts<T> + Send,
            {
                handler.#handler_method(request, context).await?;
                Ok(())
            }
        }
    }
}

/// Generate FromRequestParts implementation for path/query parameters
fn from_request_parts_impl(method: &MethodHandler<'_>) -> TokenStream {
    let input_type = method.input_type();
    let path_extractions = path_extractions(method);
    let query_extractions = query_extractions(method);
    let field_assignments = field_assignments(method.plan);

    quote! {
        impl<S: Send + Sync> axum::extract::FromRequestParts<S> for #input_type {
            type Rejection = axum::response::Response;

            async fn from_request_parts(
                parts: &mut axum::http::request::Parts,
                _state: &S,
            ) -> Result<Self, Self::Rejection> {
                #path_extractions
                #query_extractions

                Ok(#input_type {
                    #field_assignments
                })
            }
        }
    }
}

/// Generate FromRequest implementation for JSON body
fn from_request_impl(method: &MethodHandler<'_>) -> TokenStream {
    let input_type = method.input_type();

    let is_hybrid = method
        .plan
        .parameters
        .iter()
        .any(|param| matches!(param, RequestParam::Path(_) | RequestParam::Query(_)));

    // Check if we need a hybrid extractor (path/query + body)
    if is_hybrid {
        // Generate hybrid implementation
        generate_hybrid_request_impl(method)
    } else {
        // Simple JSON body extraction
        quote! {
            impl<S: Send + Sync> axum::extract::FromRequest<S> for #input_type {
                type Rejection = axum::response::Response;

                async fn from_request(
                    req: axum::extract::Request<axum::body::Body>,
                    _state: &S,
                ) -> Result<Self, Self::Rejection> {
                    let axum::extract::Json(request) = req
                        .extract()
                        .await
                        .map_err(axum::response::IntoResponse::into_response)?;
                    Ok(request)
                }
            }
        }
    }
}

/// Generate hybrid FromRequest implementation for methods with path/query + body
fn generate_hybrid_request_impl(method: &MethodHandler<'_>) -> TokenStream {
    let input_type = method.input_type().unwrap();
    let path_extractions = path_extractions(method);
    let query_extractions = query_extractions(method);
    // Oneof fields deserialize from JSON like any other field, so no special treatment needed.
    let body_extractions = generate_body_extractions_tokens(method.plan, &input_type);
    let field_assignments = field_assignments(method.plan);

    quote! {
        impl<S: Send + Sync> axum::extract::FromRequest<S> for #input_type {
            type Rejection = axum::response::Response;

            async fn from_request(
                mut req: axum::extract::Request<axum::body::Body>,
                _state: &S,
            ) -> Result<Self, Self::Rejection> {
                // Extract path and query parameters
                let (mut parts, body) = req.into_parts();
                #path_extractions
                #query_extractions

                // Extract body fields
                let body_req = axum::extract::Request::from_parts(parts, body);
                #body_extractions

                Ok(#input_type {
                    #field_assignments
                })
            }
        }
    }
}

/// Generate path parameter extractions as TokenStream
fn path_extractions(method: &MethodHandler<'_>) -> TokenStream {
    let params = &method.plan.path_parameters().collect_vec();

    if params.is_empty() {
        quote! {}
    } else {
        let param_names: Vec<Ident> = params
            .iter()
            .map(|p| format_ident!("{}", p.field_name))
            .collect();
        let param_types: Vec<Type> = params
            .iter()
            .map(|p| method.field_type(&p.field_type, RenderContext::Extractor))
            .collect();

        quote! {
            let axum::extract::Path((#(#param_names),*)) = parts
                .extract::<axum::extract::Path<(#(#param_types),*)>>()
                .await
                .map_err(axum::response::IntoResponse::into_response)?;
        }
    }
}

/// Generate query parameter extractions as TokenStream
fn query_extractions(method: &MethodHandler<'_>) -> TokenStream {
    let params = method.plan.query_parameters().collect_vec();
    if params.is_empty() {
        quote! {}
    } else {
        let query_fields = params.iter().map(|p| {
            let name = format_ident!("{}", p.name);
            // Use QueryExtractor so enums render as their actual type (not i32):
            // query strings carry variant names as strings, not integers.
            let type_tokens = method.field_type(&p.field_type, RenderContext::QueryExtractor);
            // Repeated fields need #[serde(default)] so an absent key deserializes as an
            // empty Vec rather than a deserialization error.
            if p.is_optional() || p.field_type.is_repeated {
                quote! { #[serde(default)] #name: #type_tokens }
            } else {
                quote! { #name: #type_tokens }
            }
        });

        let param_names: Vec<Ident> = params.iter().map(|p| format_ident!("{}", p.name)).collect();

        quote! {
            #[derive(serde::Deserialize)]
            struct QueryParams {
                #(#query_fields,)*
            }
            // axum_extra::extract::Query uses serde_html_form which supports repeated query
            // parameters (?foo=a&foo=b → Vec<T>), unlike axum::extract::Query (serde_urlencoded).
            let axum_extra::extract::Query(QueryParams { #(#param_names),* }) = parts
                .extract::<axum_extra::extract::Query<QueryParams>>()
                .await
                .map_err(axum::response::IntoResponse::into_response)?;
        }
    }
}

/// Generate field assignments for request struct construction as TokenStream
fn field_assignments(method: &MethodPlan) -> TokenStream {
    let assignments = method.parameters.iter().map(|param| {
        let ident = param.field_ident();
        // Enum query params are extracted as their actual Rust type (via QueryExtractor context)
        // but prost struct fields store enums as i32, so we cast here.
        match param {
            RequestParam::Query(q) if matches!(q.field_type.base_type, BaseType::Enum(_)) => {
                if q.field_type.is_repeated {
                    quote! { #ident: #ident.into_iter().map(|v| v as i32).collect() }
                } else if q.is_optional() {
                    quote! { #ident: #ident.map(|v| v as i32) }
                } else {
                    quote! { #ident: #ident as i32 }
                }
            }
            _ => quote! { #ident },
        }
    });
    quote! { #(#assignments,)* }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::{QueryParam, RequestParam};
    use crate::parsing::types::{BaseType, UnifiedType};

    fn make_query_plan(params: Vec<RequestParam>) -> MethodPlan {
        use crate::analysis::RequestType;
        use crate::google::api::{HttpRule, http_rule::Pattern};
        use crate::parsing::{CodeGenMetadata, HttpPattern, MethodMetadata};
        MethodPlan {
            metadata: MethodMetadata {
                service_name: "TestService".to_string(),
                method_name: "ListThings".to_string(),
                input_type: "ListThingsRequest".to_string(),
                output_type: "ListThingsResponse".to_string(),
                operation: None,
                http_rule: HttpRule {
                    selector: "".to_string(),
                    pattern: Some(Pattern::Get("/things".to_string())),
                    body: "".to_string(),
                    response_body: "".to_string(),
                    additional_bindings: vec![],
                },
                input_fields: vec![],
                documentation: None,
            },
            handler_function_name: "list_things".to_string(),
            http_pattern: HttpPattern::parse("/things"),
            http_method: "GET".to_string(),
            parameters: params,
            has_response: true,
            request_type: RequestType::List,
            output_resource_type: None,
        }
    }

    fn repeated_string_param(name: &str) -> RequestParam {
        RequestParam::Query(QueryParam {
            name: name.to_string(),
            field_type: UnifiedType {
                base_type: BaseType::String,
                is_optional: false,
                is_repeated: true,
            },
            documentation: None,
        })
    }

    fn optional_enum_param(name: &str) -> RequestParam {
        RequestParam::Query(QueryParam {
            name: name.to_string(),
            field_type: UnifiedType {
                base_type: BaseType::Enum("unitycatalog.tables.v1.TableType".to_string()),
                is_optional: true,
                is_repeated: false,
            },
            documentation: None,
        })
    }

    fn repeated_enum_param(name: &str) -> RequestParam {
        RequestParam::Query(QueryParam {
            name: name.to_string(),
            field_type: UnifiedType {
                base_type: BaseType::Enum("unitycatalog.tables.v1.TableType".to_string()),
                is_optional: false,
                is_repeated: true,
            },
            documentation: None,
        })
    }

    #[test]
    fn test_field_assignments_repeated_string_uses_shorthand() {
        let plan = make_query_plan(vec![repeated_string_param("tags")]);
        let tokens = field_assignments(&plan).to_string();
        // Repeated strings use struct shorthand (no cast needed)
        assert!(tokens.contains("tags"), "should emit 'tags'");
        assert!(!tokens.contains("as i32"), "should not cast string to i32");
    }

    #[test]
    fn test_field_assignments_optional_enum_casts_to_i32() {
        let plan = make_query_plan(vec![optional_enum_param("table_type")]);
        let tokens = field_assignments(&plan).to_string();
        assert!(
            tokens.contains("map"),
            "optional enum should use .map(|v| v as i32)"
        );
        assert!(tokens.contains("as i32"), "should cast enum to i32");
    }

    #[test]
    fn test_field_assignments_repeated_enum_collects_as_i32() {
        let plan = make_query_plan(vec![repeated_enum_param("table_types")]);
        let tokens = field_assignments(&plan).to_string();
        assert!(
            tokens.contains("into_iter"),
            "repeated enum should use into_iter().map(|v| v as i32).collect()"
        );
        assert!(
            tokens.contains("as i32"),
            "should cast enum variants to i32"
        );
    }
}

/// Generate body parameter extractions as TokenStream
fn generate_body_extractions_tokens(method: &MethodPlan, response_type: &Ident) -> TokenStream {
    let body_fields = method.body_fields().collect_vec();
    if body_fields.is_empty() {
        quote! {}
    } else {
        let field_names: Vec<_> = body_fields
            .iter()
            .map(|f| format_ident!("{}", f.name))
            .collect();
        quote! {
            let axum::extract::Json::<#response_type>(body) = body_req
                .extract()
                .await
                .map_err(axum::response::IntoResponse::into_response)?;
            let (#(#field_names),*) = (
                #(body.#field_names),*
            );
        }
    }
}
