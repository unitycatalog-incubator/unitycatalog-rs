use std::collections::HashMap;

use protobuf::descriptor::{FileDescriptorProto, FileDescriptorSet};
pub use types::{RenderContext, TypeConverter};

pub(crate) use self::http::*;
pub(crate) use self::models::*;
pub mod types;

mod enum_parser;
mod http;
mod message;
mod models;
mod service;

// Known extension field numbers
const GOOGLE_API_HTTP_EXTENSION: u32 = 72295728; // google.api.http
const GNOSTIC_OPERATION_EXTENSION: u32 = 1143; // gnostic.openapi.v3.operation
const GOOGLE_API_RESOURCE_EXTENSION: u32 = 1053; // google.api.resource
const GOOGLE_API_FIELD_BEHAVIOR_EXTENSION: u32 = 1052; // google.api.field_behavior

pub fn parse_file_descriptor_set(
    file_descriptor_set: &FileDescriptorSet,
) -> Result<CodeGenMetadata, Box<dyn std::error::Error>> {
    let mut codegen_metadata = CodeGenMetadata {
        messages: HashMap::new(),
        enums: HashMap::new(),
        services: HashMap::new(),
    };

    // Process each file descriptor
    for file_descriptor in &file_descriptor_set.file {
        process_file_descriptor(file_descriptor, &mut codegen_metadata)?;
    }

    Ok(codegen_metadata)
}

/// Process a single protobuf file descriptor
///
/// Extracts all messages, services, and annotations from the file.
/// Collects metadata for code generation.
pub fn process_file_descriptor(
    file_desc: &FileDescriptorProto,
    codegen_metadata: &mut CodeGenMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = file_desc.name();

    // Extract source code info for documentation
    let source_code_info = file_desc.source_code_info.as_ref();

    // Process enums in the file
    for (enum_index, enum_desc) in file_desc.enum_type.iter().enumerate() {
        let package_name = file_desc.package();
        let type_prefix = if package_name.is_empty() {
            String::new()
        } else {
            format!(".{}", package_name)
        };
        enum_parser::process_enum(
            enum_desc,
            codegen_metadata,
            &type_prefix,
            source_code_info,
            &[5, enum_index as i32], // enum_type is field 5 in FileDescriptorProto
        )?;
    }

    // Process messages in the file
    for (message_index, message) in file_desc.message_type.iter().enumerate() {
        let package_name = file_desc.package();
        let type_prefix = if package_name.is_empty() {
            String::new()
        } else {
            format!(".{}", package_name)
        };

        message::process_message(
            message,
            file_name,
            codegen_metadata,
            &type_prefix,
            source_code_info,
            &[4, message_index as i32], // message_type is field 4 in FileDescriptorProto
        )?;
    }

    // Process services in the file
    for (service_index, service) in file_desc.service.iter().enumerate() {
        service::process_service(service, codegen_metadata, source_code_info, service_index)?;
    }

    Ok(())
}
