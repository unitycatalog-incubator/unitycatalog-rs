use std::fs;
use std::path::PathBuf;

use clap::Parser;
use protobuf::Message;
use protobuf::descriptor::FileDescriptorSet;

use unitycatalog_build::Result;
use unitycatalog_build::codegen::generate_code;
use unitycatalog_build::parsing::parse_file_descriptor_set;
use unitycatalog_build::{CodeGenConfig, CodeGenOutput};

#[derive(Parser)]
struct Cli {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    // Load and parse file descriptors
    let descriptor_path = fs::canonicalize(PathBuf::from(args.descriptors))?;
    let descriptor_bytes = fs::read(&descriptor_path)?;
    let file_descriptor_set = FileDescriptorSet::parse_from_bytes(&descriptor_bytes)?;

    let codegen_metadata = parse_file_descriptor_set(&file_descriptor_set)?;

    // Resolve output directories
    let output_common = fs::canonicalize(PathBuf::from(&args.output_common))?;
    let output_models_gen = args
        .output_models_gen
        .as_ref()
        .map(|p| fs::canonicalize(PathBuf::from(p)))
        .transpose()?;
    let output_server = args
        .output_server
        .as_ref()
        .map(|p| fs::canonicalize(PathBuf::from(p)))
        .transpose()?;
    let output_client = args
        .output_client
        .as_ref()
        .map(|p| fs::canonicalize(PathBuf::from(p)))
        .transpose()?;
    let output_python = args
        .output_python
        .as_ref()
        .map(|p| fs::canonicalize(PathBuf::from(p)))
        .transpose()?;
    let output_node = args
        .output_node
        .as_ref()
        .map(|p| fs::canonicalize(PathBuf::from(p)))
        .transpose()?;
    let output_node_ts = args
        .output_node_ts
        .as_ref()
        .map(|p| fs::canonicalize(PathBuf::from(p)))
        .transpose()?;

    let output = CodeGenOutput {
        common: output_common,
        models_gen: output_models_gen,
        server: output_server,
        client: output_client,
        python: output_python,
        node: output_node,
        node_ts: output_node_ts,
        python_typings_filename: args
            .python_typings_filename
            .unwrap_or_else(|| "unitycatalog_client.pyi".to_string()),
    };

    let mut config = CodeGenConfig::unitycatalog_defaults(output);

    if let Some(v) = args.context_type {
        config.context_type_path = v;
    }
    if let Some(v) = args.result_type {
        config.result_type_path = v;
    }
    if let Some(v) = args.models_path_template {
        config.models_path_template = v;
    }
    if let Some(v) = args.models_path_crate_template {
        config.models_path_crate_template = v;
    }

    generate_code(&codegen_metadata, &config)?;

    Ok(())
}
