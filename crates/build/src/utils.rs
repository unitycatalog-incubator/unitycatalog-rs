//! Shared utilities for the build crate
//!
//! This module contains common functions used across different parts of the code generation
//! pipeline to reduce duplication and improve maintainability.
//!
//! # HTTP Rule Pattern Parsing
//!
//! The `paths` module provides comprehensive parsing of HTTP rule patterns from protobuf
//! service definitions. This allows extraction of detailed information from URL templates.
//!
//! ## Example Usage
//!
//! ```rust
//! use unitycatalog_build::utils::paths::HttpPattern;
//! use unitycatalog_build::google::api::{HttpRule, http_rule::Pattern};
//!
//! // Parse a URL template directly
//! let pattern = HttpPattern::parse("/catalogs/{name}/schemas/{schema}");
//! assert_eq!(pattern.parameters, vec!["name", "schema"]);
//! assert_eq!(pattern.static_prefix, "/catalogs/");
//! assert_eq!(pattern.base_path(), "catalogs");
//!
//! // Generate format string for URL construction
//! let (format_str, args) = pattern.to_format_string();
//! assert_eq!(format_str, "/catalogs/{}/schemas/{}");
//! assert_eq!(args, vec!["name", "schema"]);
//!
//! // Extract parameters from a concrete URL
//! let values = pattern.extract_parameters("/catalogs/main/schemas/default").unwrap();
//! assert_eq!(values, vec!["main", "default"]);
//!
//! // Parse from HttpRule
//! let http_rule = HttpRule {
//!     pattern: Some(Pattern::Get("/tables/{name}".to_string())),
//!     ..Default::default()
//! };
//! let pattern = unitycatalog_build::utils::paths::extract_http_rule_pattern(&http_rule).unwrap();
//! let method = unitycatalog_build::utils::paths::extract_http_method(&http_rule).unwrap();
//! assert_eq!(pattern.parameters, vec!["name"]);
//! assert_eq!(method, "GET");
//! ```

use crate::MessageField;
use convert_case::{Case, Casing};

/// String manipulation utilities
pub mod strings {
    use super::*;

    /// Convert service name to handler trait name
    /// e.g., "CatalogsService" -> "CatalogHandler"
    pub fn service_to_handler_name(service_name: &str) -> String {
        if let Some(base) = service_name.strip_suffix("Service") {
            format!("{}Handler", base.trim_end_matches('s'))
        } else {
            format!("{}Handler", service_name)
        }
    }

    /// Convert operation ID to handler method name
    /// e.g., "ListCatalogs" -> "list_catalogs"
    pub fn operation_to_method_name(operation_id: &str) -> String {
        operation_id.to_case(Case::Snake)
    }

    /// Convert operation ID to route handler function name
    /// e.g., "ListCatalogs" -> "list_catalogs_handler"
    pub fn operation_to_route_name(operation_id: &str) -> String {
        format!("{}_handler", operation_to_method_name(operation_id))
    }

    /// Extract base path from service name
    /// e.g., "CatalogsService" -> "catalogs"
    pub fn service_to_base_path(service_name: &str) -> String {
        if let Some(base) = service_name.strip_suffix("Service") {
            base.to_case(Case::Snake)
        } else {
            service_name.to_case(Case::Snake)
        }
    }

    /// Extract the simple type name from a protobuf type
    /// e.g., ".unitycatalog.catalogs.v1.CatalogInfo" -> "CatalogInfo"
    pub fn extract_simple_type_name(full_type: &str) -> String {
        full_type
            .split('.')
            .next_back()
            .unwrap_or(full_type)
            .to_string()
    }

    /// Convert protobuf field name to Rust identifier
    /// e.g., "full_name" -> "full_name", "someField" -> "some_field"
    pub fn to_rust_field_name(proto_name: &str) -> String {
        proto_name.to_case(Case::Snake)
    }
}

/// Path parameter extraction and matching utilities
pub mod paths {
    use super::*;

