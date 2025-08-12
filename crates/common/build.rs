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

use prost::Message as _;
use protobuf::Message;
use protobuf::descriptor::{
    DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet,
    MethodDescriptorProto, ServiceDescriptorProto,
};
use unitycatalog_build::gnostic::openapi::v3::Operation;
use unitycatalog_build::google::api::HttpRule;
use unitycatalog_build::{CodeGenMetadata, MethodMetadata};

// Known extension field numbers
const GOOGLE_API_HTTP_EXTENSION: u32 = 72295728; // google.api.http
const GNOSTIC_OPERATION_EXTENSION: u32 = 1143; // gnostic.openapi.v3.operation

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

    // TODO: Generate code from collected metadata
    // generate_handler_code(&codegen_metadata)?;

    Ok(())
}

/// Process a single protobuf file descriptor
///
/// Extracts all messages, services, and annotations from the file.
/// Collects metadata for code generation.
fn process_file_descriptor(
    file_desc: &FileDescriptorProto,
    codegen_metadata: &mut CodeGenMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = file_desc.name();
    println!("cargo:warning=Processing file: {}", file_name);

    // Extract gnostic file-level annotations
    extract_file_annotations(file_desc)?;

    // Process messages in the file
    for message in &file_desc.message_type {
        process_message(message, file_name)?;
    }

    // Process services in the file
    for service in &file_desc.service {
        process_service(service, file_name, codegen_metadata)?;
    }

    // Print summary for this file
    if !file_desc.service.is_empty() {
        println!(
            "cargo:warning=File {} contains {} services with {} total methods",
            file_name,
            file_desc.service.len(),
            file_desc
                .service
                .iter()
                .map(|s| s.method.len())
                .sum::<usize>()
        );
    }

    Ok(())
}

/// Extract gnostic annotations from file-level options
fn extract_file_annotations(
    file_desc: &FileDescriptorProto,
) -> Result<(), Box<dyn std::error::Error>> {
    if file_desc.options.is_some() {
        // Check for extension fields in file options
        let options = file_desc.options.as_ref().unwrap();
        let unknown_fields = options.unknown_fields();
        let field_count = unknown_fields.iter().count();
        if field_count > 0 {
            println!(
                "cargo:warning=File {} has {} extension fields",
                file_desc.name(),
                field_count
            );

            for (field_number, field_value) in unknown_fields.iter() {
                let data = match field_value {
                    protobuf::UnknownValueRef::LengthDelimited(bytes) => bytes,
                    _ => {
                        println!(
                            "cargo:warning=    Skipping non-length-delimited field {}",
                            field_number
                        );
                        continue;
                    }
                };
                println!(
                    "cargo:warning=  Extension field {}: {} bytes",
                    field_number,
                    data.len()
                );

                // TODO: Parse known extension fields
                // if field_number == SOME_FILE_EXTENSION {
                //     parse_file_extension(data)?;
                // }
            }
        }
    }
    Ok(())
}

/// Process a protobuf message definition
fn process_message(
    message: &DescriptorProto,
    _file_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let message_name = message.name();

    // Extract gnostic message-level annotations
    extract_message_annotations(message, message_name)?;

    // Process fields in the message
    for field in &message.field {
        process_field(field, message_name)?;
    }

    // Process nested messages
    for nested_message in &message.nested_type {
        process_message(nested_message, _file_name)?;
    }

    Ok(())
}

/// Extract gnostic annotations from message-level options
fn extract_message_annotations(
    message: &DescriptorProto,
    message_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if message.options.is_some() {
        let options = message.options.as_ref().unwrap();
        let unknown_fields = options.unknown_fields();

        let field_count = unknown_fields.iter().count();
        if field_count > 0 {
            println!(
                "cargo:warning=Message {} has {} extension fields",
                message_name, field_count
            );
        }
    }
    Ok(())
}

/// Process a message field definition
fn process_field(
    field: &FieldDescriptorProto,
    message_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let field_name = field.name();

    // Extract gnostic field-level annotations
    extract_field_annotations(field, message_name, field_name)?;

    Ok(())
}

/// Extract gnostic annotations from field-level options
fn extract_field_annotations(
    field: &FieldDescriptorProto,
    message_name: &str,
    field_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if field.options.is_some() {
        let options = field.options.as_ref().unwrap();
        let unknown_fields = options.unknown_fields();

        let field_count = unknown_fields.iter().count();
        if field_count > 0 {
            println!(
                "cargo:warning=Field {}.{} has {} extension fields",
                message_name, field_name, field_count
            );
        }
    }
    Ok(())
}

