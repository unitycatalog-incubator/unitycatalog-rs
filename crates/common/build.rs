//! Unity Catalog Build Script
//!
//! This build script processes protobuf file descriptors to extract gnostic annotations
//! and generate Rust code for REST API handlers. It uses the `protobuf` crate to access
//! extension fields that contain Google API and gnostic OpenAPI annotations.
//!
//! ## Extension Fields Accessed
//!
//! - `google.api.http` (field 72295728): HTTP method and path information
//! - `gnostic.openapi.v3.operation` (field 1042): OpenAPI operation metadata including operation_id
//!
//! ## How to Extend
//!
//! ### Adding New Extension Parsing
//! 1. Find the extension field number in the protobuf definition
//! 2. Add parsing logic in the relevant extract_*_annotations function
//! 3. Decode the binary extension data using appropriate message type
//!
//! ### Adding Code Generation
//! 1. Collect extracted metadata in structured format
//! 2. Create template files for handler traits and implementations
//! 3. Write generated code to appropriate output directory

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use protobuf::Message;
use protobuf::descriptor::FileDescriptorSet;
use unitycatalog_build::CodeGenMetadata;
use unitycatalog_build::codegen::generate_rest_handlers;
use unitycatalog_build::parsing::process_file_descriptor;

/// Main build script entry point
///
/// Loads protobuf file descriptors and processes them to extract gnostic annotations
/// for code generation. Uses protobuf crate for extension field access.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../../proto");
    println!("cargo:rerun-if-changed=descriptors");

    // Path where buf will generate the file descriptors
    let descriptor_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .join("descriptors")
        .join("descriptors.bin");

    // Check if descriptor file exists
    if !descriptor_path.exists() {
        println!(
            "cargo:warning=File descriptor set not found at {}. Skipping codegen.",
            descriptor_path.display()
        );
        println!(
            "cargo:warning=Run `cd proto && buf build --output ../crates/common/descriptors/descriptors.bin` to generate descriptors."
        );
        return Ok(());
    }

    // Load and parse file descriptors
    let descriptor_bytes = fs::read(&descriptor_path)?;
    let file_descriptor_set = FileDescriptorSet::parse_from_bytes(&descriptor_bytes)?;

    println!(
        "cargo:warning=Loaded {} file descriptors",
        file_descriptor_set.file.len()
    );

    let mut codegen_metadata = CodeGenMetadata {
        methods: Vec::new(),
        messages: std::collections::HashMap::new(),
    };

    // Process each file descriptor
    for file_descriptor in &file_descriptor_set.file {
        process_file_descriptor(file_descriptor, &mut codegen_metadata)?;
    }

    // Summary of collected metadata
    println!(
        "cargo:warning=Collected metadata for {} methods across all services",
        codegen_metadata.methods.len()
    );

    // Generate code from collected metadata
    let output_dir = PathBuf::from(&format!("{}/src/codegen/", env!("CARGO_MANIFEST_DIR")));
    generate_rest_handlers(&codegen_metadata, &output_dir)?;

    // Format generated code with rustfmt
    format_generated_code(&output_dir)?;

    Ok(())
}

/// Format all generated Rust files with rustfmt
fn format_generated_code(output_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:warning=Formatting generated code with rustfmt...");

    // Find all .rs files in the output directory recursively
    let mut rust_files = Vec::new();
    collect_rust_files(output_dir, &mut rust_files)?;

    if rust_files.is_empty() {
        println!("cargo:warning=No Rust files found to format");
        return Ok(());
    }

    // Run rustfmt on all collected files with Rust 2021 edition
    let mut cmd = Command::new("rustfmt");
    cmd.arg("--edition").arg("2021");
    cmd.args(&rust_files);

    match cmd.output() {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("cargo:warning=rustfmt failed: {}", stderr);
            } else {
                println!(
                    "cargo:warning=Successfully formatted {} Rust files",
                    rust_files.len()
                );
            }
        }
        Err(e) => {
            println!(
                "cargo:warning=Failed to run rustfmt: {}. Generated code will not be formatted.",
                e
            );
        }
    }

    Ok(())
}

/// Recursively collect all .rs files in a directory
fn collect_rust_files(
    dir: &PathBuf,
    rust_files: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.exists() || !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_rust_files(&path, rust_files)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            rust_files.push(path);
        }
    }

    Ok(())
}