    /// Represents a segment in a URL template
    #[derive(Debug, Clone, PartialEq)]
    pub enum UrlSegment {
        /// A static literal segment like "catalogs" or "metadata"
        Static(String),
        /// A path parameter like "{name}" or "{catalog_name}"
        Parameter(String),
    }

    /// Parsed representation of an HTTP rule pattern
    #[derive(Debug, Clone)]
    pub struct HttpPattern {
        /// The original template string
        pub template: String,
        /// Parsed segments in order
        pub segments: Vec<UrlSegment>,
        /// Just the parameter names in order (for backward compatibility)
        pub parameters: Vec<String>,
        /// Static prefix (everything before the first parameter)
        pub static_prefix: String,
        /// Static suffix (everything after the last parameter)
        pub static_suffix: String,
    }

    impl HttpPattern {
        /// Parse an HTTP rule pattern template
        pub fn parse(template: &str) -> Self {
            let segments = parse_url_segments(template);
            let parameters = segments
                .iter()
                .filter_map(|seg| match seg {
                    UrlSegment::Parameter(name) => Some(name.clone()),
                    UrlSegment::Static(_) => None,
                })
                .collect();

            let static_prefix = extract_static_prefix(&segments);
            let static_suffix = extract_static_suffix(&segments);

            HttpPattern {
                template: template.to_string(),
                segments,
                parameters,
                static_prefix,
                static_suffix,
            }
        }

        /// Get the base path (static prefix without leading slash)
        pub fn base_path(&self) -> String {
            self.static_prefix
                .trim_start_matches('/')
                .trim_end_matches('/')
                .to_string()
        }

        /// Check if this pattern has any parameters
        pub fn has_parameters(&self) -> bool {
            !self.parameters.is_empty()
        }

        /// Get the number of parameters
        pub fn parameter_count(&self) -> usize {
            self.parameters.len()
        }

        /// Get parameter names in the order they appear in the URL
        pub fn parameter_names(&self) -> &[String] {
            &self.parameters
        }

        /// Generate a format string for URL construction
        /// Returns ("/catalogs/{}", ["name"]) for "/catalogs/{name}"
        pub fn to_format_string(&self) -> (String, Vec<String>) {
            let mut format_parts = Vec::new();
            let mut format_args = Vec::new();

            for segment in &self.segments {
                match segment {
                    UrlSegment::Static(literal) => {
                        format_parts.push(literal.clone());
                    }
                    UrlSegment::Parameter(name) => {
                        format_parts.push("{}".to_string());
                        format_args.push(name.clone());
                    }
                }
            }

            (format_parts.join(""), format_args)
        }

        /// Extract parameter values from a concrete URL
        /// Returns parameter values in the same order as parameter_names()
        pub fn extract_parameters(&self, url: &str) -> Option<Vec<String>> {
            if self.parameters.is_empty() {
                return if url == self.template {
                    Some(Vec::new())
                } else {
                    None
                };
            }

            // Build regex pattern by replacing each {param} with a capture group
            let mut regex_pattern = self.template.clone();
            for param_name in &self.parameters {
                let placeholder = format!("{{{}}}", param_name);
                regex_pattern = regex_pattern.replace(&placeholder, "([^/]+)");
            }
            // Escape everything except the capture groups we just added
            let mut escaped_pattern = String::new();
            let mut chars = regex_pattern.chars().peekable();
            while let Some(ch) = chars.next() {
                if ch == '(' && chars.peek() == Some(&'[') {
                    // This is our capture group, don't escape it
                    escaped_pattern.push(ch);
                    while let Some(next_ch) = chars.next() {
                        escaped_pattern.push(next_ch);
                        if next_ch == ')' {
                            break;
                        }
                    }
                } else {
                    // Escape special regex characters
                    match ch {
                        '.' | '^' | '$' | '*' | '+' | '?' | '\\' | '[' | ']' | '|' => {
                            escaped_pattern.push('\\');
                            escaped_pattern.push(ch);
                        }
                        _ => escaped_pattern.push(ch),
                    }
                }
            }
            let final_pattern = format!("^{}$", escaped_pattern);

            // Use regex to extract values
            if let Ok(re) = regex::Regex::new(&final_pattern) {
                if let Some(captures) = re.captures(url) {
                    let mut values = Vec::new();
                    for i in 1..=self.parameters.len() {
                        if let Some(capture) = captures.get(i) {
                            values.push(capture.as_str().to_string());
                        } else {
                            return None;
                        }
                    }
                    return Some(values);
                }
            }

            None
        }
    }

