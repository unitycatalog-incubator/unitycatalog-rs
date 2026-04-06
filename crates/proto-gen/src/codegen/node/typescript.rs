//! TypeScript client generation for idiomatic Node.js API.
//!
//! Generates a `client.ts` that wraps NAPI-RS native bindings with typed
//! protobuf decoding via `@bufbuild/protobuf`.

use convert_case::{Case, Casing};
use itertools::Itertools;

use super::super::python::derive_resource_accessor_params;
use crate::analysis::{RequestParam, RequestType};
use crate::codegen::{BindingsConfig, MethodHandler, ServiceHandler};
use crate::parsing::types::{BaseType, unified_to_typescript};

/// Format optional documentation as a JSDoc comment block.
fn format_jsdoc(documentation: Option<&str>, indent: &str) -> String {
    let Some(doc) = documentation else {
        return String::new();
    };
    let trimmed = doc.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let lines: Vec<String> = trimmed
        .lines()
        .map(|l| format!("{}   * {}", indent, l.trim()))
        .collect();
    format!("{}/**\n{}\n{}   */\n", indent, lines.join("\n"), indent)
}

fn is_napi_supported(param: &RequestParam) -> bool {
    is_napi_supported_type(&param.field_type().base_type)
}

fn is_napi_supported_type(base_type: &BaseType) -> bool {
    match base_type {
        BaseType::String
        | BaseType::Int32
        | BaseType::Int64
        | BaseType::Bool
        | BaseType::Float32
        | BaseType::Float64
        | BaseType::Bytes
        | BaseType::Unit
        | BaseType::Enum(_) => true,
        BaseType::Map(k, v) => {
            is_napi_supported_type(&k.base_type) && is_napi_supported_type(&v.base_type)
        }
        BaseType::Message(_) | BaseType::OneOf(_) => false,
    }
}

/// TypeScript error class definitions and `parseNativeError` helper.
fn generate_error_classes(bindings: &BindingsConfig) -> String {
    let base = &bindings.ts_error_base_class;
    let prefix = &bindings.ts_error_code_prefix;
    format!(
        r#"// ── {base} error hierarchy ────────────────────────────────────────────────────────

/** Base class for all {base} errors. */
export class {base} extends Error {{
  readonly errorCode: string;
  constructor(message: string, errorCode: string) {{
    super(message);
    this.name = "{base}";
    this.errorCode = errorCode;
  }}
}}

export class NotFoundError extends {base} {{
  constructor(message: string) {{
    super(message, "RESOURCE_NOT_FOUND");
    this.name = "NotFoundError";
  }}
}}

export class AlreadyExistsError extends {base} {{
  constructor(message: string) {{
    super(message, "RESOURCE_ALREADY_EXISTS");
    this.name = "AlreadyExistsError";
  }}
}}

export class PermissionDeniedError extends {base} {{
  constructor(message: string) {{
    super(message, "PERMISSION_DENIED");
    this.name = "PermissionDeniedError";
  }}
}}

export class UnauthenticatedError extends {base} {{
  constructor(message: string) {{
    super(message, "UNAUTHENTICATED");
    this.name = "UnauthenticatedError";
  }}
}}

export class InvalidParameterError extends {base} {{
  constructor(message: string) {{
    super(message, "INVALID_PARAMETER_VALUE");
    this.name = "InvalidParameterError";
  }}
}}

export class RequestLimitError extends {base} {{
  constructor(message: string) {{
    super(message, "REQUEST_LIMIT_EXCEEDED");
    this.name = "RequestLimitError";
  }}
}}

export class InternalServerError extends {base} {{
  constructor(message: string) {{
    super(message, "INTERNAL_ERROR");
    this.name = "InternalServerError";
  }}
}}

export class ServiceUnavailableError extends {base} {{
  constructor(message: string) {{
    super(message, "TEMPORARILY_UNAVAILABLE");
    this.name = "ServiceUnavailableError";
  }}
}}

type ErrorConstructor = new (message: string) => {base};

const ERROR_MAP: Record<string, ErrorConstructor> = {{
  RESOURCE_NOT_FOUND: NotFoundError,
  RESOURCE_ALREADY_EXISTS: AlreadyExistsError,
  PERMISSION_DENIED: PermissionDeniedError,
  UNAUTHENTICATED: UnauthenticatedError,
  INVALID_PARAMETER_VALUE: InvalidParameterError,
  REQUEST_LIMIT_EXCEEDED: RequestLimitError,
  INTERNAL_ERROR: InternalServerError,
  TEMPORARILY_UNAVAILABLE: ServiceUnavailableError,
}};

/**
 * Parse a native NAPI error that may carry a `{prefix}:<CODE>:<message>` prefix
 * and re-throw as the appropriate typed subclass of `{base}`.
 */
function parseNativeError(e: unknown): never {{
  if (e instanceof Error) {{
    const match = e.message.match(/^{prefix}:([^:]+):([\s\S]*)$/);
    if (match) {{
      const [, code, message] = match;
      const Ctor = ERROR_MAP[code] ?? {base};
      throw new Ctor(message);
    }}
  }}
  throw e;
}}

// ── end {base} error hierarchy ─────────────────────────────────────────────────────

"#
    )
}

