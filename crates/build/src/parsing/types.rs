//! Type conversion utilities for code generation
//!
//! This module provides utilities for converting between protobuf types and Rust types
//! during code generation.

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) static CONVERTER: TypeConverter = TypeConverter {};

/// Context for rendering types in different situations
#[derive(Debug, Clone, Copy)]
pub enum RenderContext {
    /// A constructor (new method) in Rust
    Constructor,
    /// when extracting from a request inside implementations of FromRequest or FromRequestParts
    Extractor,
    /// Regular parameter type
    Parameter,
    /// Return type
    ReturnType,
    /// Field type in a struct
    FieldType,
    /// Builder method parameter
    BuilderMethod,
    /// Python parameter type
    PythonParameter,
}

/// Utility for converting between protobuf and Rust types
pub struct TypeConverter;

impl Default for TypeConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeConverter {
    /// Create a new type converter
    pub fn new() -> Self {
        Self
    }

    /// Convert a unified type to a Rust type string
    pub fn unified_to_rust(&self, unified_type: &UnifiedType, context: RenderContext) -> String {
        let base_type_str = match &unified_type.base_type {
            BaseType::String => {
                if matches!(context, RenderContext::Constructor) && !unified_type.is_optional {
                    "impl Into<String>".to_string()
                } else {
                    "String".to_string()
                }
            }
            BaseType::Int32 => "i32".to_string(),
            BaseType::Int64 => "i64".to_string(),
            BaseType::Bool => "bool".to_string(),
            BaseType::Float64 => "f64".to_string(),
            BaseType::Float32 => "f32".to_string(),
            BaseType::Bytes => "Vec<u8>".to_string(),
            BaseType::Unit => "()".to_string(),
            BaseType::Message(name) => self.extract_simple_type_name(name),
            BaseType::Enum(name) => {
                if matches!(context, RenderContext::Extractor) {
                    "i32".to_string()
                } else {
                    // For enums, extract the simple name and use it directly
                    convert_protobuf_enum_to_rust_type(&format!("TYPE_ENUM:{}", name))
                }
            }
            BaseType::OneOf(name) => self.extract_simple_type_name(name),
            BaseType::Map(key_type, value_type) => {
                if matches!(context, RenderContext::Constructor) && !unified_type.is_optional {
                    "impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>".to_string()
                } else {
                    let key_str = self.unified_to_rust(key_type, context);
                    let value_str = self.unified_to_rust(value_type, context);
                    format!("HashMap<{}, {}>", key_str, value_str)
                }
            }
        };

        let mut result = base_type_str;

        // Apply repeated wrapper.
        // when creating a builder method, we require the inner type only.
        if unified_type.is_repeated && !matches!(context, RenderContext::BuilderMethod) {
            result = format!("Vec<{}>", result);
        }

        // Apply optional wrapper
        // - in builder methods we wrap this in impl Into<Option<T>> so we just need the inner type
        // - in python parameters signatures we wrap repeated fields in optional `Option<Vec<T>>`
        //   to distinguish an empty array from a missing field
        if (unified_type.is_optional && !matches!(context, RenderContext::BuilderMethod))
            || (matches!(context, RenderContext::PythonParameter)
                && (matches!(unified_type.base_type, BaseType::Map(_, _))
                    || unified_type.is_repeated))
        {
            result = format!("Option<{}>", result);
        }

        result
    }

