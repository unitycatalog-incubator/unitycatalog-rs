use std::fs;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use protobuf::Message;
use protobuf::descriptor::FileDescriptorSet;

use proto_gen::error::Result;
use proto_gen::{
    BindingsConfig, CodeGenConfig, CodeGenOutput, ResourceEnumConfig, enrich_openapi,
    generate_code, parse_file_descriptor_set,
};

#[derive(Parser)]
#[command(
    name = "proto-gen",
    about = "Generate Rust/Python/Node.js code from proto descriptors"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Rust/Python/Node.js code from proto descriptor
    Generate(GenerateArgs),
    /// Enrich openapi.yaml with validation rules from buf JSON Schema files
    EnrichOpenapi(EnrichOpenApiArgs),
}

#[derive(Args)]
struct GenerateArgs {
    #[clap(long, env = "UC_BUILD_OUTPUT_COMMON")]
    output_common: String,

    #[clap(long, env = "UC_BUILD_OUTPUT_MODELS_GEN")]
    output_models_gen: Option<String>,

    #[clap(long, env = "UC_BUILD_OUTPUT_SERVER")]
    output_server: Option<String>,

    #[clap(long, env = "UC_BUILD_OUTPUT_CLIENT")]
    output_client: Option<String>,

    #[clap(long, env = "UC_BUILD_OUTPUT_PYTHON")]
    output_python: Option<String>,

    #[clap(long, env = "UC_BUILD_OUTPUT_NODE")]
    output_node: Option<String>,

    #[clap(long, env = "UC_BUILD_OUTPUT_NODE_TS")]
    output_node_ts: Option<String>,

    #[clap(long, short, env = "UC_BUILD_DESCRIPTORS")]
    descriptors: String,

    /// Fully-qualified path to the request context type (e.g. `my_crate::Context`).
    #[clap(long, env = "UC_BUILD_CONTEXT_TYPE")]
    context_type: Option<String>,

    /// Fully-qualified path to the Result alias (e.g. `my_crate::Result`).
    #[clap(long, env = "UC_BUILD_RESULT_TYPE")]
    result_type: Option<String>,

    /// Template for the external model import path. Use `{service}` as placeholder.
    #[clap(long, env = "UC_BUILD_MODELS_PATH_TEMPLATE")]
    models_path_template: Option<String>,

    /// Template for the crate-local model import path. Use `{service}` as placeholder.
    #[clap(long, env = "UC_BUILD_MODELS_PATH_CRATE_TEMPLATE")]
    models_path_crate_template: Option<String>,

    /// Filename for the generated Python typings stub.
    #[clap(long, env = "UC_BUILD_PYTHON_TYPINGS_FILENAME")]
    python_typings_filename: Option<String>,
}

#[derive(Args)]
struct EnrichOpenApiArgs {
    /// Path to openapi.yaml
    #[arg(long, default_value = "openapi/openapi.yaml")]
    spec: PathBuf,
    /// Directory containing *.schema.strict.bundle.json files
    #[arg(long, default_value = "openapi/jsonschema")]
    jsonschema_dir: PathBuf,
    /// Proto descriptor binary for path/body deduplication (Pass 2); omit to skip
    #[arg(long)]
    descriptors: Option<PathBuf>,
    /// Translate snake_case JSON Schema property names to camelCase in OpenAPI
    #[arg(long, default_value_t = false)]
    camel_case: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate(args) => run_generate(args),
        Commands::EnrichOpenapi(args) => {
            enrich_openapi(
                &args.spec,
                &args.jsonschema_dir,
                args.camel_case,
                args.descriptors.as_deref(),
            )?;
            Ok(())
        }
    }
}

/// Build a [`CodeGenConfig`] with Unity Catalog defaults from parsed CLI arguments.
///
/// Mirrors the config constructed by `crates/build/src/main.rs` so that both binaries
/// produce identical output and can be diffed for verification.
fn make_uc_config(output: CodeGenOutput, args: &GenerateArgs) -> CodeGenConfig {
    let has_bindings_output =
        output.python.is_some() || output.node.is_some() || output.node_ts.is_some();

    let bindings = has_bindings_output.then(|| BindingsConfig {
        aggregate_client_name: "UnityCatalogClient".to_string(),
        client_crate_name: "unitycatalog_client".to_string(),
        py_error_type: "PyUnityCatalogError".to_string(),
        py_result_type: "PyUnityCatalogResult".to_string(),
        napi_error_ext_trait: "NapiErrorExt".to_string(),
        typings_package_filter: Some("unitycatalog".to_string()),
        ts_error_base_class: "UnityCatalogError".to_string(),
        ts_error_code_prefix: "UC".to_string(),
    });

    let mut config = CodeGenConfig {
        context_type_path: "crate::api::RequestContext".to_string(),
        result_type_path: "crate::Result".to_string(),
        models_path_template: "unitycatalog_common::models::{service}::v1".to_string(),
        models_path_crate_template: "crate::models::{service}::v1".to_string(),
        output,
        resource_enum: Some(ResourceEnumConfig {
            package_prefix: ".unitycatalog.".to_string(),
            super_levels: 2,
        }),
        bindings,
    };

    if let Some(ref v) = args.context_type {
        config.context_type_path = v.clone();
    }
    if let Some(ref v) = args.result_type {
        config.result_type_path = v.clone();
    }
    if let Some(ref v) = args.models_path_template {
        config.models_path_template = v.clone();
    }
    if let Some(ref v) = args.models_path_crate_template {
        config.models_path_crate_template = v.clone();
    }

    config
}

fn run_generate(args: GenerateArgs) -> Result<(), Box<dyn std::error::Error>> {
    let descriptor_path = fs::canonicalize(PathBuf::from(&args.descriptors))?;
    let descriptor_bytes = fs::read(&descriptor_path)?;
    let file_descriptor_set = FileDescriptorSet::parse_from_bytes(&descriptor_bytes)?;

    let metadata = parse_file_descriptor_set(&file_descriptor_set)?;

    let resolve_dir = |p: &str| fs::canonicalize(PathBuf::from(p));

    let output_common = resolve_dir(&args.output_common)?;
    let output_models_gen = args
        .output_models_gen
        .as_deref()
        .map(resolve_dir)
        .transpose()?;
    let output_server = args.output_server.as_deref().map(resolve_dir).transpose()?;
    let output_client = args.output_client.as_deref().map(resolve_dir).transpose()?;
    let output_python = args.output_python.as_deref().map(resolve_dir).transpose()?;
    let output_node = args.output_node.as_deref().map(resolve_dir).transpose()?;
    let output_node_ts = args
        .output_node_ts
        .as_deref()
        .map(resolve_dir)
        .transpose()?;

    let python_typings_filename = args
        .python_typings_filename
        .clone()
        .unwrap_or_else(|| "unitycatalog_client.pyi".to_string());

    let output = CodeGenOutput {
        common: output_common,
        models_gen: output_models_gen,
        server: output_server,
        client: output_client,
        python: output_python,
        node: output_node,
        node_ts: output_node_ts,
        python_typings_filename,
    };

    let config = make_uc_config(output, &args);
    generate_code(&metadata, &config)?;

    Ok(())
}