/// Generate the complete `client.ts` file for all services.
pub(crate) fn generate_client_ts(services: &[ServiceHandler<'_>]) -> String {
    let bindings = services
        .first()
        .and_then(|s| s.config.bindings.as_ref())
        .expect("bindings config required for node_ts output");

    // Sort once up-front so all loops below produce stable output regardless of
    // the HashMap iteration order of `CodeGenMetadata::services`.
    let sorted: Vec<&ServiceHandler<'_>> = services
        .iter()
        .sorted_by_key(|s| &s.plan.service_name)
        .collect();

    let mut out = String::new();

    out.push_str(&generate_imports_sorted(&sorted));
    out.push('\n');
    out.push_str(&generate_error_classes(bindings));

    // Generate options interfaces for all services
    for service in &sorted {
        for method in service.methods() {
            if let Some(iface) = generate_options_interface(&method) {
                out.push_str(&iface);
                out.push('\n');
            }
        }
    }

    // Generate resource client classes
    for service in &sorted {
        if let Some(class) = generate_resource_client_class(service) {
            out.push_str(&class);
            out.push('\n');
        }
    }

    // Generate the main aggregate client class
    out.push_str(&generate_aggregate_client_sorted(&sorted, bindings));

    out
}

fn generate_imports_sorted(services: &[&ServiceHandler<'_>]) -> String {
    let bindings = services
        .first()
        .and_then(|s| s.config.bindings.as_ref())
        .expect("bindings config required for node_ts output");

    let napi_aggregate_name = format!("Napi{}", bindings.aggregate_client_name);

    let mut type_names: Vec<String> = Vec::new();
    let mut schema_names: Vec<String> = Vec::new();

    for service in services {
        if let Some(resource) = service.resource() {
            let type_name = resource
                .type_name
                .split('.')
                .next_back()
                .unwrap_or(&resource.type_name);
            if !type_names.contains(&type_name.to_string()) {
                type_names.push(type_name.to_string());
                schema_names.push(format!("{}Schema", type_name));
            }
        }

        for method in service.methods() {
            if let Some(output) = method.output_type() {
                let name = output.to_string();
                if !type_names.contains(&name)
                    && !name.ends_with("Response")
                    && !name.ends_with("Request")
                {
                    type_names.push(name.clone());
                    schema_names.push(format!("{}Schema", name));
                }
            }
        }
    }

    type_names.sort();
    type_names.dedup();
    schema_names.sort();
    schema_names.dedup();

    let mut native_classes: Vec<String> = vec![format!("{} as NativeClient", napi_aggregate_name)];
    for service in services {
        if service.resource().is_some() {
            let napi_name = format!("Napi{}", service.client_type());
            let native_alias = format!("Native{}", service.client_type());
            native_classes.push(format!("{} as {}", napi_name, native_alias));
        }
    }
    // native_classes already stable since services slice is pre-sorted; sort for safety
    native_classes.sort();

    let type_imports = type_names
        .iter()
        .map(|t| format!("  type {},", t))
        .collect::<Vec<_>>()
        .join("\n");

    let schema_imports = schema_names
        .iter()
        .map(|s| format!("  {},", s))
        .collect::<Vec<_>>()
        .join("\n");

    let native_imports = native_classes
        .iter()
        .map(|n| format!("  {},", n))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"import {{ fromBinary }} from "@bufbuild/protobuf";
import {{
{type_imports}
{schema_imports}
}} from "./models";
import {{
{native_imports}
}} from "./native";
"#
    )
}

/// Generate an options interface for a method's optional parameters.
fn generate_options_interface(method: &MethodHandler<'_>) -> Option<String> {
    let optional_params: Vec<&RequestParam> = method
        .optional_parameters()
        .filter(|p| !p.is_path_param() && is_napi_supported(p))
        .collect();

    if optional_params.is_empty() {
        return None;
    }

    let interface_name = format!("{}Options", method.plan.metadata.method_name);

    let mut fields = String::new();
    for param in &optional_params {
        let ts_name = param.name().to_case(Case::Camel);
        let ts_type = unified_to_typescript(param.field_type());
        // Strip the " | undefined" suffix since we use `?:` syntax
        let ts_type = ts_type.strip_suffix(" | undefined").unwrap_or(&ts_type);
        if let Some(doc) = param.documentation() {
            let cleaned = doc.trim().replace('\n', "\n   * ");
            fields.push_str(&format!("  /** {} */\n", cleaned));
        }
        fields.push_str(&format!("  {}?: {};\n", ts_name, ts_type));
    }

    Some(format!(
        "export interface {} {{\n{}}}\n",
        interface_name, fields
    ))
}

/// Generate a resource client class (e.g. CatalogClient, SchemaClient).
fn generate_resource_client_class(service: &ServiceHandler<'_>) -> Option<String> {
    let resource = service.resource()?;
    let type_name = resource
        .type_name
        .split('.')
        .next_back()
        .unwrap_or(&resource.type_name);
    let client_type = service.client_type().to_string();
    let native_type = format!("Native{}", client_type);

    let mut methods = String::new();

    for method in service.methods() {
        match &method.plan.request_type {
            RequestType::Get => {
                methods.push_str(&generate_resource_get_method(&method, type_name));
            }
            RequestType::Update => {
                methods.push_str(&generate_resource_update_method(&method, type_name));
            }
            RequestType::Delete => {
                methods.push_str(&generate_resource_delete_method(&method));
            }
            _ => {}
        }
    }

    Some(format!(
        r#"export class {client_type} {{
  private readonly inner: {native_type};

  /** @internal */
  constructor(inner: {native_type}) {{
    this.inner = inner;
  }}

{methods}}}
"#
    ))
}

