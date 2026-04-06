use std::fs;
use std::path::{Path, PathBuf};

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
    Generate(Box<GenerateArgs>),
    /// Enrich openapi.yaml with validation rules from buf JSON Schema files
    EnrichOpenapi(EnrichOpenApiArgs),
}

#[derive(Args)]
struct GenerateArgs {
    /// Path to a YAML config file; CLI flags override values from the file.
    #[clap(long, short = 'c')]
    config: Option<PathBuf>,

    #[clap(long, env = "UC_BUILD_OUTPUT_COMMON")]
    output_common: Option<String>,

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
    descriptors: Option<String>,

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
    /// Path to a YAML config file; CLI flags override values from the file.
    #[arg(long, short = 'c')]
    config: Option<PathBuf>,

    /// Path to openapi.yaml
    #[arg(long)]
    spec: Option<PathBuf>,
    /// Directory containing *.schema.strict.bundle.json files
    #[arg(long)]
    jsonschema_dir: Option<PathBuf>,
    /// Proto descriptor binary for path/body deduplication (Pass 2); omit to skip
    #[arg(long)]
    descriptors: Option<PathBuf>,
    /// Translate snake_case JSON Schema property names to camelCase in OpenAPI
    #[arg(long)]
    camel_case: Option<bool>,
}

// ---------------------------------------------------------------------------
// Config file types
// ---------------------------------------------------------------------------

/// Top-level config file schema. Both subcommands share a single file, each
/// with its own optional section. CLI flags always override config file values.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtoGenConfig {
    generate: Option<FileGenerateConfig>,
    enrich_openapi: Option<FileEnrichOpenApiConfig>,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FileGenerateConfig {
    output_common: Option<String>,
    output_models_gen: Option<String>,
    output_server: Option<String>,
    output_client: Option<String>,
    output_python: Option<String>,
    output_node: Option<String>,
    output_node_ts: Option<String>,
    descriptors: Option<String>,
    context_type: Option<String>,
    result_type: Option<String>,
    models_path_template: Option<String>,
    models_path_crate_template: Option<String>,
    python_typings_filename: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FileEnrichOpenApiConfig {
    spec: Option<PathBuf>,
    jsonschema_dir: Option<PathBuf>,
    descriptors: Option<PathBuf>,
    camel_case: Option<bool>,
}

fn load_proto_gen_config(
    path: &Path,
) -> std::result::Result<ProtoGenConfig, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&text)?)
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate(args) => run_generate(*args),
        Commands::EnrichOpenapi(args) => run_enrich_openapi(args),
    }
}

// ---------------------------------------------------------------------------
// generate subcommand
// ---------------------------------------------------------------------------

fn run_generate(mut args: GenerateArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Merge config file values (CLI flags win — only fill in fields not already set).
    if let Some(ref config_path) = args.config.clone() {
        let cfg = load_proto_gen_config(config_path)?
            .generate
            .unwrap_or_default();

        macro_rules! fill {
            ($field:ident) => {
                if args.$field.is_none() {
                    args.$field = cfg.$field;
                }
            };
        }

        fill!(output_common);
        fill!(output_models_gen);
        fill!(output_server);
        fill!(output_client);
        fill!(output_python);
        fill!(output_node);
        fill!(output_node_ts);
        fill!(descriptors);
        fill!(context_type);
        fill!(result_type);
        fill!(models_path_template);
        fill!(models_path_crate_template);
        fill!(python_typings_filename);
    }

    let descriptors = args.descriptors.as_deref().ok_or(
        "required argument missing: --descriptors (or set it in the config file under generate.descriptors)",
    )?.to_owned();
    let output_common = args.output_common.as_deref().ok_or(
        "required argument missing: --output-common (or set it in the config file under generate.output_common)",
    )?.to_owned();

    let descriptor_path = fs::canonicalize(PathBuf::from(&descriptors))?;
    let descriptor_bytes = fs::read(&descriptor_path)?;
    let file_descriptor_set = FileDescriptorSet::parse_from_bytes(&descriptor_bytes)?;

    let metadata = parse_file_descriptor_set(&file_descriptor_set)?;

    let resolve_dir = |p: &str| fs::canonicalize(PathBuf::from(p));

    let output_common = resolve_dir(&output_common)?;
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

// ---------------------------------------------------------------------------
// enrich-openapi subcommand
// ---------------------------------------------------------------------------

fn run_enrich_openapi(mut args: EnrichOpenApiArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Merge config file values (CLI flags win).
    if let Some(ref config_path) = args.config.clone() {
        let cfg = load_proto_gen_config(config_path)?
            .enrich_openapi
            .unwrap_or_default();

        macro_rules! fill {
            ($field:ident) => {
                if args.$field.is_none() {
                    args.$field = cfg.$field;
                }
            };
        }

        fill!(spec);
        fill!(jsonschema_dir);
        fill!(descriptors);
        fill!(camel_case);
    }

    // Apply defaults for fields not set by either CLI or config.
    let spec = args
        .spec
        .unwrap_or_else(|| PathBuf::from("openapi/openapi.yaml"));
    let jsonschema_dir = args
        .jsonschema_dir
        .unwrap_or_else(|| PathBuf::from("openapi/jsonschema"));
    let camel_case = args.camel_case.unwrap_or(false);

    enrich_openapi(
        &spec,
        &jsonschema_dir,
        camel_case,
        args.descriptors.as_deref(),
    )?;

    Ok(())
}
