use std::collections::HashSet;
use std::fs;
use std::path::Path;

use glob::glob;
use protobuf::Message;
use serde_json::Map as JsonMap;
use serde_yaml::Value as YamlValue;

use crate::Result;
use crate::parsing::{CodeGenMetadata, parse_file_descriptor_set};

/// Run Pass 0 (gnostic ref fix), Pass 1 (validation enrichment), and optionally Pass 2 (path/body dedup).
pub fn run(
    spec: &Path,
    jsonschema_dir: &Path,
    camel_case: bool,
    descriptors: Option<&Path>,
) -> Result<()> {
    let spec_str = fs::read_to_string(spec)
        .map_err(|e| crate::Error::Build(format!("Failed to read {}: {}", spec.display(), e)))?;
    let mut doc: YamlValue = serde_yaml::from_str(&spec_str).map_err(|e| {
        crate::Error::Build(format!("Failed to parse YAML {}: {}", spec.display(), e))
    })?;

    fix_gnostic_refs(&mut doc);
    enrich_from_jsonschema(&mut doc, jsonschema_dir, camel_case)?;

    if let Some(desc_path) = descriptors {
        let bytes = fs::read(desc_path).map_err(|e| {
            crate::Error::Build(format!(
                "Failed to read descriptors {}: {}",
                desc_path.display(),
                e
            ))
        })?;
        let fds = protobuf::descriptor::FileDescriptorSet::parse_from_bytes(&bytes)
            .map_err(|e| crate::Error::Build(format!("Failed to parse descriptors: {}", e)))?;
        let metadata = parse_file_descriptor_set(&fds)?;
        dedup_path_params(&mut doc, &metadata);
    }

    let out = serde_yaml::to_string(&doc)
        .map_err(|e| crate::Error::Build(format!("Failed to serialize YAML: {}", e)))?;
    fs::write(spec, out)
        .map_err(|e| crate::Error::Build(format!("Failed to write {}: {}", spec.display(), e)))?;

    Ok(())
}

// ── Pass 0 ────────────────────────────────────────────────────────────────────

/// Rewrite gnostic-generated `#/$defs/a.b.v1.TypeName.schema.strict.json` refs to
/// valid OpenAPI `#/components/schemas/TypeName` refs throughout the document.
fn fix_gnostic_refs(doc: &mut YamlValue) {
    match doc {
        YamlValue::Mapping(map) => {
            if let Some(YamlValue::String(s)) = map.get_mut("$ref") {
                if let Some(fixed) = rewrite_gnostic_ref(s) {
                    *s = fixed;
                }
            }
            for (_, v) in map.iter_mut() {
                fix_gnostic_refs(v);
            }
        }
        YamlValue::Sequence(seq) => {
            for item in seq.iter_mut() {
                fix_gnostic_refs(item);
            }
        }
        _ => {}
    }
}

fn rewrite_gnostic_ref(ref_str: &str) -> Option<String> {
    let stem = ref_str
        .strip_prefix("#/$defs/")?
        .strip_suffix(".schema.strict.json")?;
    let start = stem.find(|c: char| c.is_uppercase())?;
    let type_name = &stem[start..];
    Some(format!("#/components/schemas/{type_name}"))
}

// ── Pass 1 ────────────────────────────────────────────────────────────────────

