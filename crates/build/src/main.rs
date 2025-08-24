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

    generate_code(
        &codegen_metadata,
        &output_dir_common,
        &output_dir_server,
        &output_dir_client,
        &output_dir_python,
    )?;

    Ok(())
}