fn generate_resource_get_method(method: &MethodHandler<'_>, type_name: &str) -> String {
    let schema_name = format!("{}Schema", type_name);
    let options_type = format!("{}Options", method.plan.metadata.method_name);
    let jsdoc = format_jsdoc(method.plan.metadata.documentation.as_deref(), "  ");

    let optional_params: Vec<&RequestParam> = method
        .optional_parameters()
        .filter(|p| !p.is_path_param() && is_napi_supported(p))
        .collect();

    if optional_params.is_empty() {
        return format!(
            r#"{jsdoc}  async get(): Promise<{type_name}> {{
    try {{
      return fromBinary({schema_name}, await this.inner.get());
    }} catch (e) {{ throw parseNativeError(e); }}
  }}

"#
        );
    }

    let destructure_fields = optional_params
        .iter()
        .map(|p| p.name().to_case(Case::Camel))
        .collect::<Vec<_>>()
        .join(", ");

    let call_args = optional_params
        .iter()
        .map(|p| p.name().to_case(Case::Camel))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"{jsdoc}  async get(options?: {options_type}): Promise<{type_name}> {{
    const {{ {destructure_fields} }} = options || {{}};
    try {{
      return fromBinary({schema_name}, await this.inner.get({call_args}));
    }} catch (e) {{ throw parseNativeError(e); }}
  }}

"#
    )
}

fn generate_resource_update_method(method: &MethodHandler<'_>, type_name: &str) -> String {
    let schema_name = format!("{}Schema", type_name);
    let options_type = format!("{}Options", method.plan.metadata.method_name);
    let jsdoc = format_jsdoc(method.plan.metadata.documentation.as_deref(), "  ");

    let optional_params: Vec<&RequestParam> = method
        .optional_parameters()
        .filter(|p| !p.is_path_param() && is_napi_supported(p))
        .collect();

    if optional_params.is_empty() {
        return format!(
            r#"{jsdoc}  async update(): Promise<{type_name}> {{
    try {{
      return fromBinary({schema_name}, await this.inner.update());
    }} catch (e) {{ throw parseNativeError(e); }}
  }}

"#
        );
    }

    let destructure_fields = optional_params
        .iter()
        .map(|p| p.name().to_case(Case::Camel))
        .collect::<Vec<_>>()
        .join(", ");

    let call_args = optional_params
        .iter()
        .map(|p| p.name().to_case(Case::Camel))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"{jsdoc}  async update(options?: {options_type}): Promise<{type_name}> {{
    const {{ {destructure_fields} }} = options || {{}};
    try {{
      return fromBinary({schema_name}, await this.inner.update({call_args}));
    }} catch (e) {{ throw parseNativeError(e); }}
  }}