/// Process a gRPC service definition
fn process_service(
    service: &ServiceDescriptorProto,
    _file_name: &str,
    codegen_metadata: &mut CodeGenMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    let service_name = service.name();

    println!(
        "cargo:warning=Processing service {} with {} methods",
        service_name,
        service.method.len()
    );

    // Extract gnostic service-level annotations
    extract_service_annotations(service, service_name)?;

    // Process methods in the service
    for method in &service.method {
        process_method(method, service_name, codegen_metadata)?;
    }

    Ok(())
}

/// Extract gnostic annotations from service-level options
fn extract_service_annotations(
    service: &ServiceDescriptorProto,
    service_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if service.options.is_some() {
        let options = service.options.as_ref().unwrap();
        let unknown_fields = options.unknown_fields();

        let field_count = unknown_fields.iter().count();
        if field_count > 0 {
            println!(
                "cargo:warning=Service {} has {} extension fields",
                service_name, field_count
            );
        }
    }
    Ok(())
}

/// Process a gRPC method definition and extract REST API metadata
fn process_method(
    method: &MethodDescriptorProto,
    service_name: &str,
    codegen_metadata: &mut CodeGenMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    let method_name = method.name();
    let input_type = method.input_type();
    let output_type = method.output_type();

    println!(
        "cargo:warning=  Method {}.{} - input: {}, output: {}",
        service_name, method_name, input_type, output_type
    );

    // Initialize method metadata
    let mut method_metadata = MethodMetadata {
        service_name: service_name.to_string(),
        method_name: method_name.to_string(),
        input_type: input_type.to_string(),
        output_type: output_type.to_string(),
        operation: None,
        http_rule: None,
    };

    // Extract gnostic method-level annotations
    extract_method_annotations(method, service_name, method_name, &mut method_metadata)?;

    // Add to collected metadata
    codegen_metadata.methods.push(method_metadata);

    Ok(())
}

/// Extract gnostic annotations from method-level options
///
/// This is where we extract the key information for REST API generation:
/// - HTTP method and path from google.api.http extension
/// - Operation ID from gnostic.openapi.v3.operation extension
fn extract_method_annotations(
    method: &MethodDescriptorProto,
    service_name: &str,
    method_name: &str,
    method_metadata: &mut MethodMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    if method.options.is_none() {
        println!(
            "cargo:warning=    Method {}.{} has no options",
            service_name, method_name
        );
        return Ok(());
    }

    let options = method.options.as_ref().unwrap();

    let unknown_fields = options.unknown_fields();

    // Process each extension field
    for (field_number, field_value) in unknown_fields.iter() {
        let data = match field_value {
            protobuf::UnknownValueRef::LengthDelimited(bytes) => bytes,
            _ => {
                println!(
                    "cargo:warning=    Skipping non-length-delimited field {}",
                    field_number
                );
                continue;
            }
        };

        match field_number {
            GOOGLE_API_HTTP_EXTENSION => {
                println!(
                    "cargo:warning=    Found google.api.http extension in {}.{}",
                    service_name, method_name
                );

                // Parse HTTP rule from extension data
                match HttpRule::decode(data) {
                    Ok(rule) => {
                        println!(
                            "cargo:warning=      Successfully parsed HTTP rule: {:?}",
                            rule
                        );
                        method_metadata.http_rule = Some(rule);
                    }
                    Err(e) => {
                        println!("cargo:warning=      Failed to parse HTTP rule: {}", e);
                    }
                }
            }
            GNOSTIC_OPERATION_EXTENSION => {
                println!(
                    "cargo:warning=    Found gnostic.openapi.v3.operation extension in {}.{}",
                    service_name, method_name
                );

                // Parse operation from extension data
                match Operation::decode(data) {
                    Ok(operation) => {
                        println!("cargo:warning=      Operation: {:?}", operation);
                        method_metadata.operation = Some(operation);
                    }
                    Err(e) => {
                        println!(
                            "cargo:warning=      Failed to parse gnostic operation: {}",
                            e
                        );
                    }
                }
            }
            _ => {
                // Unknown extension field
                println!(
                    "cargo:warning=    Unknown extension field {} in {}.{}",
                    field_number, service_name, method_name
                );
            }
        }
    }

    Ok(())
}

/// Extract path parameters from HTTP path template
///
/// Parses path templates like "/catalogs/{name}" to extract parameter names
fn extract_path_parameters(path_template: &str) -> Vec<String> {
    let mut params = Vec::new();
    let mut chars = path_template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut param = String::new();
            while let Some(&next_ch) = chars.peek() {
                if next_ch == '}' {
                    chars.next(); // consume the '}'
                    break;
                }
                param.push(chars.next().unwrap());
            }
            if !param.is_empty() {
                params.push(param);
            }
        }
    }

    params
}

// TODO: Code generation functions
// fn generate_handler_code(metadata: &CodeGenMetadata) -> Result<(), Box<dyn std::error::Error>> {
//     // Generate handler traits and implementations from collected metadata
//     Ok(())
// }