    /// Parse URL template into segments
    fn parse_url_segments(template: &str) -> Vec<UrlSegment> {
        let mut segments = Vec::new();
        let mut chars = template.chars().peekable();
        let mut current_static = String::new();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Save any accumulated static content
                if !current_static.is_empty() {
                    segments.push(UrlSegment::Static(current_static.clone()));
                    current_static.clear();
                }

                // Parse parameter name
                let mut param_name = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '}' {
                        chars.next(); // consume the '}'
                        break;
                    }
                    param_name.push(chars.next().unwrap());
                }

                if !param_name.is_empty() {
                    segments.push(UrlSegment::Parameter(param_name));
                }
            } else {
                current_static.push(ch);
            }
        }

        // Add any remaining static content
        if !current_static.is_empty() {
            segments.push(UrlSegment::Static(current_static));
        }

        segments
    }

    /// Extract static prefix (everything before first parameter)
    fn extract_static_prefix(segments: &[UrlSegment]) -> String {
        let mut prefix = String::new();
        for segment in segments {
            match segment {
                UrlSegment::Static(literal) => prefix.push_str(literal),
                UrlSegment::Parameter(_) => break,
            }
        }
        prefix
    }

    /// Extract static suffix (everything after last parameter)
    fn extract_static_suffix(segments: &[UrlSegment]) -> String {
        let mut suffix = String::new();
        let mut found_last_param_index = None;

        // Find the last parameter index
        for (i, segment) in segments.iter().enumerate() {
            if matches!(segment, UrlSegment::Parameter(_)) {
                found_last_param_index = Some(i);
            }
        }

        // If we found a parameter, collect everything after it
        if let Some(last_param_index) = found_last_param_index {
            for segment in segments.iter().skip(last_param_index + 1) {
                if let UrlSegment::Static(literal) = segment {
                    suffix.push_str(literal);
                }
            }
        }

        suffix
    }

    /// Extract path parameter names from URL template like "/catalogs/{name}"
    /// (Kept for backward compatibility)
    pub fn extract_path_parameters(path_template: &str) -> Vec<String> {
        HttpPattern::parse(path_template).parameters
    }

    /// Extract pattern information from an HttpRule
    pub fn extract_http_rule_pattern(
        http_rule: &crate::google::api::HttpRule,
    ) -> Option<HttpPattern> {
        use crate::google::api::http_rule::Pattern;

        let template = match &http_rule.pattern {
            Some(Pattern::Get(path)) => path,
            Some(Pattern::Post(path)) => path,
            Some(Pattern::Put(path)) => path,
            Some(Pattern::Delete(path)) => path,
            Some(Pattern::Patch(path)) => path,
            Some(Pattern::Custom(custom)) => &custom.path,
            None => return None,
        };

        Some(HttpPattern::parse(template))
    }

    /// Get HTTP method string from HttpRule
    pub fn extract_http_method(http_rule: &crate::google::api::HttpRule) -> Option<String> {
        use crate::google::api::http_rule::Pattern;

        match &http_rule.pattern {
            Some(Pattern::Get(_)) => Some("GET".to_string()),
            Some(Pattern::Post(_)) => Some("POST".to_string()),
            Some(Pattern::Put(_)) => Some("PUT".to_string()),
            Some(Pattern::Delete(_)) => Some("DELETE".to_string()),
            Some(Pattern::Patch(_)) => Some("PATCH".to_string()),
            Some(Pattern::Custom(custom)) => Some(custom.kind.clone()),
            None => None,
        }
    }

    /// Find matching field for a path parameter with fallback logic
    pub fn find_matching_field_for_path_param<'a>(
        path_param_name: &str,
        input_fields: &'a [MessageField],
    ) -> Option<&'a MessageField> {
        // First try exact match
        if let Some(field) = input_fields.iter().find(|f| f.name == path_param_name) {
            return Some(field);
        }

        // Try common fallback patterns based on Unity Catalog API conventions
        match path_param_name {
            "name" => {
                // For {name}, try full_name as fallback (common in schema operations)
                input_fields.iter().find(|f| f.name == "full_name")
            }
            "full_name" => {
                // For {full_name}, try name as fallback
                input_fields.iter().find(|f| f.name == "name")
            }
            _ => None,
        }
    }

    /// Generate URL formatting code for templates with path parameters
    pub fn format_url_template(path: &str, param_names: &[String]) -> (String, Vec<String>) {
        if param_names.is_empty() {
            return (path.to_string(), Vec::new());
        }

        let mut format_string = path.to_string();
        let mut format_args = Vec::new();

        for param_name in param_names {
            let placeholder = format!("{{{}}}", param_name);
            if format_string.contains(&placeholder) {
                format_string = format_string.replace(&placeholder, "{}");
                format_args.push(param_name.clone());
            }
        }

        (format_string, format_args)
    }
}

