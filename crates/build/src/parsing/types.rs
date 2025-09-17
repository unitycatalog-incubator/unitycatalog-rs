//! Type conversion utilities for code generation
//!
//! This module provides utilities for converting between protobuf types and Rust types
//! during code generation.

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;

/// Context for rendering types in different situations
#[derive(Debug, Clone, Copy)]
pub enum RenderContext {
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

    /// Convert a protobuf field type string to a unified type representation
    pub fn protobuf_to_unified(&self, proto_type: &str) -> UnifiedType {
        match proto_type {
            "TYPE_STRING" => UnifiedType {
                base_type: BaseType::String,
                is_optional: false,
                is_repeated: false,
            },
            "TYPE_INT32" => UnifiedType {
                base_type: BaseType::Int32,
                is_optional: false,
                is_repeated: false,
            },
            "TYPE_INT64" => UnifiedType {
                base_type: BaseType::Int64,
                is_optional: false,
                is_repeated: false,
            },
            "TYPE_BOOL" => UnifiedType {
                base_type: BaseType::Bool,
                is_optional: false,
                is_repeated: false,
            },
            "TYPE_DOUBLE" => UnifiedType {
                base_type: BaseType::Float64,
                is_optional: false,
                is_repeated: false,
            },
            "TYPE_FLOAT" => UnifiedType {
                base_type: BaseType::Float32,
                is_optional: false,
                is_repeated: false,
            },
            "TYPE_BYTES" => UnifiedType {
                base_type: BaseType::Bytes,
                is_optional: false,
                is_repeated: false,
            },
            _ => {
                if let Some(message_name) = proto_type.strip_prefix("TYPE_MESSAGE:") {
                    UnifiedType {
                        base_type: BaseType::Message(message_name.to_string()),
                        is_optional: false,
                        is_repeated: false,
                    }
                } else if let Some(enum_name) = proto_type.strip_prefix("TYPE_ENUM:") {
                    UnifiedType {
                        base_type: BaseType::Enum(enum_name.to_string()),
                        is_optional: false,
                        is_repeated: false,
                    }
                } else if let Some(oneof_name) = proto_type.strip_prefix("TYPE_ONEOF:") {
                    UnifiedType {
                        base_type: BaseType::OneOf(oneof_name.to_string()),
                        is_optional: false,
                        is_repeated: false,
                    }
                } else if proto_type.contains("map<") {
                    // Parse map<key, value> format
                    if let Some((key_type, value_type)) = self.parse_map_type(proto_type) {
                        let key_unified = self.protobuf_to_unified(&key_type);
                        let value_unified = self.protobuf_to_unified(&value_type);
                        UnifiedType {
                            base_type: BaseType::Map(
                                Box::new(key_unified),
                                Box::new(value_unified),
                            ),
                            is_optional: false,
                            is_repeated: false,
                        }
                    } else {
                        // Fallback to string map
                        UnifiedType::map(UnifiedType::string(), UnifiedType::string())
                    }
                } else {
                    // Default fallback to string
                    UnifiedType::string()
                }
            }
        }
    }

    /// Convert a unified type to a Rust type string
    pub fn unified_to_rust(&self, unified_type: &UnifiedType, context: RenderContext) -> String {
        let base_type_str = match &unified_type.base_type {
            BaseType::String => "String".to_string(),
            BaseType::Int32 => "i32".to_string(),
            BaseType::Int64 => "i64".to_string(),
            BaseType::Bool => "bool".to_string(),
            BaseType::Float64 => "f64".to_string(),
            BaseType::Float32 => "f32".to_string(),
            BaseType::Bytes => "Vec<u8>".to_string(),
            BaseType::Unit => "()".to_string(),
            BaseType::Message(name) => self.extract_simple_type_name(name),
            BaseType::Enum(name) => {
                // For enums, extract the simple name and use it directly
                self.convert_protobuf_enum_to_rust_type(&format!("TYPE_ENUM:{}", name))
            }
            BaseType::OneOf(name) => self.extract_simple_type_name(name),
            BaseType::Map(key_type, value_type) => {
                let key_str = self.unified_to_rust(key_type, context);
                let value_str = self.unified_to_rust(value_type, context);
                format!("HashMap<{}, {}>", key_str, value_str)
            }
        };

        let mut result = base_type_str;

        // Apply repeated wrapper.
        // when creating a builder method, we require the innre type only.
        if unified_type.is_repeated && !matches!(context, RenderContext::BuilderMethod) {
            result = format!("Vec<{}>", result);
        }

        // Apply optional wrapper
        // - in builder methods we wrap this in impl Into<Option<T>> so we just need the inner type
        if unified_type.is_optional && !matches!(context, RenderContext::BuilderMethod) {
            result = format!("Option<{}>", result);
        }

        result
    }

