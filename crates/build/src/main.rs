use std::fs;
use std::path::PathBuf;

use clap::Parser;
use protobuf::Message;
use protobuf::descriptor::FileDescriptorSet;

use unitycatalog_build::CodeGenMetadata;
use unitycatalog_build::codegen::generate_rest_handlers;
use unitycatalog_build::error::Result;
use unitycatalog_build::parsing::process_file_descriptor;

#[derive(Parser)]
struct Cli {
    #[clap(long, env = "UC_BUILD_OUTPUT_SERVER")]
    output_server: String,

    #[clap(long, env = "UC_BUILD_OUTPUT_CLIENT")]
    output_client: String,

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

    println!("server -> {}", args.output_server);
    println!("client -> {}", args.output_client);

    // Generate code from collected metadata
    // let output_dir_server = fs::canonicalize(PathBuf::from(&args.output_server))?;
    let output_dir_server = PathBuf::from(&args.output_server);
    println!("server -> {}", output_dir_server.display());

    let output_dir_client = fs::canonicalize(PathBuf::from(&args.output_client))?;
    println!("client -> {}", output_dir_client.display());

    generate_rest_handlers(&codegen_metadata, &output_dir_server, &output_dir_client)?;

    Ok(())
}
