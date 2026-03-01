use std::fs;
use std::path::PathBuf;

use clap::Parser;
use protobuf::Message;
use protobuf::descriptor::FileDescriptorSet;

use unitycatalog_build::Result;
use unitycatalog_build::codegen::generate_code;
use unitycatalog_build::parsing::parse_file_descriptor_set;

#[derive(Parser)]
struct Cli {
    #[clap(long, env = "UC_BUILD_OUTPUT_COMMON")]
    output_common: String,

    #[clap(long, env = "UC_BUILD_OUTPUT_SERVER")]
    output_server: String,

    #[clap(long, env = "UC_BUILD_OUTPUT_CLIENT")]
    output_client: String,

    #[clap(long, env = "UC_BUILD_OUTPUT_PYTHON")]
    output_python: String,

    #[clap(long, env = "UC_BUILD_OUTPUT_NODE")]
    output_node: Option<String>,

    #[clap(long, env = "UC_BUILD_OUTPUT_NODE_TS")]
    output_node_ts: Option<String>,

    #[clap(long, short, env = "UC_BUILD_DESCRIPTORS")]
    descriptors: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    // Load and parse file descriptors
    let descriptor_path = fs::canonicalize(PathBuf::from(args.descriptors))?;
    let descriptor_bytes = fs::read(&descriptor_path)?;
    let file_descriptor_set = FileDescriptorSet::parse_from_bytes(&descriptor_bytes)?;

    let codegen_metadata = parse_file_descriptor_set(&file_descriptor_set)?;

    // Generate code from collected metadata
    let output_dir_common = fs::canonicalize(PathBuf::from(&args.output_common))?;
    let output_dir_server = fs::canonicalize(PathBuf::from(&args.output_server))?;
    let output_dir_client = fs::canonicalize(PathBuf::from(&args.output_client))?;
    let output_dir_python = fs::canonicalize(PathBuf::from(&args.output_python))?;

    let output_dir_node = args
        .output_node
        .as_ref()
        .map(|p| fs::canonicalize(PathBuf::from(p)))
        .transpose()?;
    let output_dir_node_ts = args
        .output_node_ts
        .as_ref()
        .map(|p| fs::canonicalize(PathBuf::from(p)))
        .transpose()?;

    generate_code(
        &codegen_metadata,
        &output_dir_common,
        &output_dir_server,
        &output_dir_client,
        &output_dir_python,
        output_dir_node.as_deref(),
        output_dir_node_ts.as_deref(),
    )?;

    Ok(())
}