    /// Generate a Rust parameter type string (may use impl Into<> for convenience)
    pub fn rust_parameter_type(&self, unified_type: &UnifiedType) -> String {
        match &unified_type.base_type {
            BaseType::String if !unified_type.is_optional => "impl Into<String>".to_string(),
            BaseType::Map(_, _) => {
                "impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>".to_string()
            }
            _ => self.unified_to_rust(unified_type, RenderContext::Parameter),
        }
    }

    /// Generate field assignment code for flexible optional fields
    pub fn flexible_optional_field_assignment(
        &self,
        unified_type: &UnifiedType,
        field_ident: &proc_macro2::Ident,
    ) -> TokenStream {
        if unified_type.is_optional {
            quote! { #field_ident.into() }
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
                    quote! { #field_ident.into().map(|e| e as i32) }
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
    ) -> TokenStream {
        match &unified_type.base_type {
            BaseType::String if !unified_type.is_optional => quote! { #field_ident.into() },
            BaseType::Enum(_) => quote! { #field_ident as i32 },
            BaseType::Map(_, _) => quote! {
                #field_ident.into_iter().map(|(k, v)| (k.into(), v.into())).collect()
            },
            _ => quote! { #field_ident },
        }
    }

    /// Parse a map type string like "map<string, int32>"
    fn parse_map_type(&self, type_str: &str) -> Option<(String, String)> {
        // Remove "map<" prefix and ">" suffix
        let inner = type_str.strip_prefix("map<")?.strip_suffix(">")?;

        // Find the comma separating key and value types
        let mut depth = 0;
        let mut comma_pos = None;

        for (i, c) in inner.char_indices() {
            match c {
                '<' => depth += 1,
                '>' => depth -= 1,
                ',' if depth == 0 => {
                    comma_pos = Some(i);
                    break;
                }
                _ => {}
            }
        }

        let comma_pos = comma_pos?;
        let key_type = inner[..comma_pos].trim();
        let value_type = inner[comma_pos + 1..].trim();

        // Convert simple types to TYPE_ format
        let key_type = self.normalize_type_name(key_type);
        let value_type = self.normalize_type_name(value_type);

        Some((key_type, value_type))
    }

    /// Normalize type names from proto format to TYPE_ format
    fn normalize_type_name(&self, type_name: &str) -> String {
        match type_name {
            "string" => "TYPE_STRING".to_string(),
            "int32" => "TYPE_INT32".to_string(),
            "int64" => "TYPE_INT64".to_string(),
            "bool" => "TYPE_BOOL".to_string(),
            "double" => "TYPE_DOUBLE".to_string(),
            "float" => "TYPE_FLOAT".to_string(),
            "bytes" => "TYPE_BYTES".to_string(),
            _ => format!("TYPE_MESSAGE:{}", type_name),
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

    /// Convert protobuf enum type to Rust type
    fn convert_protobuf_enum_to_rust_type(&self, proto_type: &str) -> String {
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
                        self.convert_enum_name_to_rust(enum_simple_name)
                    }
                } else {
                    // Fallback to simple enum name
                    self.convert_enum_name_to_rust(enum_simple_name)
                }
            } else {
                // Simple enum name without dots
                self.convert_enum_name_to_rust(enum_name)
            }
        } else {
            "i32".to_string() // Fallback for unknown enum types
        }
    }

    /// Convert enum name from proto format to Rust format
    fn convert_enum_name_to_rust(&self, enum_name: &str) -> String {
        // Convert from proto naming to Rust naming if needed
        if enum_name.contains('_') && enum_name.chars().all(|c| c.is_uppercase() || c == '_') {
            enum_name.to_case(Case::Pascal)
        } else {
            enum_name.to_string()
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_converter() -> TypeConverter {
        TypeConverter::new()
    }

    #[test]
    fn test_basic_type_conversion() {
        let converter = create_test_converter();

        let string_type = converter.protobuf_to_unified("TYPE_STRING");
        assert!(matches!(string_type.base_type, BaseType::String));
        assert!(!string_type.is_optional);
        assert!(!string_type.is_repeated);

        let rust_type = converter.unified_to_rust(&string_type, RenderContext::Parameter);
        assert_eq!(rust_type, "String");
    }

    #[test]
    fn test_optional_type_conversion() {
        let converter = create_test_converter();

        let mut string_type = converter.protobuf_to_unified("TYPE_STRING");
        string_type.is_optional = true;

        let rust_type = converter.unified_to_rust(&string_type, RenderContext::Parameter);
        assert_eq!(rust_type, "Option<String>");
    }

    #[test]
    fn test_repeated_type_conversion() {
        let converter = create_test_converter();

        let mut string_type = converter.protobuf_to_unified("TYPE_STRING");
        string_type.is_repeated = true;

        let rust_type = converter.unified_to_rust(&string_type, RenderContext::Parameter);
        assert_eq!(rust_type, "Vec<String>");
    }

    #[test]
    fn test_map_type_parsing() {
        let converter = create_test_converter();

        if let Some((key, value)) = converter.parse_map_type("map<string, int32>") {
            assert_eq!(key, "TYPE_STRING");
            assert_eq!(value, "TYPE_INT32");
        } else {
            panic!("Failed to parse map type");
        }
    }

    #[test]
    fn test_message_type_conversion() {
        let converter = create_test_converter();

        let message_type =
            converter.protobuf_to_unified("TYPE_MESSAGE:unity.catalog.CreateCatalogRequest");
        let rust_type = converter.unified_to_rust(&message_type, RenderContext::Parameter);
        assert_eq!(rust_type, "CreateCatalogRequest");
    }

    #[test]
    fn test_enum_type_conversion() {
        let converter = create_test_converter();

        // Test simple enum
        let enum_type = converter.protobuf_to_unified("TYPE_ENUM:CATALOG_TYPE");
        let rust_type = converter.unified_to_rust(&enum_type, RenderContext::Parameter);
        assert_eq!(rust_type, "CatalogType");

        // Test package-level enum (not nested in a message)
        let package_enum_type =
            converter.protobuf_to_unified("TYPE_ENUM:.unitycatalog.credentials.v1.Purpose");
        let package_rust_type =
            converter.unified_to_rust(&package_enum_type, RenderContext::Parameter);
        assert_eq!(package_rust_type, "Purpose");

        // Test nested enum with qualified path
        let nested_enum_type = converter.protobuf_to_unified(
            "TYPE_ENUM:.unitycatalog.temporary_credentials.v1.GenerateTemporaryTableCredentialsRequest.Operation"
        );
        let nested_rust_type =
            converter.unified_to_rust(&nested_enum_type, RenderContext::Parameter);
        assert_eq!(
            nested_rust_type,
            "generate_temporary_table_credentials_request::Operation"
        );

        // Test another nested enum
        let path_enum_type = converter.protobuf_to_unified(
            "TYPE_ENUM:.unitycatalog.temporary_credentials.v1.GenerateTemporaryPathCredentialsRequest.Operation"
        );
        let path_rust_type = converter.unified_to_rust(&path_enum_type, RenderContext::Parameter);
        assert_eq!(
            path_rust_type,
            "generate_temporary_path_credentials_request::Operation"
        );
    }
}
