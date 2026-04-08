use std::path::PathBuf;

use proto_gen::parsing::parse_file_descriptor_set;
use proto_gen::{BindingsConfig, CodeGenConfig, CodeGenOutput, generate_code};
use protobuf::Message;
use protobuf::descriptor::FileDescriptorSet;
use tempfile::TempDir;

fn load_descriptor() -> FileDescriptorSet {
    let bytes = include_bytes!("../proto/example.bin");
    FileDescriptorSet::parse_from_bytes(bytes).expect("valid descriptor")
}

fn make_test_config(
    common: PathBuf,
    node_ts: PathBuf,
    python: PathBuf,
    node: PathBuf,
) -> CodeGenConfig {
    CodeGenConfig {
        context_type_path: "crate::Context".to_string(),
        result_type_path: "crate::Result".to_string(),
        models_path_template: "example_common::models::{service}::v1".to_string(),
        models_path_crate_template: "crate::models::{service}::v1".to_string(),
        output: CodeGenOutput {
            common,
            models: None,
            models_subdir: "_gen".to_string(),
            server: None,
            client: None,
            python: Some(python),
            node: Some(node),
            node_ts: Some(node_ts),
            python_typings_filename: "example_client.pyi".to_string(),
        },
        generate_resource_enum: false,
        error_type_path: None,
        generate_object_conversions: false,
        bindings: Some(BindingsConfig {
            aggregate_client_name: "ExampleClient".to_string(),
            client_crate_name: "example_client".to_string(),
            py_error_type: "PyExampleError".to_string(),
            py_result_type: "PyExampleResult".to_string(),
            napi_error_ext_trait: "NapiErrorExt".to_string(),
            typings_package_filter: Some(".example.".to_string()),
            ts_error_base_class: "ExampleError".to_string(),
            ts_error_code_prefix: "EX".to_string(),
        }),
        models_gen_dir: None,
    }
}

fn collect_generated_files(dir: &std::path::Path) -> Vec<String> {
    let mut contents = Vec::new();
    for entry in walkdir(dir) {
        if entry.is_file() {
            if let Ok(text) = std::fs::read_to_string(&entry) {
                contents.push(text);
            }
        }
    }
    contents
}

fn walkdir(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walkdir(&path));
            } else {
                files.push(path);
            }
        }
    }
    files
}

#[test]
fn test_codegen_produces_no_unitycatalog_strings() {
    let descriptor = load_descriptor();
    let metadata = parse_file_descriptor_set(&descriptor).expect("parse succeeded");

    let tmp = TempDir::new().expect("tempdir");
    let common_dir = tmp.path().join("common");
    let node_ts_dir = tmp.path().join("node_ts");
    let python_dir = tmp.path().join("python");
    let node_dir = tmp.path().join("node");

    for dir in &[&common_dir, &node_ts_dir, &python_dir, &node_dir] {
        std::fs::create_dir_all(dir).expect("create dir");
    }

    let config = make_test_config(
        common_dir.clone(),
        node_ts_dir.clone(),
        python_dir.clone(),
        node_dir.clone(),
    );

    generate_code(&metadata, &config).expect("generate_code succeeded");

    let all_dirs = [&common_dir, &node_ts_dir, &python_dir, &node_dir];
    let all_files: Vec<String> = all_dirs
        .iter()
        .flat_map(|d| collect_generated_files(d))
        .collect();

    assert!(
        !all_files.is_empty(),
        "expected generated files to be written"
    );

    for content in &all_files {
        let lower = content.to_lowercase();
        assert!(
            !lower.contains("unitycatalog"),
            "generated output contains 'unitycatalog': {:.200}",
            content
        );
        assert!(
            !lower.contains("pyunitycatalog"),
            "generated output contains 'pyunitycatalog'"
        );
        assert!(
            !lower.contains("napiunitycatalog"),
            "generated output contains 'napiunitycatalog'"
        );
    }
}

#[test]
fn test_codegen_uses_configured_error_base_class() {
    let descriptor = load_descriptor();
    let metadata = parse_file_descriptor_set(&descriptor).expect("parse succeeded");

    let tmp = TempDir::new().expect("tempdir");
    let common_dir = tmp.path().join("common");
    let node_ts_dir = tmp.path().join("node_ts");
    let python_dir = tmp.path().join("python");
    let node_dir = tmp.path().join("node");

    for dir in &[&common_dir, &node_ts_dir, &python_dir, &node_dir] {
        std::fs::create_dir_all(dir).expect("create dir");
    }

    let config = make_test_config(common_dir, node_ts_dir.clone(), python_dir, node_dir);

    generate_code(&metadata, &config).expect("generate_code succeeded");

    let ts_files = collect_generated_files(&node_ts_dir);
    assert!(!ts_files.is_empty(), "expected TypeScript files");

    let client_ts = ts_files
        .iter()
        .find(|f| f.contains("ExampleError"))
        .expect("expected ExampleError in TypeScript output");

    assert!(
        client_ts.contains("ExampleError"),
        "TypeScript output should use configured error base class"
    );
    assert!(
        client_ts.contains("EX:"),
        "TypeScript output should use configured error code prefix in regex"
    );
}

#[test]
fn test_codegen_uses_configured_aggregate_client_name() {
    let descriptor = load_descriptor();
    let metadata = parse_file_descriptor_set(&descriptor).expect("parse succeeded");

    let tmp = TempDir::new().expect("tempdir");
    let common_dir = tmp.path().join("common");
    let node_ts_dir = tmp.path().join("node_ts");
    let python_dir = tmp.path().join("python");
    let node_dir = tmp.path().join("node");

    for dir in &[&common_dir, &node_ts_dir, &python_dir, &node_dir] {
        std::fs::create_dir_all(dir).expect("create dir");
    }

    let config = make_test_config(
        common_dir,
        node_ts_dir.clone(),
        python_dir.clone(),
        node_dir,
    );

    generate_code(&metadata, &config).expect("generate_code succeeded");

    let ts_files = collect_generated_files(&node_ts_dir);
    let all_ts = ts_files.join("\n");
    assert!(
        all_ts.contains("ExampleClient"),
        "TypeScript output should use configured aggregate client name"
    );

    let py_files = collect_generated_files(&python_dir);
    let all_py = py_files.join("\n");
    assert!(
        all_py.contains("ExampleClient"),
        "Python output should use configured aggregate client name"
    );
}
