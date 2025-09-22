use std::collections::HashSet;

use convert_case::{Case, Casing};
use quote::format_ident;
use syn::Ident;

use crate::analysis::messages::MessageRegistry;
use crate::error::{Error, Result};
use crate::google::api::{ResourceDescriptor, http_rule::Pattern};
use crate::parsing::types::UnifiedType;
use crate::parsing::{HttpPattern, MethodMetadata};

/// The Operation a method is performing
///
/// There are standard CRUD operations, as well as custom operations.
///
/// standard operations on collections are:
/// - List: Retrieve a list of resources
/// - Create: Create a new resource
///
/// standard operations on individual resources are:
/// - Get: Retrieve a single resource
/// - Update: Update an existing resource
/// - Delete: Delete a resource
///
/// Custom operations are:
/// - Custom(String): Custom operation with a name
#[derive(Debug, Clone, PartialEq)]
pub enum RequestType {
    List,
    Create,
    Get,
    Update,
    Delete,
    Custom(Pattern),
}

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
    /// Resources managed by this service
    pub managed_resources: Vec<ManagedResource>,
}

/// Plan for generating code for a single method
#[derive(Debug, Clone)]
pub(crate) struct MethodPlan {
    /// Original method metadata
    pub metadata: MethodMetadata,
    /// Rust function name for the handler method
    pub handler_function_name: String,
    pub http_pattern: HttpPattern,
    /// HTTP method and path for routing
    pub http_method: String,
    /// parameters passed to the method
    pub parameters: Vec<RequestParam>,
    /// Whether this method returns a response body
    pub has_response: bool,
    /// Request type for this method
    pub request_type: RequestType,
    /// The resource type name returned by this method (if any)
    pub output_resource_type: Option<String>,
}

impl MethodPlan {
    pub fn path_parameters(&self) -> impl Iterator<Item = &PathParam> {
        self.parameters.iter().filter_map(|param| match param {
            RequestParam::Path(path_param) => Some(path_param),
            _ => None,
        })
    }

    pub fn query_parameters(&self) -> impl Iterator<Item = &QueryParam> {
        self.parameters.iter().filter_map(|param| match param {
            RequestParam::Query(query_param) => Some(query_param),
            _ => None,
        })
    }

    pub fn body_fields(&self) -> impl Iterator<Item = &BodyField> {
        self.parameters.iter().filter_map(|param| match param {
            RequestParam::Body(body_field) => Some(body_field),
            _ => None,
        })
    }
}

#[derive(Debug, Clone)]
pub enum RequestParam {
    Path(PathParam),
    Query(QueryParam),
    Body(BodyField),
}

impl RequestParam {
    pub fn name(&self) -> &str {
        match self {
            RequestParam::Path(param) => &param.field_name,
            RequestParam::Query(param) => &param.name,
            RequestParam::Body(param) => &param.name,
        }
    }

    pub fn field_type(&self) -> &UnifiedType {
        match self {
            RequestParam::Path(param) => &param.field_type,
            RequestParam::Query(param) => &param.field_type,
            RequestParam::Body(param) => &param.field_type,
        }
    }

    pub fn field_ident(&self) -> Ident {
        format_ident!("{}", self.name())
    }

    pub fn is_optional(&self) -> bool {
        match self {
            RequestParam::Path(_) => false,
            RequestParam::Query(param) => param.is_optional(),
            RequestParam::Body(param) => param.is_optional(),
        }
    }

    pub fn is_path_param(&self) -> bool {
        matches!(self, RequestParam::Path(_))
    }
}

/// A path parameter in a URL template
#[derive(Debug, Clone)]
pub struct PathParam {
    /// Field name in the request struct (e.g., "full_name")
    pub field_name: String,
    /// Parsed type of the path parameter
    pub field_type: UnifiedType,
}

impl From<PathParam> for RequestParam {
    fn from(param: PathParam) -> Self {
        RequestParam::Path(param)
    }
}