    /// Generate field assignment code for flexible optional fields
    fn flexible_optional_field_assignment(
        &self,
        unified_type: &UnifiedType,
        field_ident: &proc_macro2::Ident,
    ) -> TokenStream {
        if unified_type.is_optional {
            match &unified_type.base_type {
                BaseType::Enum(_) => quote! { #field_ident.into().map(|e| e as i32) },
                _ => quote! { #field_ident.into() },
            }
        } else {
            // For non-optional types that are being made optional in the builder
            match &unified_type.base_type {
                BaseType::String => quote! { #field_ident.into() },
                BaseType::Int32
                | BaseType::Int64
                | BaseType::Bool
                | BaseType::Float64
                | BaseType::Float32 => {
                    quote! { #field_ident.into() }
                }
                BaseType::Enum(_) => {
                    quote! { #field_ident as i32 }
                }
                _ => quote! { #field_ident.into().map(|s| s.to_string()) },
            }
        }
    }

    /// Generate field assignment code
    pub fn field_assignment(
        &self,
        unified_type: &UnifiedType,
        field_ident: &proc_macro2::Ident,
        ctx: &RenderContext,
    ) -> TokenStream {
        if matches!(ctx, RenderContext::BuilderMethod) {
            return self.flexible_optional_field_assignment(unified_type, field_ident);
        }
        match &unified_type.base_type {
            BaseType::String if !unified_type.is_optional => quote! { #field_ident.into() },
            BaseType::Enum(_) => quote! { #field_ident as i32 },
            BaseType::Map(_, _) => quote! {
                #field_ident.into_iter().map(|(k, v)| (k.into(), v.into())).collect()
            },
            _ => quote! { #field_ident },
        }
    }

    /// Extract simple type name from qualified protobuf type
    fn extract_simple_type_name(&self, name: &str) -> String {
        // Handle fully qualified names like "unity.catalog.CreateCatalogRequest"
        if let Some(last_part) = name.split('.').next_back() {
            last_part.to_string()
        } else {
            name.to_string()
        }
    }
}

/// Convert protobuf enum type to Rust type
fn convert_protobuf_enum_to_rust_type(proto_type: &str) -> String {
    if let Some(enum_name) = proto_type.strip_prefix("TYPE_ENUM:") {
        // Remove leading dot if present
        let enum_name = enum_name.trim_start_matches('.');

        // Check if this is a nested enum (has a parent message)
        // Nested enums have PascalCase message names before the enum name
        if let Some(last_dot) = enum_name.rfind('.') {
            let parent_part = &enum_name[..last_dot];
            let enum_simple_name = &enum_name[last_dot + 1..];

            // Check if the parent part ends with a PascalCase message name (nested enum)
            // vs. a package name like "v1" (package-level enum)
            let parent_parts: Vec<&str> = parent_part.split('.').collect();
            if let Some(last_part) = parent_parts.last() {
                // If the last part starts with uppercase, it's likely a message name (nested enum)
                // If it's "v1" or similar version, it's a package-level enum
                if last_part.chars().next().is_some_and(|c| c.is_uppercase())
                    && *last_part != "V1"
                    && !last_part
                        .chars()
                        .all(|c| c.is_lowercase() || c.is_numeric())
                {
                    // This is a nested enum - convert parent message to snake_case module
                    let snake_case_module = last_part.to_case(Case::Snake);

                    // Convert enum name from UPPER_SNAKE_CASE to PascalCase if needed
                    let enum_rust_name = if enum_simple_name.contains('_')
                        && enum_simple_name
                            .chars()
                            .all(|c| c.is_uppercase() || c == '_')
                    {
                        enum_simple_name.to_case(Case::Pascal)
                    } else {
                        enum_simple_name.to_string()
                    };

                    format!("{}::{}", snake_case_module, enum_rust_name)
                } else {
                    // This is a package-level enum - just use the simple name
                    convert_enum_name_to_rust(enum_simple_name)
                }
            } else {
                // Fallback to simple enum name
                convert_enum_name_to_rust(enum_simple_name)
            }
        } else {
            // Simple enum name without dots
            convert_enum_name_to_rust(enum_name)
        }
    } else {
        "i32".to_string() // Fallback for unknown enum types
    }
}

/// Convert enum name from proto format to Rust format
fn convert_enum_name_to_rust(enum_name: &str) -> String {
    // Convert from proto naming to Rust naming if needed
    if enum_name.contains('_') && enum_name.chars().all(|c| c.is_uppercase() || c == '_') {
        enum_name.to_case(Case::Pascal)
    } else {
        enum_name.to_string()
    }
}

/// Unified type representation that can be converted to different target languages
#[derive(Debug, Clone)]
pub struct UnifiedType {
    /// The base type
    pub base_type: BaseType,
    /// Whether this type is optional (Option<T> in Rust)
    pub is_optional: bool,
    /// Whether this type is repeated (Vec<T> in Rust)
    pub is_repeated: bool,
}

/// Base type categories that can be converted to specific language types
#[derive(Debug, Clone)]
pub enum BaseType {
    /// String type
    String,
    /// 32-bit integer
    Int32,
    /// 64-bit integer
    Int64,
    /// Boolean
    Bool,
    /// 64-bit float
    Float64,
    /// 32-bit float
    Float32,
    /// Byte array
    Bytes,
    /// Protobuf message type
    Message(String),
    /// Protobuf enum type
    Enum(String),
    /// Protobuf oneof type
    OneOf(String),
    /// Map type with key and value types
    Map(Box<UnifiedType>, Box<UnifiedType>),
    /// Unit/void type
    Unit,
}

impl UnifiedType {
    /// Create a simple string type
    pub fn string() -> Self {
        Self {
            base_type: BaseType::String,
            is_optional: false,
            is_repeated: false,
        }
    }

    /// Create an optional version of this type
    pub fn optional(mut self) -> Self {
        self.is_optional = true;
        self
    }

    /// Create a repeated version of this type
    pub fn repeated(mut self) -> Self {
        self.is_repeated = true;
        self
    }

    /// Create a map type
    pub fn map(key: UnifiedType, value: UnifiedType) -> Self {
        Self {
            base_type: BaseType::Map(Box::new(key), Box::new(value)),
            is_optional: false,
            is_repeated: false,
        }
    }
}