/// Type mapping utilities
pub mod types {
    use crate::RequestType;

    /// Convert protobuf field type to Rust type
    pub fn field_type_to_rust_type(field_type: &str) -> String {
        match field_type {
            "TYPE_STRING" => "String".to_string(),
            "TYPE_INT32" => "i32".to_string(),
            "TYPE_INT64" => "i64".to_string(),
            "TYPE_BOOL" => "bool".to_string(),
            "TYPE_DOUBLE" => "f64".to_string(),
            "TYPE_FLOAT" => "f32".to_string(),
            "TYPE_BYTES" => "Vec<u8>".to_string(),
            _ => {
                // For message types, extract the simple name
                if field_type.starts_with("TYPE_MESSAGE:") {
                    super::strings::extract_simple_type_name(&field_type[13..])
                } else if field_type.starts_with("TYPE_ENUM:") {
                    // Enum fields are represented as i32 in the generated structs
                    "i32".to_string()
                } else if let Some(oneof_name) = field_type.strip_prefix("TYPE_ONEOF:") {
                    // Extract the oneof name and create an enum type
                    super::strings::extract_simple_type_name(oneof_name)
                } else {
                    // Default to String for unknown types
                    "String".to_string()
                }
            }
        }
    }

    /// Determine Rust type for path parameters
    pub fn path_param_type(param_name: &str) -> String {
        match param_name {
            "id" => "uuid::Uuid".to_string(),
            _ => "String".to_string(),
        }
    }

    /// Determine if a request type should have a response body
    pub fn has_response_body(request_type: &RequestType) -> bool {
        !matches!(request_type, RequestType::Delete)
    }

    /// Make a type optional by wrapping in Option<T>
    pub fn make_optional(rust_type: &str) -> String {
        format!("Option<{}>", rust_type)
    }
}

/// Validation utilities
pub mod validation {
    use crate::codegen::{GenerationPlan, MethodPlan};

    /// Validate that a generation plan is complete and correct
    pub fn validate_plan(plan: &GenerationPlan) -> Result<(), Box<dyn std::error::Error>> {
        let mut errors = Vec::new();

        // Check that all services have at least one method
        for service in &plan.services {
            if service.methods.is_empty() {
                errors.push(format!("Service {} has no methods", service.service_name));
            }

            // Check that all methods have required information
            for method in &service.methods {
                validate_method_plan(method, &mut errors);
            }
        }

        if !errors.is_empty() {
            return Err(format!("Validation errors: {}", errors.join(", ")).into());
        }

        Ok(())
    }

