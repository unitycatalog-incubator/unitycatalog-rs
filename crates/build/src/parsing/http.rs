/// Represents a segment in a URL template
#[derive(Debug, Clone, PartialEq)]
pub enum UrlSegment {
    /// A static literal segment like "catalogs" or "metadata"
    Static(String),
    /// A path parameter like "{name}" or "{catalog_name}"
    Parameter(String),
}

/// Parsed representation of an HTTP rule pattern
///
/// ### Example Usage
///
/// ```rust
/// use unitycatalog_build::utils::paths::HttpPattern;
/// use unitycatalog_build::google::api::{HttpRule, http_rule::Pattern};
///
/// // Parse a URL template directly
/// let pattern = HttpPattern::parse("/catalogs/{name}/schemas/{schema}");
/// assert_eq!(pattern.parameters, vec!["name", "schema"]);
/// assert_eq!(pattern.static_prefix, "/catalogs/");
/// assert_eq!(pattern.base_path(), "catalogs");
///
/// // Generate format string for URL construction
/// let (format_str, args) = pattern.to_format_string();
/// assert_eq!(format_str, "/catalogs/{}/schemas/{}");
/// assert_eq!(args, vec!["name", "schema"]);
///
/// // Extract parameters from a concrete URL
/// let values = pattern.extract_parameters("/catalogs/main/schemas/default").unwrap();
/// assert_eq!(values, vec!["main", "default"]);
///
/// // Parse from HttpRule
/// let http_rule = HttpRule {
///     pattern: Some(Pattern::Get("/tables/{name}".to_string())),
///     ..Default::default()
/// };
/// let pattern = unitycatalog_build::utils::paths::extract_http_rule_pattern(&http_rule).unwrap();
/// let method = unitycatalog_build::utils::paths::extract_http_method(&http_rule).unwrap();
/// assert_eq!(pattern.parameters, vec!["name"]);
/// assert_eq!(method, "GET");
/// ```
#[derive(Debug, Clone)]
pub(crate) struct HttpPattern {
    /// The original template string
    pub template: String,
    /// Parsed segments in order
    pub segments: Vec<UrlSegment>,
    /// Just the parameter names in order (for backward compatibility)
    pub parameters: Vec<String>,
    /// Static prefix (everything before the first parameter)
    pub static_prefix: String,
    /// Static suffix (everything after the last parameter)
    pub static_suffix: Option<String>,
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

    pub fn ends_with_static(&self) -> bool {
        self.segments
            .last()
            .is_some_and(|seg| matches!(seg, UrlSegment::Static(_)))
    }

    pub fn ends_with_parameter(&self) -> bool {
        self.segments
            .last()
            .is_some_and(|seg| matches!(seg, UrlSegment::Parameter(_)))
    }

    /// Get the base path (static prefix without leading slash)
    pub fn base_path(&self) -> String {
        self.static_prefix
            .trim_start_matches('/')
            .trim_end_matches('/')
            .to_string()
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

        (
            format_parts.join("").trim_start_matches('/').to_string(),
            format_args,
        )
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
fn extract_static_suffix(segments: &[UrlSegment]) -> Option<String> {
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

    (!suffix.is_empty()).then_some(suffix)
}

/// Extract path parameter names from URL template like "/catalogs/{name}"
/// (Kept for backward compatibility)
pub(crate) fn extract_path_parameters(path_template: &str) -> Vec<String> {
    HttpPattern::parse(path_template).parameters
}

/// Extract pattern information from an HttpRule
pub(crate) fn extract_http_rule_pattern(
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

/// Determine if a field should be extracted from request body
pub(crate) fn should_be_body_field(field_name: &str, body_spec: &str) -> bool {
    match body_spec {
        "*" => true, // All fields not in path go to body
        "" => false, // No body fields
        specific_fields => specific_fields.split(',').any(|name| name == field_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_path_parameters() {
        assert_eq!(extract_path_parameters("/catalogs/{name}"), vec!["name"]);
        assert_eq!(
            extract_path_parameters("/shares/{share}/schemas/{schema}/tables/{name}"),
            vec!["share", "schema", "name"]
        );
        assert_eq!(extract_path_parameters("/catalogs"), Vec::<String>::new());
    }

    #[test]
    fn test_http_pattern_parsing() {
        // Test simple static path
        let pattern = HttpPattern::parse("/catalogs");
        assert_eq!(pattern.parameters, Vec::<String>::new());
        assert_eq!(pattern.static_prefix, "/catalogs");
        assert_eq!(pattern.static_suffix, None);

        // Test single parameter
        let pattern = HttpPattern::parse("/catalogs/{name}");
        assert_eq!(pattern.parameters, vec!["name"]);
        assert_eq!(pattern.static_prefix, "/catalogs/");
        assert_eq!(pattern.static_suffix, None);

        // Test multiple parameters
        let pattern = HttpPattern::parse("/shares/{share}/schemas/{schema}/tables/{name}");
        assert_eq!(pattern.parameters, vec!["share", "schema", "name"]);
        assert_eq!(pattern.static_prefix, "/shares/");
        assert_eq!(pattern.static_suffix, None);

        // Test parameter with suffix
        let pattern = HttpPattern::parse("/catalogs/{name}/metadata");
        assert_eq!(pattern.parameters, vec!["name"]);
        assert_eq!(pattern.static_prefix, "/catalogs/");
        assert_eq!(pattern.static_suffix.as_deref(), Some("/metadata"));
    }

    #[test]
    fn test_http_pattern_segments() {
        let pattern = HttpPattern::parse("/shares/{share}/schemas/{schema}");

        use UrlSegment;
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
        let pattern = HttpPattern::parse("/catalogs/{name}");
        let (format_str, args) = pattern.to_format_string();
        assert_eq!(format_str, "/catalogs/{}");
        assert_eq!(args, vec!["name"]);

        let pattern = HttpPattern::parse("/shares/{share}/schemas/{schema}");
        let (format_str, args) = pattern.to_format_string();
        assert_eq!(format_str, "/shares/{}/schemas/{}");
        assert_eq!(args, vec!["share", "schema"]);
    }

    #[test]
    fn test_http_pattern_base_path() {
        let pattern = HttpPattern::parse("/catalogs/{name}");
        assert_eq!(pattern.base_path(), "catalogs");

        let pattern = HttpPattern::parse("/shares/{share}/schemas");
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

        let pattern = extract_http_rule_pattern(&http_rule).unwrap();
        assert_eq!(pattern.parameters, vec!["name"]);
        assert_eq!(pattern.template, "/catalogs/{name}");

        // Test POST pattern
        let http_rule = HttpRule {
            pattern: Some(Pattern::Post("/catalogs".to_string())),
            ..Default::default()
        };

        let pattern = extract_http_rule_pattern(&http_rule).unwrap();
        assert_eq!(pattern.parameters, Vec::<String>::new());
        assert_eq!(pattern.template, "/catalogs");

        // Test None pattern
        let http_rule = HttpRule {
            pattern: None,
            ..Default::default()
        };

        assert!(extract_http_rule_pattern(&http_rule).is_none());
    }

    #[test]
    fn test_should_be_body_field() {
        assert!(should_be_body_field("any_field", "*"));
        assert!(!should_be_body_field("any_field", ""));
        assert!(should_be_body_field("specific", "specific"));
        assert!(!should_be_body_field("other", "specific"));
    }
}