/// A query parameter for HTTP requests
#[derive(Debug, Clone)]
pub struct QueryParam {
    /// Parameter name
    pub name: String,
    /// Parsed type of the query parameter
    pub field_type: UnifiedType,
}

impl QueryParam {
    /// denotes if the parameter is optional
    pub fn is_optional(&self) -> bool {
        self.field_type.is_optional
    }
}

impl From<QueryParam> for RequestParam {
    fn from(param: QueryParam) -> Self {
        RequestParam::Query(param)
    }
}

/// A body field that should be extracted from request body
#[derive(Debug, Clone)]
pub struct BodyField {
    /// Field name
    pub name: String,
    /// Whether this field is optional
    pub optional: bool,
    /// Parsed type of the query parameter
    pub field_type: UnifiedType,
}

impl BodyField {
    /// denotes if the parameter is optional
    pub fn is_optional(&self) -> bool {
        self.optional
    }
}

impl From<BodyField> for RequestParam {
    fn from(field: BodyField) -> Self {
        RequestParam::Body(field)
    }
}

/// Information about a resource managed by a service
#[derive(Debug, Clone)]
pub struct ManagedResource {
    /// Resource type name (e.g., "Catalog")
    pub type_name: String,
    /// Resource descriptor information
    pub descriptor: ResourceDescriptor,
}

pub(super) struct MethodPlanner<'a> {
    method: &'a MethodMetadata,
    pattern: Pattern,
    pub(super) path: HttpPattern,
    registry: &'a MessageRegistry<'a>,
}

impl<'a> MethodPlanner<'a> {
    pub fn try_new(method: &'a MethodMetadata, registry: &'a MessageRegistry<'a>) -> Result<Self> {
        let Some(pattern) = &method.http_rule.pattern else {
            return Err(Error::MissingAnnotation {
                object: method.method_name.clone(),
                message: "Missing HTTP rule pattern".to_string(),
            });
        };
        let raw_path = match pattern {
            Pattern::Get(p) => p.clone(),
            Pattern::Post(p) => p.clone(),
            Pattern::Put(p) => p.clone(),
            Pattern::Delete(p) => p.clone(),
            Pattern::Patch(p) => p.clone(),
            Pattern::Custom(p) => p.path.clone(),
        };
        let path = HttpPattern::parse(&raw_path);
        Ok(Self {
            method,
            path,
            pattern: pattern.clone(),
            registry,
        })
    }

    /// Determines if the rpc is a standard get method
    ///
    /// Tests largely on conditions mentioned in [API-131].
    ///
    /// [API-131]: https://google.aip.dev/131
    pub fn is_standard_get(&self) -> bool {
        let snake_name = self.method.method_name.to_case(Case::Snake);
        let Some((verb, resource)) = snake_name.split_once("_") else {
            return false;
        };
        if !matches!(self.pattern, Pattern::Get(_)) || verb != "get" || self.path.ends_with_static()
        {
            return false;
        }
        self.registry.resource_from_singular(resource).is_some()
    }

    /// Determines if the rpc is a standard list method
    ///
    /// Tests largely on conditions mentioned in [API-132].
    ///
    /// [API-132]: https://google.aip.dev/132
    pub fn is_standard_list(&self) -> bool {
        let snake_name = self.method.method_name.to_case(Case::Snake);
        let Some((verb, resource)) = snake_name.split_once("_") else {
            return false;
        };
        if !matches!(self.pattern, Pattern::Get(_))
            || verb != "list"
            || self.path.ends_with_parameter()
        {
            return false;
        }
        self.registry.resource_from_plural(resource).is_some()
    }