"#
    )
}

fn generate_resource_delete_method(method: &MethodHandler<'_>) -> String {
    let jsdoc = format_jsdoc(method.plan.metadata.documentation.as_deref(), "  ");
    let optional_params: Vec<&RequestParam> = method
        .optional_parameters()
        .filter(|p| !p.is_path_param() && is_napi_supported(p))
        .collect();

    if optional_params.is_empty() {
        return format!(
            r#"{jsdoc}  async delete(): Promise<void> {{
    try {{
      await this.inner.delete();
    }} catch (e) {{ throw parseNativeError(e); }}
  }}

"#
        );
    }

    let options_type = format!("{}Options", method.plan.metadata.method_name);

    let destructure_fields = optional_params
        .iter()
        .map(|p| p.name().to_case(Case::Camel))
        .collect::<Vec<_>>()
        .join(", ");

    let call_args = optional_params
        .iter()
        .map(|p| p.name().to_case(Case::Camel))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"{jsdoc}  async delete(options?: {options_type}): Promise<void> {{
    const {{ {destructure_fields} }} = options || {{}};
    try {{
      await this.inner.delete({call_args});
    }} catch (e) {{ throw parseNativeError(e); }}
  }}

"#
    )
}

/// Generate the main aggregate client class (e.g. `UnityCatalogClient`).
fn generate_aggregate_client_sorted(
    services: &[&ServiceHandler<'_>],
    bindings: &BindingsConfig,
) -> String {
    let aggregate_client_name = &bindings.aggregate_client_name;

    let mut methods = String::new();

    for service in services {
        for method in service.methods() {
            if !method.is_collection_method() {
                continue;
            }
            match &method.plan.request_type {
                RequestType::List => {
                    methods.push_str(&generate_collection_list_method(service, &method));
                }
                RequestType::Create => {
                    methods.push_str(&generate_collection_create_method(service, &method));
                }
                _ => {}
            }
        }

        // Resource accessor methods (e.g. .catalog("name"), .schema("cat", "schema"))
        if let Some(accessor) = generate_resource_accessor(service) {
            methods.push_str(&accessor);
        }
    }

    format!(
        r#"export class {aggregate_client_name} {{
  private readonly inner: NativeClient;

  constructor(url: string, token?: string) {{
    this.inner = NativeClient.fromUrl(url, token);
  }}

{methods}}}
"#
    )
}

/// Computed parameters for generating a typed aggregate-client method.
///
/// All three collection method variants (list, create-with-return, create-void) share
/// the same parameter-building logic. `MethodCallSpec` captures that shared state once
/// so the rendering functions only differ in their return type and native call expression.
struct MethodCallSpec {
    full_param_list: String,
    optional_destructure: String,
    all_args: String,
}