fn enrich_from_jsonschema(
    spec: &mut YamlValue,
    jsonschema_dir: &Path,
    camel_case: bool,
) -> Result<()> {
    let pattern = jsonschema_dir
        .join("*.schema.strict.bundle.json")
        .to_string_lossy()
        .into_owned();

    let mut files: Vec<std::path::PathBuf> = glob(&pattern)
        .map_err(|e| crate::Error::Build(format!("Glob pattern error: {}", e)))?
        .filter_map(|r: Result<std::path::PathBuf, _>| r.ok())
        .collect();
    files.sort();

    if files.is_empty() {
        eprintln!(
            "enrich-openapi: no JSON Schema files found in {}",
            jsonschema_dir.display()
        );
        return Ok(());
    }

    let mut updated = 0usize;
    let mut added = 0usize;

    for path in &files {
        let filename = path
            .file_name()
            .and_then(|f: &std::ffi::OsStr| f.to_str())
            .unwrap_or_default();
        let type_name = match type_name_from_filename(filename) {
            Some(n) => n,
            None => {
                eprintln!("enrich-openapi: skipping {filename} (cannot extract type name)");
                continue;
            }
        };

        let content = fs::read_to_string(path).map_err(|e| {
            crate::Error::Build(format!("Failed to read {}: {}", path.display(), e))
        })?;
        let bundle: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            crate::Error::Build(format!("Failed to parse JSON {}: {}", path.display(), e))
        })?;

        let defs: JsonMap<String, serde_json::Value> = bundle
            .get("$defs")
            .and_then(|v: &serde_json::Value| v.as_object())
            .cloned()
            .unwrap_or_default();

        let root_ref = bundle
            .get("$ref")
            .and_then(|v: &serde_json::Value| v.as_str())
            .unwrap_or("");
        let root_key = root_ref.strip_prefix("#/$defs/").unwrap_or("");
        let root_schema: serde_json::Value = match defs.get(root_key) {
            Some(s) => s.clone(),
            None => {
                eprintln!(
                    "enrich-openapi: could not resolve root $ref '{root_ref}' for {type_name}, skipping"
                );
                continue;
            }
        };

        // Navigate to components.schemas.<TypeName>
        let schemas = spec
            .get_mut("components")
            .and_then(|c| c.get_mut("schemas"));
        let schemas = match schemas {
            Some(s) => s,
            None => {
                eprintln!("enrich-openapi: openapi.yaml has no components.schemas, skipping");
                break;
            }
        };

        let exists = schemas
            .as_mapping()
            .map(|m| m.contains_key(type_name.as_str()))
            .unwrap_or(false);

        if !exists {
            let ty = root_schema
                .get("type")
                .and_then(|v: &serde_json::Value| v.as_str())
                .unwrap_or("object");
            if let Some(map) = schemas.as_mapping_mut() {
                let mut entry = serde_yaml::Mapping::new();
                entry.insert(
                    YamlValue::String("type".into()),
                    YamlValue::String(ty.to_string()),
                );
                map.insert(
                    YamlValue::String(type_name.clone()),
                    YamlValue::Mapping(entry),
                );
            }
            added += 1;
        } else {
            updated += 1;
        }

        let oa_schema = schemas
            .as_mapping_mut()
            .and_then(|m| m.get_mut(type_name.as_str()));
        if let Some(oa) = oa_schema {
            enrich_schema(oa, &root_schema, &defs, camel_case);
        }
    }

    println!(
        "enrich-openapi: enriched {} schemas ({} updated, {} added)",
        updated + added,
        updated,
        added
    );
    Ok(())
}

/// Extract type name from a JSON Schema bundle filename.
/// e.g. "dda.coordinator.v1.CreateWaveRequest.schema.strict.bundle.json" → "CreateWaveRequest"
fn type_name_from_filename(filename: &str) -> Option<String> {
    let stem = filename.strip_suffix(".schema.strict.bundle.json")?;
    let start = stem.find(|c: char| c.is_uppercase())?;
    Some(stem[start..].to_string())
}