    /// Determines if the rpc is a standard create method
    ///
    /// Tests largely on conditions mentioned in [API-133].
    ///
    /// [API-133]: https://google.aip.dev/133
    pub fn is_standard_create(&self) -> bool {
        let snake_name = self.method.method_name.to_case(Case::Snake);
        let Some((verb, resource)) = snake_name.split_once("_") else {
            return false;
        };
        if !matches!(self.pattern, Pattern::Post(_))
            || verb != "create"
            || self.path.ends_with_parameter()
        {
            return false;
        }
        self.registry.resource_from_singular(resource).is_some()
    }

    /// Determines if the rpc is a standard update method
    ///
    /// Tests largely on conditions mentioned in [API-134].
    ///
    /// [API-134]: https://google.aip.dev/134
    pub fn is_standard_update(&self) -> bool {
        let snake_name = self.method.method_name.to_case(Case::Snake);
        let Some((verb, resource)) = snake_name.split_once("_") else {
            return false;
        };
        if !matches!(self.pattern, Pattern::Patch(_))
            || verb != "update"
            || self.path.ends_with_static()
        {
            return false;
        }
        self.registry.resource_from_singular(resource).is_some()
    }

    /// Determines if the rpc is a standard delete method
    ///
    /// Tests largely on conditions mentioned in [API-135].
    ///
    /// [API-135]: https://google.aip.dev/135
    pub fn is_standard_delete(&self) -> bool {
        let snake_name = self.method.method_name.to_case(Case::Snake);
        let Some((verb, resource)) = snake_name.split_once("_") else {
            return false;
        };
        if !matches!(self.pattern, Pattern::Delete(_))
            || verb != "delete"
            || self.path.ends_with_static()
        {
            return false;
        }
        self.registry.resource_from_singular(resource).is_some()
    }

    pub fn request_type(&self) -> RequestType {
        match &self.pattern {
            Pattern::Get(_) if self.is_standard_get() => RequestType::Get,
            Pattern::Get(_) if self.is_standard_list() => RequestType::List,
            Pattern::Post(_) if self.is_standard_create() => RequestType::Create,
            Pattern::Patch(_) if self.is_standard_update() => RequestType::Update,
            Pattern::Delete(_) if self.is_standard_delete() => RequestType::Delete,
            Pattern::Get(_) => RequestType::Custom(self.pattern.clone()),
            Pattern::Post(_) => RequestType::Custom(self.pattern.clone()),
            Pattern::Delete(_) => RequestType::Custom(self.pattern.clone()),
            Pattern::Patch(_) => RequestType::Custom(self.pattern.clone()),
            Pattern::Custom(_) => todo!("Implement custom request type"),
            Pattern::Put(_) => todo!("Implement PUT request type"),
        }
    }

    pub fn has_response(&self) -> bool {
        !self.method.output_type.is_empty() && !self.method.output_type.ends_with("Empty")
    }

    /// Extract the resource type name from the method's output type
    pub fn output_resource_type(&self) -> Option<String> {
        if self.has_response() {
            // Remove leading dot and package prefix to get just the type name
            let output_type = &self.method.output_type;
            if let Some(last_dot) = output_type.rfind('.') {
                Some(output_type[last_dot + 1..].to_string())
            } else {
                Some(output_type.clone())
            }
        } else {
            None
        }
    }
}

/// Extract managed resources from service methods
pub fn extract_managed_resources(
    registry: &MessageRegistry<'_>,
    methods: &[MethodPlan],
) -> Vec<ManagedResource> {
    let mut resources = Vec::new();
    let mut seen_types = HashSet::<String>::new();

    for method in methods {
        if let Some(ref resource_type) = method.output_resource_type {
            // Skip if we've already processed this resource type
            if seen_types.contains(resource_type) {
                continue;
            }

            // Look up the resource descriptor for this type
            if let Some(descriptor) = registry.get_resource_descriptor(resource_type) {
                resources.push(ManagedResource {
                    type_name: resource_type.clone(),
                    descriptor: descriptor.clone(),
                });
                seen_types.insert(resource_type.clone());
            }
        }
    }

    resources
}
