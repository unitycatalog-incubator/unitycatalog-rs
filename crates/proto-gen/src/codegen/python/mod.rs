//! Python code generation module
//!
//! Split into two submodules:
//! - `bindings`: PyO3 binding generation (Rust → Python wrapper structs)
//! - `typings`: `.pyi` type stub generation for Python IDE support

mod bindings;
mod typings;

pub(crate) use bindings::{generate, main_module};
pub(crate) use typings::generate_typings;

use crate::analysis::RequestType;
use crate::codegen::{MethodHandler, ServiceHandler};
use crate::parsing::types::{BaseType, UnifiedType};

static DOCS_TARGET_WIDTH: usize = 100;

/// Extract parameter names from a `ResourceDescriptor` pattern string.
///
/// For example, `"catalogs/{catalog}/schemas/{schema}"` yields
/// `["catalog_name", "schema_name"]`.
///
/// A single-parameter resource returns `["name"]` for brevity.
pub(super) fn resource_pattern_params(pattern: &str) -> Vec<String> {
    let params: Vec<String> = pattern
        .split('/')
        .filter(|seg| seg.starts_with('{') && seg.ends_with('}'))
        .map(|seg| {
            let inner = &seg[1..seg.len() - 1];
            format!("{}_name", inner)
        })
        .collect();

    if params.len() == 1 {
        vec!["name".to_string()]
    } else {
        params
    }
}

/// Derive the parameter names for a resource accessor method on the main client.
///
/// Uses two signals from the proto metadata to determine whether the resource
/// needs a hierarchical parameter list (e.g. `catalog_name, schema_name`) or a
/// single composite name:
///
/// 1. `name_field = "full_name"` on the `ResourceDescriptor` indicates the
///    resource stores a decomposable composite name (e.g. Schema).
/// 2. The Get method's path parameter name: `{full_name}` means the API accepts
///    a pre-composed name (single param), while `{name}` means the resource uses
///    its own simple name and needs parent context from the List method.
///
/// The parent hierarchy is read from the List method's required string-typed
/// query parameters (e.g. `catalog_name`, `schema_name`).
pub(super) fn derive_resource_accessor_params(service: &ServiceHandler<'_>) -> Vec<String> {
    let resource = match service.resource() {
        Some(r) => r,
        None => return vec!["name".to_string()],
    };

    let has_explicit_full_name = resource.descriptor.name_field == "full_name";

    let get_path_param_name = service
        .methods()
        .find(|m| matches!(m.plan.request_type, RequestType::Get))
        .and_then(|m| m.plan.path_parameters().next().map(|p| p.name.clone()));

    let parent_params: Vec<String> = service
        .methods()
        .find(|m| matches!(m.plan.request_type, RequestType::List))
        .map(|m| {
            m.required_parameters()
                .filter(|p| !p.is_path_param())
                .filter(|p| matches!(p.field_type().base_type, BaseType::String))
                .map(|p| p.name().to_string())
                .collect()
        })
        .unwrap_or_default();

    let should_decompose = has_explicit_full_name
        || (get_path_param_name.as_deref() == Some("name") && !parent_params.is_empty());

    if should_decompose {
        let mut params = parent_params;
        params.push(format!("{}_name", resource.descriptor.singular));
        params
    } else {
        vec!["name".to_string()]
    }
}

fn is_list_method(method: &MethodHandler<'_>) -> bool {
    matches!(method.plan.request_type, RequestType::List)
}

fn python_type_annotation(unified_type: &UnifiedType) -> String {
    crate::parsing::types::unified_to_python_type(unified_type)
}

fn python_type_annotation_from_ident(ident: &syn::Ident) -> String {
    let type_str = ident.to_string();
    match type_str.as_str() {
        "String" => "str".to_string(),
        "i32" | "i64" => "int".to_string(),
        "f32" | "f64" => "float".to_string(),
        "bool" => "bool".to_string(),
        "Vec < u8 >" => "bytes".to_string(),
        "()" => "None".to_string(),
        _ => {
            if let Some(simple_name) = type_str.split("::").last() {
                simple_name.trim().to_string()
            } else {
                type_str
            }
        }
    }
}

fn extract_simple_type_name(full_type: &str) -> String {
    full_type
        .split('.')
        .next_back()
        .unwrap_or(full_type)
        .to_string()
}

fn sanitize_python_field_name(field_name: &str) -> String {
    match field_name {
        "not" | "and" | "or" | "is" | "in" | "def" | "class" | "if" | "else" | "for" | "while"
        | "try" | "except" | "finally" | "with" | "as" | "import" | "from" | "pass" | "break"
        | "continue" | "return" | "yield" | "raise" | "assert" | "del" | "global" | "nonlocal"
        | "lambda" | "None" | "True" | "False" | "async" | "await" => {
            format!("{}_", field_name)
        }
        _ => field_name.to_string(),
    }
}

fn clean_and_format_description(text: &str) -> String {
    let cleaned = text.trim();
    if cleaned.is_empty() {
        return String::new();
    }

    // Split on blank lines to preserve paragraph structure
    let paragraphs: Vec<String> = cleaned
        .split("\n\n")
        .map(|para| {
            para.lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|para| !para.is_empty())
        .map(|para| textwrap::fill(&para, DOCS_TARGET_WIDTH))
        .collect();

    paragraphs.join("\n\n")
}