impl MethodCallSpec {
    fn build(
        method: &MethodHandler<'_>,
        required_params: &[&RequestParam],
        optional_params: &[&RequestParam],
    ) -> Self {
        let options_type = format!("{}Options", method.plan.metadata.method_name);

        let required_param_list = required_params
            .iter()
            .map(|p| {
                format!(
                    "{}: {}",
                    p.name().to_case(Case::Camel),
                    unified_to_typescript(p.field_type()).replace(" | undefined", "")
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        let has_options = !optional_params.is_empty();

        let full_param_list = if has_options {
            if required_param_list.is_empty() {
                format!("options?: {}", options_type)
            } else {
                format!("{}, options?: {}", required_param_list, options_type)
            }
        } else {
            required_param_list.clone()
        };

        let optional_destructure = if has_options {
            let fields = optional_params
                .iter()
                .map(|p| p.name().to_case(Case::Camel))
                .collect::<Vec<_>>()
                .join(", ");
            format!("    const {{ {} }} = options || {{}};\n", fields)
        } else {
            String::new()
        };

        let mut args: Vec<String> = required_params
            .iter()
            .map(|p| p.name().to_case(Case::Camel))
            .collect();
        for p in optional_params {
            args.push(p.name().to_case(Case::Camel));
        }
        let all_args = args.join(", ");

        Self {
            full_param_list,
            optional_destructure,
            all_args,
        }
    }
}

fn generate_collection_list_method(
    _service: &ServiceHandler<'_>,
    method: &MethodHandler<'_>,
) -> String {
    let jsdoc = format_jsdoc(method.plan.metadata.documentation.as_deref(), "  ");
    let method_name = method.plan.handler_function_name.to_case(Case::Camel);
    let items_field = match method.list_output_field() {
        Some(field) => field,
        None => return String::new(),
    };
    let item_type_name = items_field.unified_type.type_ident().to_string();
    let schema_name = format!("{}Schema", item_type_name);

    let required_params: Vec<&RequestParam> = method
        .required_parameters()
        .filter(|p| !p.is_path_param() && is_napi_supported(p))
        .collect();
    let optional_params: Vec<&RequestParam> = method
        .optional_parameters()
        .filter(|p| !p.is_path_param() && p.name() != "page_token" && is_napi_supported(p))
        .collect();

    let spec = MethodCallSpec::build(method, &required_params, &optional_params);
    let MethodCallSpec {
        full_param_list,
        optional_destructure,
        all_args,
    } = spec;

    format!(
        r#"{jsdoc}  async {method_name}({full_param_list}): Promise<{item_type_name}[]> {{
{optional_destructure}    try {{
      return (await this.inner.{method_name}({all_args})).map((data) =>
        fromBinary({schema_name}, data),
      );
    }} catch (e) {{ throw parseNativeError(e); }}
  }}

"#
    )
}

fn generate_collection_create_method(
    _service: &ServiceHandler<'_>,
    method: &MethodHandler<'_>,
) -> String {
    let output_type = match method.output_type() {
        Some(t) => t.to_string(),
        None => return generate_void_create_method(method),
    };
    let schema_name = format!("{}Schema", output_type);

    let jsdoc = format_jsdoc(method.plan.metadata.documentation.as_deref(), "  ");
    let method_name = method.plan.handler_function_name.to_case(Case::Camel);

    let required_params: Vec<&RequestParam> = method
        .required_parameters()
        .filter(|p| !p.is_path_param() && is_napi_supported(p))
        .collect();
    let optional_params: Vec<&RequestParam> = method
        .optional_parameters()
        .filter(|p| !p.is_path_param() && is_napi_supported(p))
        .collect();

    let spec = MethodCallSpec::build(method, &required_params, &optional_params);
    let MethodCallSpec {
        full_param_list,
        optional_destructure,
        all_args,
    } = spec;

    format!(
        r#"{jsdoc}  async {method_name}({full_param_list}): Promise<{output_type}> {{
{optional_destructure}    try {{
      return fromBinary({schema_name}, await this.inner.{method_name}({all_args}));
    }} catch (e) {{ throw parseNativeError(e); }}
  }}

"#
    )
}

fn generate_void_create_method(method: &MethodHandler<'_>) -> String {
    let jsdoc = format_jsdoc(method.plan.metadata.documentation.as_deref(), "  ");
    let method_name = method.plan.handler_function_name.to_case(Case::Camel);

    let required_params: Vec<&RequestParam> = method
        .required_parameters()
        .filter(|p| !p.is_path_param() && is_napi_supported(p))
        .collect();
    let optional_params: Vec<&RequestParam> = method
        .optional_parameters()
        .filter(|p| !p.is_path_param() && is_napi_supported(p))
        .collect();

    let spec = MethodCallSpec::build(method, &required_params, &optional_params);
    let MethodCallSpec {
        full_param_list,
        optional_destructure,
        all_args,
    } = spec;

    format!(
        r#"{jsdoc}  async {method_name}({full_param_list}): Promise<void> {{
{optional_destructure}    try {{
      await this.inner.{method_name}({all_args});
    }} catch (e) {{ throw parseNativeError(e); }}
  }}

"#
    )
}

fn generate_resource_accessor(service: &ServiceHandler<'_>) -> Option<String> {
    if service.plan.managed_resources.is_empty() {
        return None;
    }

    let resource = service.resource().unwrap();
    let method_name = resource.descriptor.singular.to_case(Case::Camel);
    let client_type = service.client_type().to_string();

    let params = derive_resource_accessor_params(service);

    let param_list = params
        .iter()
        .map(|p| format!("{}: string", p.to_case(Case::Camel)))
        .collect::<Vec<_>>()
        .join(", ");

    let arg_list = params
        .iter()
        .map(|p| p.to_case(Case::Camel))
        .collect::<Vec<_>>()
        .join(", ");

    Some(format!(
        r#"  {method_name}({param_list}): {client_type} {{
    return new {client_type}(this.inner.{method_name}({arg_list}));
  }}

"#
    ))
}