/// Convert snake_case to camelCase.
fn snake_to_camel(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalise_next = false;
    for ch in s.chars() {
        if ch == '_' {
            capitalise_next = true;
        } else if capitalise_next {
            result.push(ch.to_ascii_uppercase());
            capitalise_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

/// Resolve a `#/$defs/<key>` reference within a bundle's $defs map.
fn resolve_ref<'a>(
    ref_str: &str,
    defs: &'a JsonMap<String, serde_json::Value>,
) -> Option<&'a serde_json::Value> {
    let key = ref_str.strip_prefix("#/$defs/")?;
    defs.get(key)
}

const VALIDATION_FIELDS: &[&str] = &[
    "minLength",
    "maxLength",
    "pattern",
    "minimum",
    "maximum",
    "exclusiveMinimum",
    "exclusiveMaximum",
    "minItems",
    "maxItems",
    "enum",
    "additionalProperties",
    "required",
    "description",
    "title",
];

/// Merge validation fields from a JSON Schema node into an OpenAPI YAML schema node.
fn merge_validation(source: &serde_json::Value, target: &mut YamlValue) {
    for &key in VALIDATION_FIELDS {
        let val = match source.get(key) {
            Some(v) => v,
            None => continue,
        };

        if key == "exclusiveMinimum" {
            if let Some(n) = val.as_f64() {
                // JSON Schema 2020-12 numeric form → OpenAPI 3.0 boolean form
                yaml_set(
                    target,
                    "minimum",
                    YamlValue::Number(serde_yaml::Number::from(n)),
                );
                yaml_set(target, "exclusiveMinimum", YamlValue::Bool(true));
            }
            continue;
        }

        if key == "exclusiveMaximum" {
            if let Some(n) = val.as_f64() {
                // JSON Schema 2020-12 numeric form → OpenAPI 3.0 boolean form
                yaml_set(
                    target,
                    "maximum",
                    YamlValue::Number(serde_yaml::Number::from(n)),
                );
                yaml_set(target, "exclusiveMaximum", YamlValue::Bool(true));
            }
            continue;
        }

        yaml_set(target, key, json_to_yaml(val));
    }
}

/// Recursively enrich an OpenAPI schema YAML node with validation from a JSON Schema node.
fn enrich_schema(
    openapi: &mut YamlValue,
    json_schema: &serde_json::Value,
    defs: &JsonMap<String, serde_json::Value>,
    camel_case: bool,
) {
    merge_validation(json_schema, openapi);

    // Recurse into properties
    if let Some(js_props) = json_schema.get("properties").and_then(|v| v.as_object()) {
        for (snake_key, js_prop) in js_props {
            let lookup_key = if camel_case {
                snake_to_camel(snake_key)
            } else {
                snake_key.clone()
            };

            let resolved: std::borrow::Cow<serde_json::Value> =
                if let Some(ref_str) = js_prop.get("$ref").and_then(|v| v.as_str()) {
                    match resolve_ref(ref_str, defs) {
                        Some(r) => std::borrow::Cow::Borrowed(r),
                        None => std::borrow::Cow::Borrowed(js_prop),
                    }
                } else {
                    std::borrow::Cow::Borrowed(js_prop)
                };

            if let Some(oa_prop) = openapi
                .get_mut("properties")
                .and_then(|p| p.get_mut(lookup_key.as_str()))
            {
                enrich_schema(oa_prop, &resolved, defs, camel_case);
            }
        }
    }

    // Recurse into items
    if let Some(js_items) = json_schema.get("items") {
        let resolved: std::borrow::Cow<serde_json::Value> =
            if let Some(ref_str) = js_items.get("$ref").and_then(|v| v.as_str()) {
                match resolve_ref(ref_str, defs) {
                    Some(r) => std::borrow::Cow::Borrowed(r),
                    None => std::borrow::Cow::Borrowed(js_items),
                }
            } else {
                std::borrow::Cow::Borrowed(js_items)
            };

        if let Some(oa_items) = openapi.get_mut("items") {
            enrich_schema(oa_items, &resolved, defs, camel_case);
        }
    }

    // Recurse into combiners
    for combiner in &["allOf", "oneOf", "anyOf"] {
        if let Some(js_list) = json_schema.get(combiner).and_then(|v| v.as_array()) {
            if let Some(oa_list) = openapi.get_mut(combiner).and_then(|v| v.as_sequence_mut()) {
                for (i, js_entry) in js_list.iter().enumerate() {
                    if i >= oa_list.len() {
                        break;
                    }
                    let resolved: std::borrow::Cow<serde_json::Value> =
                        if let Some(ref_str) = js_entry.get("$ref").and_then(|v| v.as_str()) {
                            match resolve_ref(ref_str, defs) {
                                Some(r) => std::borrow::Cow::Borrowed(r),
                                None => std::borrow::Cow::Borrowed(js_entry),
                            }
                        } else {
                            std::borrow::Cow::Borrowed(js_entry)
                        };
                    enrich_schema(&mut oa_list[i], &resolved, defs, camel_case);
                }
            }
        }
    }
}

// ── Pass 2 ────────────────────────────────────────────────────────────────────

fn dedup_path_params(spec: &mut YamlValue, metadata: &CodeGenMetadata) {
    let mut removed_total = 0usize;

    for service in metadata.services.values() {
        for method in &service.methods {
            let path_params: HashSet<String> =
                method.http_pattern.parameters.iter().cloned().collect();
            if path_params.is_empty() {
                continue;
            }

            let input_type = method
                .input_type
                .rfind('.')
                .map(|i| &method.input_type[i + 1..])
                .unwrap_or(&method.input_type);

            let schema = spec
                .get_mut("components")
                .and_then(|c| c.get_mut("schemas"))
                .and_then(|s| s.get_mut(input_type));

            let schema = match schema {
                Some(s) => s,
                None => continue,
            };

            // Remove from properties
            if let Some(props) = schema
                .get_mut("properties")
                .and_then(|p| p.as_mapping_mut())
            {
                let before = props.len();
                props.retain(|k, _| {
                    let key = k.as_str().unwrap_or("");
                    !path_params.contains(key)
                });
                removed_total += before - props.len();
            }

            // Remove from required array
            if let Some(required) = schema.get_mut("required").and_then(|r| r.as_sequence_mut()) {
                required.retain(|v| {
                    let key = v.as_str().unwrap_or("");
                    !path_params.contains(key)
                });
            }
        }
    }

    if removed_total > 0 {
        println!(
            "enrich-openapi: dedup removed {removed_total} path-bound field(s) from request body schemas"
        );
    }
}

// ── YAML helpers ──────────────────────────────────────────────────────────────

fn yaml_set(target: &mut YamlValue, key: &str, value: YamlValue) {
    if let YamlValue::Mapping(m) = target {
        m.insert(YamlValue::String(key.to_string()), value);
    }
}

fn json_to_yaml(v: &serde_json::Value) -> YamlValue {
    match v {
        serde_json::Value::Null => YamlValue::Null,
        serde_json::Value::Bool(b) => YamlValue::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                YamlValue::Number(serde_yaml::Number::from(i))
            } else if let Some(f) = n.as_f64() {
                YamlValue::Number(serde_yaml::Number::from(f))
            } else {
                YamlValue::String(n.to_string())
            }
        }
        serde_json::Value::String(s) => YamlValue::String(s.clone()),
        serde_json::Value::Array(arr) => {
            YamlValue::Sequence(arr.iter().map(json_to_yaml).collect())
        }
        serde_json::Value::Object(obj) => {
            let mut mapping = serde_yaml::Mapping::new();
            for (k, val) in obj {
                mapping.insert(YamlValue::String(k.clone()), json_to_yaml(val));
            }
            YamlValue::Mapping(mapping)
        }
    }
}