    fn validate_method_plan(method: &MethodPlan, errors: &mut Vec<String>) {
        if method.handler_function_name.is_empty() {
            errors.push(format!(
                "Method {} has empty handler function name",
                method.metadata.method_name
            ));
        }

        if method.http_method.is_empty() {
            errors.push(format!(
                "Method {} has empty HTTP method",
                method.metadata.method_name
            ));
        }

        if method.http_path.is_empty() {
            errors.push(format!(
                "Method {} has empty HTTP path",
                method.metadata.method_name
            ));
        }

        // Validate that path parameters in URL match extracted parameters
        let url_params = super::paths::extract_path_parameters(&method.http_path);
        if url_params.len() != method.path_params.len() {
            errors.push(format!(
                "Method {} has mismatched path parameters: URL has {}, extracted {}",
                method.metadata.method_name,
                url_params.len(),
                method.path_params.len()
            ));
        }
    }
}

/// Request classification utilities
pub mod requests {
    use crate::RequestType;

    /// Determine request type from operation ID or method name
    pub fn classify_request_type(operation_id: Option<&str>, method_name: &str) -> RequestType {
        let name = operation_id.unwrap_or(method_name);

        if name.starts_with("List") {
            RequestType::List
        } else if name.starts_with("Create") || name.starts_with("Generate") {
            RequestType::Create
        } else if name.starts_with("Get") {
            RequestType::Get
        } else if name.starts_with("Update") || name.starts_with("Query") {
            RequestType::Update
        } else if name.starts_with("Delete") {
            RequestType::Delete
        } else {
            RequestType::Get // Default fallback
        }
    }

    /// Determine if a field should be extracted from request body
    pub fn should_be_body_field(field_name: &str, body_spec: &str) -> bool {
        match body_spec {
            "*" => true, // All fields not in path go to body
            "" => false, // No body fields
            specific_field => field_name == specific_field,
        }
    }

