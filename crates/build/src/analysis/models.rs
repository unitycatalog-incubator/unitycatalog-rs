use crate::parsing::MethodMetadata;

/// High-level plan for what code to generate
#[derive(Debug)]
pub struct GenerationPlan {
    /// Services to generate handlers for
    pub services: Vec<ServicePlan>,
}

/// Plan for generating code for a single service
#[derive(Debug, Clone)]
pub struct ServicePlan {
    /// Service name (e.g., "CatalogsService")
    pub service_name: String,
    /// Handler trait name (e.g., "CatalogHandler")
    pub handler_name: String,
    /// Base URL path for this service (e.g., "catalogs")
    pub base_path: String,
    /// Methods to generate for this service
    pub methods: Vec<MethodPlan>,
}

/// Plan for generating code for a single method
#[derive(Debug, Clone)]
pub struct MethodPlan {
    /// Original method metadata
    pub metadata: MethodMetadata,
    /// Rust function name for the handler method
    pub handler_function_name: String,
    /// Rust function name for the route handler
    pub route_function_name: String,
    /// HTTP method and path for routing
    pub http_method: String,
    pub http_path: String,
    /// Path parameters extracted from the URL template
    pub path_params: Vec<PathParam>,
    /// Query parameters (for List operations)
    pub query_params: Vec<QueryParam>,
    /// Body fields that should be extracted from request body
    pub body_fields: Vec<BodyField>,
    /// Whether this method returns a response body
    pub has_response: bool,
}

/// A path parameter in a URL template
#[derive(Debug, Clone)]
pub struct PathParam {
    /// Template parameter name (e.g., "name" from "/catalogs/{name}")
    pub template_param: String,
    /// Field name in the request struct (e.g., "full_name")
    pub field_name: String,
    /// Rust type for this parameter
    pub rust_type: String,
}

/// A query parameter for HTTP requests
#[derive(Debug, Clone)]
pub struct QueryParam {
    /// Parameter name
    pub name: String,
    /// Rust type for this parameter
    pub rust_type: String,
    /// Whether this parameter is optional
    pub optional: bool,
}

/// A body field that should be extracted from request body
#[derive(Debug, Clone)]
pub struct BodyField {
    /// Field name
    pub name: String,
    /// Rust type for this field
    pub rust_type: String,
    /// Whether this field is optional
    pub optional: bool,
}
