use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use clap::{Args, Parser, Subcommand};
use protobuf::Message;
use protobuf::descriptor::FileDescriptorSet;

use unitycatalog_build::CodeGenMetadata;
use unitycatalog_build::codegen::generate_rest_handlers;
use unitycatalog_build::error::Result;
use unitycatalog_build::parsing::process_file_descriptor;

#[derive(Parser)]
struct Cli {
    #[clap(long, short, env = "UC_BUILD_OUTPUT")]
    output: String,

    #[clap(long, short, env = "UC_BUILD_DESCRIPTORS")]
    descriptors: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    // Load and parse file descriptors
    let descriptor_path = fs::canonicalize(PathBuf::from(args.descriptors))?;
    let descriptor_bytes = fs::read(&descriptor_path)?;
    let file_descriptor_set = FileDescriptorSet::parse_from_bytes(&descriptor_bytes)?;

    let mut codegen_metadata = CodeGenMetadata {
        methods: Vec::new(),
        messages: std::collections::HashMap::new(),
    };

    // Process each file descriptor
    for file_descriptor in &file_descriptor_set.file {
        process_file_descriptor(file_descriptor, &mut codegen_metadata)?;
    }

    // Generate code from collected metadata
    let output_dir = fs::canonicalize(PathBuf::from(&args.output))?;
    generate_rest_handlers(&codegen_metadata, &output_dir)?;

    Ok(())
}