    /// Check if a method requires pagination parameters
    pub fn needs_pagination(request_type: &RequestType) -> bool {
        matches!(request_type, RequestType::List)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod string_tests {
        use super::*;

        #[test]
        fn test_service_to_handler_name() {
            assert_eq!(
                strings::service_to_handler_name("CatalogsService"),
                "CatalogHandler"
            );
            assert_eq!(
                strings::service_to_handler_name("RecipientsService"),
                "RecipientHandler"
            );
            assert_eq!(
                strings::service_to_handler_name("SchemasService"),
                "SchemaHandler"
            );
        }

        #[test]
        fn test_operation_to_method_name() {
            assert_eq!(
                strings::operation_to_method_name("ListCatalogs"),
                "list_catalogs"
            );
            assert_eq!(
                strings::operation_to_method_name("CreateCatalog"),
                "create_catalog"
            );
            assert_eq!(
                strings::operation_to_method_name("GetCatalog"),
                "get_catalog"
            );
        }

        #[test]
        fn test_service_to_base_path() {
            assert_eq!(strings::service_to_base_path("CatalogsService"), "catalogs");
            assert_eq!(
                strings::service_to_base_path("RecipientsService"),
                "recipients"
            );
        }

        #[test]
        fn test_extract_simple_type_name() {
            assert_eq!(
                strings::extract_simple_type_name(".unitycatalog.catalogs.v1.CatalogInfo"),
                "CatalogInfo"
            );
            assert_eq!(
                strings::extract_simple_type_name("SimpleType"),
                "SimpleType"
            );
        }
    }

    mod path_tests {
        use super::*;

        #[test]
        fn test_extract_path_parameters() {
            assert_eq!(
                paths::extract_path_parameters("/catalogs/{name}"),
                vec!["name"]
            );
            assert_eq!(
                paths::extract_path_parameters("/shares/{share}/schemas/{schema}/tables/{name}"),
                vec!["share", "schema", "name"]
            );
            assert_eq!(
                paths::extract_path_parameters("/catalogs"),
                Vec::<String>::new()
            );
        }

        #[test]
        fn test_format_url_template() {
            let (format_str, args) =
                paths::format_url_template("/catalogs/{name}", &["name".to_string()]);
            assert_eq!(format_str, "/catalogs/{}");
            assert_eq!(args, vec!["name"]);

            let (format_str, args) = paths::format_url_template("/catalogs", &[]);
            assert_eq!(format_str, "/catalogs");
            assert_eq!(args, Vec::<String>::new());
        }

        #[test]
        fn test_http_pattern_parsing() {
            // Test simple static path
            let pattern = paths::HttpPattern::parse("/catalogs");
            assert_eq!(pattern.parameters, Vec::<String>::new());
            assert_eq!(pattern.static_prefix, "/catalogs");
            assert_eq!(pattern.static_suffix, "");
            assert!(!pattern.has_parameters());

            // Test single parameter
            let pattern = paths::HttpPattern::parse("/catalogs/{name}");
            assert_eq!(pattern.parameters, vec!["name"]);
            assert_eq!(pattern.static_prefix, "/catalogs/");
            assert_eq!(pattern.static_suffix, "");
            assert!(pattern.has_parameters());
            assert_eq!(pattern.parameter_count(), 1);

            // Test multiple parameters
            let pattern =
                paths::HttpPattern::parse("/shares/{share}/schemas/{schema}/tables/{name}");
            assert_eq!(pattern.parameters, vec!["share", "schema", "name"]);
            assert_eq!(pattern.static_prefix, "/shares/");
            assert_eq!(pattern.static_suffix, "");
            assert_eq!(pattern.parameter_count(), 3);

            // Test parameter with suffix
            let pattern = paths::HttpPattern::parse("/catalogs/{name}/metadata");
            assert_eq!(pattern.parameters, vec!["name"]);
            assert_eq!(pattern.static_prefix, "/catalogs/");
            assert_eq!(pattern.static_suffix, "/metadata");
        }

        #[test]
        fn test_http_pattern_segments() {
            let pattern = paths::HttpPattern::parse("/shares/{share}/schemas/{schema}");

            use paths::UrlSegment;
            assert_eq!(
                pattern.segments,
                vec![
                    UrlSegment::Static("/shares/".to_string()),
                    UrlSegment::Parameter("share".to_string()),
                    UrlSegment::Static("/schemas/".to_string()),
                    UrlSegment::Parameter("schema".to_string()),
                ]
            );
        }

        #[test]
        fn test_http_pattern_to_format_string() {
            let pattern = paths::HttpPattern::parse("/catalogs/{name}");
            let (format_str, args) = pattern.to_format_string();
            assert_eq!(format_str, "/catalogs/{}");
            assert_eq!(args, vec!["name"]);

            let pattern = paths::HttpPattern::parse("/shares/{share}/schemas/{schema}");
            let (format_str, args) = pattern.to_format_string();
            assert_eq!(format_str, "/shares/{}/schemas/{}");
            assert_eq!(args, vec!["share", "schema"]);
        }

        #[test]
        fn test_http_pattern_extract_parameters() {
            let pattern = paths::HttpPattern::parse("/catalogs/{name}");
            assert_eq!(
                pattern.extract_parameters("/catalogs/main"),
                Some(vec!["main".to_string()])
            );
            assert_eq!(pattern.extract_parameters("/catalogs/"), None);
            assert_eq!(pattern.extract_parameters("/schemas/main"), None);

            let pattern = paths::HttpPattern::parse("/shares/{share}/schemas/{schema}");
            assert_eq!(
                pattern.extract_parameters("/shares/unity/schemas/default"),
                Some(vec!["unity".to_string(), "default".to_string()])
            );
            assert_eq!(pattern.extract_parameters("/shares/unity"), None);
        }

        #[test]
        fn test_http_pattern_base_path() {
            let pattern = paths::HttpPattern::parse("/catalogs/{name}");
            assert_eq!(pattern.base_path(), "catalogs");

            let pattern = paths::HttpPattern::parse("/shares/{share}/schemas");
            assert_eq!(pattern.base_path(), "shares");
        }

        #[test]
        fn test_extract_http_rule_pattern() {
            use crate::google::api::{HttpRule, http_rule::Pattern};

            // Test GET pattern
            let http_rule = HttpRule {
                pattern: Some(Pattern::Get("/catalogs/{name}".to_string())),
                ..Default::default()
            };

            let pattern = paths::extract_http_rule_pattern(&http_rule).unwrap();
            assert_eq!(pattern.parameters, vec!["name"]);
            assert_eq!(pattern.template, "/catalogs/{name}");

            // Test POST pattern
            let http_rule = HttpRule {
                pattern: Some(Pattern::Post("/catalogs".to_string())),
                ..Default::default()
            };

            let pattern = paths::extract_http_rule_pattern(&http_rule).unwrap();
            assert_eq!(pattern.parameters, Vec::<String>::new());
            assert_eq!(pattern.template, "/catalogs");

            // Test None pattern
            let http_rule = HttpRule {
                pattern: None,
                ..Default::default()
            };

            assert!(paths::extract_http_rule_pattern(&http_rule).is_none());
        }

        #[test]
        fn test_extract_http_method() {
            use crate::google::api::{CustomHttpPattern, HttpRule, http_rule::Pattern};

            let test_cases = vec![
                (Pattern::Get("/test".to_string()), "GET"),
                (Pattern::Post("/test".to_string()), "POST"),
                (Pattern::Put("/test".to_string()), "PUT"),
                (Pattern::Delete("/test".to_string()), "DELETE"),
                (Pattern::Patch("/test".to_string()), "PATCH"),
            ];

            for (pattern, expected_method) in test_cases {
                let http_rule = HttpRule {
                    pattern: Some(pattern),
                    ..Default::default()
                };

                assert_eq!(
                    paths::extract_http_method(&http_rule).unwrap(),
                    expected_method
                );
            }

            // Test custom pattern
            let custom_pattern = CustomHttpPattern {
                kind: "HEAD".to_string(),
                path: "/test".to_string(),
            };
            let http_rule = HttpRule {
                pattern: Some(Pattern::Custom(custom_pattern)),
                ..Default::default()
            };

            assert_eq!(paths::extract_http_method(&http_rule).unwrap(), "HEAD");

            // Test None pattern
            let http_rule = HttpRule {
                pattern: None,
                ..Default::default()
            };

            assert!(paths::extract_http_method(&http_rule).is_none());
        }
    }

    mod type_tests {
        use super::*;
        use crate::RequestType;

        #[test]
        fn test_field_type_to_rust_type() {
            assert_eq!(types::field_type_to_rust_type("TYPE_STRING"), "String");
            assert_eq!(types::field_type_to_rust_type("TYPE_INT32"), "i32");
            assert_eq!(types::field_type_to_rust_type("TYPE_BOOL"), "bool");
            assert_eq!(
                types::field_type_to_rust_type(
                    "TYPE_MESSAGE:.unitycatalog.catalogs.v1.CatalogInfo"
                ),
                "CatalogInfo"
            );
        }

        #[test]
        fn test_has_response_body() {
            assert!(types::has_response_body(&RequestType::Get));
            assert!(types::has_response_body(&RequestType::Create));
            assert!(types::has_response_body(&RequestType::Update));
            assert!(types::has_response_body(&RequestType::List));
            assert!(!types::has_response_body(&RequestType::Delete));
        }
    }

    mod request_tests {
        use super::*;
        use crate::RequestType;

        #[test]
        fn test_classify_request_type() {
            assert_eq!(
                requests::classify_request_type(Some("ListCatalogs"), "ListCatalogs"),
                RequestType::List
            );
            assert_eq!(
                requests::classify_request_type(Some("CreateCatalog"), "CreateCatalog"),
                RequestType::Create
            );
            assert_eq!(
                requests::classify_request_type(Some("GetCatalog"), "GetCatalog"),
                RequestType::Get
            );
            assert_eq!(
                requests::classify_request_type(Some("UpdateCatalog"), "UpdateCatalog"),
                RequestType::Update
            );
            assert_eq!(
                requests::classify_request_type(Some("DeleteCatalog"), "DeleteCatalog"),
                RequestType::Delete
            );
        }

        #[test]
        fn test_should_be_body_field() {
            assert!(requests::should_be_body_field("any_field", "*"));
            assert!(!requests::should_be_body_field("any_field", ""));
            assert!(requests::should_be_body_field("specific", "specific"));
            assert!(!requests::should_be_body_field("other", "specific"));
        }
    }
}
