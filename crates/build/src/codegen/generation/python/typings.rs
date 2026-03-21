//! Python `.pyi` type stub generation for IDE support.

use textwrap::{Options, dedent, fill, indent, refill};

use super::{
    DOCS_TARGET_WIDTH, clean_and_format_description, derive_resource_accessor_params,
    extract_simple_type_name, is_list_method, python_type_annotation,
    python_type_annotation_from_ident, resource_pattern_params, sanitize_python_field_name,
};
use crate::analysis::RequestType;
use crate::codegen::{MethodHandler, ServiceHandler};
use crate::parsing::types::{BaseType, UnifiedType, unified_to_python_type};
use crate::parsing::{CodeGenMetadata, EnumInfo, MessageField, MessageInfo};

/// Generate unified Python typings (.pyi file) for all services in a single file
pub(crate) fn generate_typings(services: &[ServiceHandler<'_>]) -> String {
    let metadata = services[0].metadata;
    let mut content = Vec::new();

    content.push("from __future__ import annotations".to_string());
    content.push("from typing import Optional, List, Dict, Any, Literal".to_string());
    content.push("import enum".to_string());
    content.push("".to_string());

    let model_classes = generate_model_classes(metadata);
    content.extend(model_classes);

    let enum_classes = generate_enum_classes(metadata);
    content.extend(enum_classes);

    let mut service_indices: Vec<usize> = (0..services.len()).collect();
    service_indices.sort_by_key(|&i| &services[i].plan.service_name);

    for &i in &service_indices {
        let service = &services[i];
        let service_class = generate_service_class_typings(service, services);
        content.push(service_class);
        content.push("".to_string());
    }

    let sorted_services: Vec<_> = service_indices.iter().map(|&i| &services[i]).collect();
    let main_client_class = generate_main_client_class_typings(&sorted_services);
    content.push(main_client_class);

    content.join("\n")
}

fn generate_method_typings_signature(method: &MethodHandler<'_>) -> Option<String> {
    let method_name = match &method.plan.request_type {
        RequestType::Get | RequestType::Update => method.plan.resource_client_method().to_string(),
        RequestType::Delete => method.plan.resource_client_method().to_string(),
        RequestType::List | RequestType::Create => method.plan.base_method_ident().to_string(),
        _ => return None,
    };

    let return_type = match &method.plan.request_type {
        RequestType::Delete => "None".to_string(),
        RequestType::List => {
            if let Some(items_field) = method.list_output_field() {
                let item_type = python_type_annotation(&items_field.unified_type);
                format!(
                    "List[{}]",
                    item_type.trim_start_matches("List[").trim_end_matches("]")
                )
            } else {
                "Any".to_string()
            }
        }
        _ => {
            if let Some(output_type) = method.output_type() {
                python_type_annotation_from_ident(&output_type)
            } else {
                "Any".to_string()
            }
        }
    };

    let parameters = generate_method_parameters_for_typings(method);
    let params_str = if parameters.is_empty() {
        "self".to_string()
    } else {
        format!("self, {}", parameters.join(", "))
    };

    let docstring = format_method_docstring_with_params(method);

    let params_vec = if params_str == "self" {
        vec!["self".to_string()]
    } else {
        params_str.split(", ").map(|s| s.to_string()).collect()
    };

    let docstring_text = if docstring.trim().is_empty() {
        None
    } else {
        Some(docstring.trim())
    };
    let method_stub =
        generate_method_template(&method_name, &params_vec, &return_type, docstring_text, 1);

    Some(method_stub)
}

fn generate_method_parameters_for_typings(method: &MethodHandler<'_>) -> Vec<String> {
    let mut parameters = Vec::new();

    for param in method.required_parameters() {
        if !param.is_path_param() {
            let param_type = python_type_annotation(param.field_type());
            parameters.push(format!("{}: {}", param.name(), param_type));
        }
    }

    for param in method.optional_parameters() {
        if !(is_list_method(method) && param.name() == "page_token") {
            let param_type = python_type_annotation(param.field_type());
            let param_type = if param_type.starts_with("Optional[") {
                param_type
            } else {
                format!("Optional[{}]", param_type)
            };
            parameters.push(format!("{}: {} = None", param.name(), param_type));
        }
    }

    parameters
}

fn format_method_docstring_with_params(method: &MethodHandler<'_>) -> String {
    let mut docstring_parts = Vec::new();

    if let Some(doc) = method.plan.metadata.documentation.as_ref() {
        let cleaned_doc = clean_and_format_description(doc);
        if !cleaned_doc.is_empty() {
            docstring_parts.push(cleaned_doc);
        }
    }

    let param_docs = collect_parameter_documentation(method);
    if !param_docs.is_empty() {
        if !docstring_parts.is_empty() {
            docstring_parts.push("".to_string());
        }
        docstring_parts.push("Args:".to_string());
        for (param_name, param_doc) in param_docs {
            let cleaned_param_doc = clean_and_format_description(&param_doc);
            let wrapped_param = format_parameter_description(&param_name, &cleaned_param_doc);
            docstring_parts.push(wrapped_param);
        }
    }

    let return_doc = get_return_type_documentation(method);
    if !return_doc.is_empty() {
        if !docstring_parts.is_empty() {
            docstring_parts.push("".to_string());
        }
        docstring_parts.push("Returns:".to_string());
        let cleaned_return_doc = clean_and_format_description(&return_doc);
        let wrapped_return = format_return_description(&cleaned_return_doc);
        docstring_parts.push(wrapped_return);
    }

    if docstring_parts.is_empty() {
        return String::new();
    }

    let mut result: Vec<String> = Vec::new();
    for (i, part) in docstring_parts.iter().enumerate() {
        if i > 0
            && !part.is_empty()
            && !result.is_empty()
            && (part.starts_with("Args:") || part.starts_with("Returns:"))
            && !result.last().unwrap().is_empty()
        {
            result.push(String::new());
        }
        result.push(part.clone());
    }
    result.join("\n")
}

fn format_parameter_description(param_name: &str, description: &str) -> String {
    let first_line_prefix = format!("    {}: ", param_name);
    let continuation_prefix = " ".repeat(first_line_prefix.len());

    let first_line_width = DOCS_TARGET_WIDTH - first_line_prefix.len();
    let continuation_width = DOCS_TARGET_WIDTH - continuation_prefix.len();

    if first_line_prefix.len() >= 90 {
        let optimally_filled = refill(description, continuation_width);
        let options = Options::new(continuation_width)
            .initial_indent(&continuation_prefix)
            .subsequent_indent(&continuation_prefix);
        return format!(
            "{}\n{}",
            first_line_prefix,
            fill(&optimally_filled, &options)
        );
    }

    let options = Options::new(first_line_width)
        .initial_indent("")
        .subsequent_indent(&continuation_prefix);

    let initial_wrapped = fill(description, &options);

    if initial_wrapped.contains('\n') {
        let refilled_desc = refill(description, continuation_width);

        let lines: Vec<&str> = refilled_desc.lines().collect();
        if lines.is_empty() {
            return first_line_prefix;
        }

        let mut result = format!("{}{}", first_line_prefix, lines[0]);
        for line in &lines[1..] {
            result.push_str(&format!("\n{}{}", continuation_prefix, line));
        }
        result
    } else {
        format!("{}{}", first_line_prefix, initial_wrapped)
    }
}

fn format_return_description(description: &str) -> String {
    let prefix = "    ";

    let available_width = DOCS_TARGET_WIDTH - prefix.len();
    let refilled_desc = refill(description, available_width);

    let options = Options::new(available_width)
        .initial_indent(prefix)
        .subsequent_indent(prefix);

    fill(&refilled_desc, &options)
}

fn collect_parameter_documentation(method: &MethodHandler<'_>) -> Vec<(String, String)> {
    let mut param_docs = Vec::new();

    for param in method.required_parameters() {
        if !param.is_path_param() {
            if let Some(doc) = param.documentation() {
                let cleaned_doc = clean_and_format_description(doc);
                if !cleaned_doc.is_empty() {
                    param_docs.push((param.name().to_string(), cleaned_doc));
                }
            }
        }
    }

    for param in method.optional_parameters() {
        if !(is_list_method(method) && param.name() == "page_token") {
            if let Some(doc) = param.documentation() {
                let cleaned_doc = clean_and_format_description(doc);
                if !cleaned_doc.is_empty() {
                    param_docs.push((param.name().to_string(), cleaned_doc));
                }
            }
        }
    }

    param_docs
}

fn get_return_type_documentation(method: &MethodHandler<'_>) -> String {
    match &method.plan.request_type {
        RequestType::Delete => "None".to_string(),
        RequestType::List => {
            if let Some(items_field) = method.list_output_field() {
                if let Some(doc) = items_field.documentation.as_ref() {
                    let cleaned_doc = clean_and_format_description(doc);
                    if cleaned_doc.is_empty() {
                        "List of items".to_string()
                    } else {
                        format!("List of {}", cleaned_doc)
                    }
                } else {
                    "List of items".to_string()
                }
            } else {
                "List of items".to_string()
            }
        }
        _ => {
            if let Some(output_message) = method.output_message() {
                if let Some(doc) = output_message.info.documentation.as_ref() {
                    let cleaned_doc = clean_and_format_description(doc);
                    if cleaned_doc.is_empty() {
                        "The requested resource".to_string()
                    } else {
                        cleaned_doc
                    }
                } else {
                    "The requested resource".to_string()
                }
            } else {
                "The requested resource".to_string()
            }
        }
    }
}

fn generate_resource_accessor_methods_for_typings(
    service: &ServiceHandler<'_>,
    all_services: &[ServiceHandler<'_>],
) -> Vec<String> {
    let parent_resource = match service.resource() {
        Some(r) => r,
        None => return vec![],
    };
    let parent_pattern = match parent_resource.descriptor.pattern.first() {
        Some(p) => p.as_str(),
        None => return vec![],
    };

    let mut methods = Vec::new();

    for other in all_services {
        let child_resource = match other.resource() {
            Some(r) => r,
            None => continue,
        };
        let child_pattern = match child_resource.descriptor.pattern.first() {
            Some(p) => p.as_str(),
            None => continue,
        };

        if child_pattern.starts_with(parent_pattern)
            && child_pattern.len() > parent_pattern.len()
            && child_pattern.as_bytes().get(parent_pattern.len()) == Some(&b'/')
        {
            let method_name = &child_resource.descriptor.singular;
            let child_params = resource_pattern_params(child_pattern);
            let mut params = vec!["self".to_string()];
            params.extend(child_params.iter().map(|p| format!("{}: str", p)));
            let return_type = format!("{}", other.client_type());

            methods.push(generate_method_template(
                method_name,
                &params,
                &return_type,
                None,
                1,
            ));
        }
    }

    methods
}

fn generate_service_class_typings(
    service: &ServiceHandler<'_>,
    all_services: &[ServiceHandler<'_>],
) -> String {
    let rust_client_ident = service.client_type();
    let client_ident = format!("{}", rust_client_ident);

    let mut method_signatures: Vec<_> = service
        .methods()
        .filter(|method| !method.is_collection_method())
        .filter_map(|method| {
            generate_method_typings_signature(&method)
                .map(|sig| (method.plan.base_method_ident().to_string(), sig))
        })
        .collect();

    method_signatures.sort_by(|a, b| a.0.cmp(&b.0));
    let mut methods: Vec<_> = method_signatures.into_iter().map(|(_, sig)| sig).collect();

    methods.extend(generate_resource_accessor_methods_for_typings(
        service,
        all_services,
    ));

    let body_content = if methods.is_empty() {
        indent("...", "    ")
    } else {
        methods.join("\n")
    };

    generate_class_from_template(&client_ident, "", None, &body_content)
}

fn generate_main_client_class_typings(services: &[&ServiceHandler<'_>]) -> String {
    let mut collection_methods = services
        .iter()
        .flat_map(|service| {
            service
                .methods()
                .filter(|m| m.is_collection_method())
                .filter_map(|method| {
                    let method_name = method.plan.base_method_ident().to_string();
                    let return_type = match &method.plan.request_type {
                        RequestType::List => {
                            if let Some(items_field) = method.list_output_field() {
                                let item_type = python_type_annotation(&items_field.unified_type);
                                format!(
                                    "List[{}]",
                                    item_type.trim_start_matches("List[").trim_end_matches("]")
                                )
                            } else {
                                "Any".to_string()
                            }
                        }
                        RequestType::Create => {
                            if let Some(output_type) = method.output_type() {
                                python_type_annotation_from_ident(&output_type)
                            } else {
                                "Any".to_string()
                            }
                        }
                        _ => return None,
                    };

                    let parameters = generate_method_parameters_for_typings(&method);
                    let mut params = vec!["self".to_string()];
                    params.extend(parameters);

                    let docstring = format_method_docstring_with_params(&method);
                    let docstring_text = if docstring.trim().is_empty() {
                        None
                    } else {
                        Some(docstring.trim())
                    };

                    Some((
                        method_name.clone(),
                        generate_method_template(
                            &method_name,
                            &params,
                            &return_type,
                            docstring_text,
                            1,
                        ),
                    ))
                })
        })
        .collect::<Vec<_>>();

    collection_methods.sort_by(|a, b| a.0.cmp(&b.0));
    let main_collection_methods = collection_methods
        .into_iter()
        .map(|(_, method)| method)
        .collect::<Vec<_>>()
        .join("\n");

    let mut resource_methods = services
        .iter()
        .filter_map(|service| {
            let resource = service.resource()?;
            let method_name = resource.descriptor.singular.clone();
            let client_name = format!("{}", service.client_type());
            let pattern_params = derive_resource_accessor_params(service);
            let mut params = vec!["self".to_string()];
            params.extend(pattern_params.iter().map(|p| format!("{}: str", p)));

            Some((
                method_name.clone(),
                generate_method_template(&method_name, &params, &client_name, None, 1),
            ))
        })
        .collect::<Vec<_>>();

    resource_methods.sort_by(|a, b| a.0.cmp(&b.0));
    let main_client_methods = resource_methods
        .into_iter()
        .map(|(_, method)| method)
        .collect::<Vec<_>>()
        .join("\n");

    let init_method = generate_method_template(
        "__init__",
        &[
            "self".to_string(),
            "base_url: str".to_string(),
            "token: Optional[str] = None".to_string(),
        ],
        "None",
        None,
        1,
    );

    let main_client_all_methods =
        if main_client_methods.is_empty() && main_collection_methods.is_empty() {
            format!("{}\n    ...", init_method)
        } else {
            [init_method, main_collection_methods, main_client_methods]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        };

    generate_class_from_template("PyUnityCatalogClient", "", None, &main_client_all_methods)
}

// --- Type collection ---

fn collect_reachable_types(metadata: &CodeGenMetadata) -> (Vec<String>, Vec<String>) {
    use std::collections::{BTreeSet, VecDeque};

    let mut message_names = BTreeSet::new();
    let mut enum_names = BTreeSet::new();
    let mut queue = VecDeque::new();

    for service in metadata.services.values() {
        for method in &service.methods {
            queue.push_back(method.input_type.clone());
            queue.push_back(method.output_type.clone());
        }
    }

    while let Some(type_name) = queue.pop_front() {
        if let Some(msg) = metadata.messages.get(&type_name) {
            if !message_names.insert(type_name.clone()) {
                continue;
            }
            for field in &msg.fields {
                collect_field_types(&field.unified_type, &mut queue);
                if let Some(variants) = &field.oneof_variants {
                    for v in variants {
                        // Enqueue any message types referenced by this variant.
                        collect_field_types(&v.field_type, &mut queue);
                    }
                }
            }
        } else if metadata.enums.contains_key(&type_name) {
            enum_names.insert(type_name.clone());
        }
    }

    let messages: Vec<String> = message_names
        .into_iter()
        .filter(|n| n.contains("unitycatalog"))
        .collect();
    let enums: Vec<String> = enum_names
        .into_iter()
        .filter(|n| n.contains("unitycatalog"))
        .collect();

    (messages, enums)
}

fn collect_field_types(ut: &UnifiedType, queue: &mut std::collections::VecDeque<String>) {
    match &ut.base_type {
        BaseType::Message(n) | BaseType::Enum(n) | BaseType::OneOf(n) => {
            let key = if n.starts_with('.') {
                n.clone()
            } else {
                format!(".{}", n)
            };
            queue.push_back(key);
        }
        BaseType::Map(k, v) => {
            collect_field_types(k, queue);
            collect_field_types(v, queue);
        }
        _ => {}
    }
}

// --- Model / enum class generation ---

fn generate_model_classes(metadata: &CodeGenMetadata) -> Vec<String> {
    let mut classes = Vec::new();
    let (reachable_messages, _) = collect_reachable_types(metadata);

    let mut matching_messages: Vec<_> = reachable_messages
        .iter()
        .filter(|name| {
            let simple = extract_simple_type_name(name);
            !simple.ends_with("Request") && !simple.ends_with("Response")
        })
        .filter_map(|name| metadata.messages.get(name.as_str()))
        .collect();

    matching_messages.sort_by_key(|msg| extract_simple_type_name(&msg.name));

    for message_info in matching_messages {
        let class_def = generate_model_class_definition(message_info);
        classes.push(class_def);
        classes.push("".to_string());
    }

    classes
}

fn generate_enum_classes(metadata: &CodeGenMetadata) -> Vec<String> {
    let mut enums = Vec::new();
    let (_, reachable_enums) = collect_reachable_types(metadata);

    let mut matching_enums: Vec<_> = reachable_enums
        .iter()
        .filter_map(|name| metadata.enums.get(name.as_str()))
        .collect();

    matching_enums.sort_by_key(|enum_info| extract_simple_type_name(&enum_info.name));

    for enum_info in matching_enums {
        let enum_def = generate_enum_class_definition(enum_info);
        enums.push(enum_def);
        enums.push("".to_string());
    }

    enums
}

fn generate_model_class_definition(message: &MessageInfo) -> String {
    let class_name = extract_simple_type_name(&message.name);
    let docstring = message
        .documentation
        .as_ref()
        .map(|doc| clean_and_format_description(doc))
        .filter(|doc| !doc.is_empty());

    let mut field_indices: Vec<usize> = (0..message.fields.len()).collect();
    field_indices.sort_by_key(|&i| &message.fields[i].name);

    let field_definitions = field_indices
        .iter()
        .map(|&i| generate_field_definition(&message.fields[i]))
        .filter(|def| !def.is_empty())
        .collect::<Vec<_>>();

    let mut oneof_field_definitions = Vec::new();
    for &i in &field_indices {
        let field = &message.fields[i];
        if let Some(variants) = &field.oneof_variants {
            let mut variant_indices: Vec<usize> = (0..variants.len()).collect();
            variant_indices.sort_by_key(|&j| &variants[j].field_name);
            for &j in &variant_indices {
                let variant = &variants[j];
                let safe_field_name = sanitize_python_field_name(&variant.field_name);
                let python_type = unified_to_python_type(&variant.field_type);

                let mut field_def = format!("    {}: Optional[{}]", safe_field_name, python_type);
                if let Some(doc) = &variant.documentation {
                    let cleaned_doc = clean_and_format_description(doc);
                    if !cleaned_doc.is_empty() {
                        field_def.push_str(&format!("\n    \"\"\"{}\"\"\"", cleaned_doc));
                    }
                }
                oneof_field_definitions.push(field_def);
            }
        }
    }

    let mut all_field_definitions = field_definitions;
    all_field_definitions.extend(oneof_field_definitions);

    let body_content = if all_field_definitions.is_empty() {
        indent("...", "    ")
    } else {
        let mut content = all_field_definitions.join("\n");
        content.push_str("\n\n");
        content.push_str(&generate_constructor_definition(message));
        content
    };

    generate_class_from_template(&class_name, "", docstring.as_deref(), &body_content)
}

fn generate_enum_class_definition(enum_info: &EnumInfo) -> String {
    let enum_name = extract_simple_type_name(&enum_info.name);
    let docstring = enum_info
        .documentation
        .as_ref()
        .map(|doc| clean_and_format_description(doc))
        .filter(|doc| !doc.is_empty());

    let body_content = if enum_info.values.is_empty() {
        indent("...", "    ")
    } else {
        let mut value_indices: Vec<usize> = (0..enum_info.values.len()).collect();
        value_indices.sort_by_key(|&i| &enum_info.values[i].name);

        let mut enum_values = Vec::new();
        for &i in &value_indices {
            let value = &enum_info.values[i];
            enum_values.push(format!("{} = \"{}\"", value.name, value.name));

            if let Some(doc) = &value.documentation {
                let cleaned_doc = clean_and_format_description(doc);
                if !cleaned_doc.is_empty() {
                    enum_values.push(format!("\"\"\"{}\"\"\"", cleaned_doc));
                }
            }
        }
        indent(&enum_values.join("\n"), "    ")
    };

    generate_class_from_template(&enum_name, "enum.Enum", docstring.as_deref(), &body_content)
}

fn generate_field_definition(field: &MessageField) -> String {
    if field.oneof_variants.is_some() {
        return String::new();
    }

    let safe_field_name = sanitize_python_field_name(&field.name);
    let mut type_annotation = python_type_annotation(&field.unified_type);

    if field.repeated {
        if type_annotation.starts_with("List[") && type_annotation.ends_with("]") {
            type_annotation = type_annotation[5..type_annotation.len() - 1].to_string();
        }
        type_annotation = format!("List[{}]", type_annotation);
    }

    if field.optional && !type_annotation.starts_with("Optional[") {
        type_annotation = format!("Optional[{}]", type_annotation);
    }

    let mut lines = Vec::new();
    lines.push(format!("    {}: {}", safe_field_name, type_annotation));

    if let Some(doc) = &field.documentation {
        let cleaned_doc = clean_and_format_description(doc);
        if !cleaned_doc.is_empty() {
            let formatted_docstring = format_field_docstring(&cleaned_doc);
            lines.push(formatted_docstring);
        }
    }

    lines.join("\n")
}

fn generate_constructor_definition(message: &MessageInfo) -> String {
    let mut params = vec!["self".to_string()];

    let mut field_indices: Vec<usize> = (0..message.fields.len()).collect();
    field_indices.sort_by_key(|&i| &message.fields[i].name);

    let mut required_fields = Vec::new();
    let mut optional_fields = Vec::new();

    for &i in &field_indices {
        let field = &message.fields[i];
        if field.oneof_variants.is_some() {
            continue;
        }

        if !field.optional && !field.repeated {
            required_fields.push(field);
        } else {
            optional_fields.push(field);
        }
    }

    for field in &required_fields {
        let mut type_annotation = python_type_annotation(&field.unified_type);

        if field.repeated {
            if type_annotation.starts_with("List[") && type_annotation.ends_with("]") {
                type_annotation = type_annotation[5..type_annotation.len() - 1].to_string();
            }
            type_annotation = format!("List[{}]", type_annotation);
        }

        let safe_field_name = sanitize_python_field_name(&field.name);
        params.push(format!("{}: {}", safe_field_name, type_annotation));
    }

    for field in &optional_fields {
        let mut type_annotation = python_type_annotation(&field.unified_type);

        let safe_field_name = sanitize_python_field_name(&field.name);

        if field.repeated {
            if type_annotation.starts_with("List[") && type_annotation.ends_with("]") {
                type_annotation = type_annotation[5..type_annotation.len() - 1].to_string();
            }
            type_annotation = format!("Optional[List[{}]]", type_annotation);
            params.push(format!("{}: {} = None", safe_field_name, type_annotation));
        } else if field.optional {
            if !type_annotation.starts_with("Optional[") {
                type_annotation = format!("Optional[{}]", type_annotation);
            }
            params.push(format!("{}: {} = None", safe_field_name, type_annotation));
        } else {
            params.push(format!("{}: {} = None", safe_field_name, type_annotation));
        }
    }

    for &i in &field_indices {
        let field = &message.fields[i];
        if let Some(variants) = &field.oneof_variants {
            let mut variant_indices: Vec<usize> = (0..variants.len()).collect();
            variant_indices.sort_by_key(|&j| &variants[j].field_name);
            for &j in &variant_indices {
                let variant = &variants[j];
                let safe_field_name = sanitize_python_field_name(&variant.field_name);
                let python_type = unified_to_python_type(&variant.field_type);
                params.push(format!(
                    "{}: Optional[{}] = None",
                    safe_field_name, python_type
                ));
            }
        }
    }

    generate_method_template("__init__", &params, "None", None, 1)
}

// --- Template helpers ---

fn clean_text(text: &str) -> String {
    dedent(text).trim().to_string()
}

fn generate_class_from_template(
    class_name: &str,
    class_type: &str,
    docstring: Option<&str>,
    body_content: &str,
) -> String {
    let template = if docstring.is_some() {
        format!(
            "class {}({}):\n    \"\"\"{}\"\"\"\n{}",
            class_name,
            class_type,
            docstring.unwrap_or(""),
            body_content
        )
    } else {
        format!("class {}({}):\n{}", class_name, class_type, body_content)
    };

    clean_text(&template)
}

fn generate_method_template(
    method_name: &str,
    params: &[String],
    return_type: &str,
    docstring: Option<&str>,
    indent_level: usize,
) -> String {
    let indent_str = "    ".repeat(indent_level);

    let params_str = if params.len() <= 3 {
        params.join(", ")
    } else {
        let param_indent = format!("{}    ", indent_str);
        let formatted_params: Vec<String> = params
            .iter()
            .map(|p| format!("{}{},", param_indent, p))
            .collect();
        format!("\n{}", formatted_params.join("\n").trim_end_matches(','))
    };

    let signature = if params.len() <= 3 {
        format!("def {}({}) -> {}:", method_name, params_str, return_type)
    } else {
        format!(
            "def {}(\n{}\n{}) -> {}:",
            method_name, params_str, indent_str, return_type
        )
    };

    let mut result = indent(&signature, &indent_str);

    if let Some(doc) = docstring {
        result.push('\n');
        let formatted_doc = format_method_docstring_for_template(doc, indent_level);
        result.push_str(&formatted_doc);
    }

    result.push_str(&format!("\n{}    ...", indent_str));
    result
}

fn format_method_docstring_for_template(docstring: &str, indent_level: usize) -> String {
    let base_indent = "    ".repeat(indent_level + 1);

    let mut sections = Vec::new();
    let mut current_section = Vec::new();
    let mut in_args_or_returns = false;

    for line in docstring.lines() {
        let trimmed = line.trim();
        if trimmed == "Args:" || trimmed == "Returns:" {
            if !current_section.is_empty() {
                sections.push((false, current_section.join("\n")));
                current_section.clear();
            }
            in_args_or_returns = true;
            current_section.push(line.to_string());
        } else if in_args_or_returns {
            current_section.push(line.to_string());
        } else {
            current_section.push(line.to_string());
        }
    }

    if !current_section.is_empty() {
        sections.push((in_args_or_returns, current_section.join("\n")));
    }

    let processed_sections: Vec<String> = sections
        .into_iter()
        .map(|(_is_args_returns, content)| content)
        .collect();

    let full_content = processed_sections.join("\n\n");

    let formatted_doc = if full_content.lines().count() == 1 {
        format!("\"\"\"{}\"\"\"", full_content)
    } else {
        let mut lines = vec!["\"\"\"".to_string()];
        for line in full_content.lines() {
            lines.push(line.to_string());
        }
        lines.push("\"\"\"".to_string());
        lines.join("\n")
    };

    indent(&formatted_doc, &base_indent)
}

fn format_field_docstring(description: &str) -> String {
    let base_indent = "    ";
    let available_width = DOCS_TARGET_WIDTH - base_indent.len() - 6;

    let optimally_filled = refill(description, available_width);

    let single_line_test = format!(
        "{}\"\"\"{}\"\"\"",
        base_indent,
        optimally_filled.replace('\n', " ")
    );
    if !optimally_filled.contains('\n') && single_line_test.len() <= DOCS_TARGET_WIDTH {
        return single_line_test;
    }

    let mut result = format!("{}\"\"\"", base_indent);
    for line in optimally_filled.lines() {
        if line.trim().is_empty() {
            result.push('\n');
        } else {
            result.push_str(&format!("\n{}{}", base_indent, line));
        }
    }
    result.push_str(&format!("\n{}\"\"\"", base_indent));
    result
}
