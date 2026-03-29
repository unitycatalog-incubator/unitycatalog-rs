use std::collections::HashSet;

use convert_case::{Case, Casing};
use quote::format_ident;
use syn::Ident;

use crate::error::{Error, Result};
use crate::google::api::{ResourceDescriptor, http_rule::Pattern};
use crate::parsing::types::UnifiedType;
use crate::parsing::{CodeGenMetadata, HttpPattern, MethodMetadata, OneofVariant};

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
    /// Documentation from protobuf service comments
    pub documentation: Option<String>,
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

    pub fn documentation(&self) -> Option<&str> {
        match self {
            RequestParam::Path(param) => param.documentation.as_deref(),
            RequestParam::Query(param) => param.documentation.as_deref(),
            RequestParam::Body(param) => param.documentation.as_deref(),
        }
    }
}

/// A path parameter in a URL template
#[derive(Debug, Clone)]
pub struct PathParam {
    /// Field name in the request struct (e.g., "full_name")
    pub field_name: String,
    /// Parsed type of the path parameter
    pub field_type: UnifiedType,
    /// Documentation from protobuf field comments
    pub documentation: Option<String>,
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
    /// Documentation from protobuf field comments
    pub documentation: Option<String>,
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
    /// Parsed type of the body parameter
    pub field_type: UnifiedType,
    /// Whether this field is a repeated (Vec) type
    pub repeated: bool,
    /// For oneof fields, the variants with their names and types
    pub oneof_variants: Option<Vec<OneofVariant>>,
    /// Documentation from protobuf field comments
    pub documentation: Option<String>,
}

impl BodyField {
    /// Denotes whether this field should be treated as optional in builder APIs.
    ///
    /// A field is optional when its `UnifiedType.is_optional` flag is set, when it is
    /// repeated, or when its base type is `Map`, `Message`, or `OneOf` (complex types
    /// always have a valid default and are therefore optional constructor parameters).
    pub fn is_optional(&self) -> bool {
        use crate::parsing::types::BaseType;
        self.field_type.is_optional
            || self.repeated
            || matches!(
                self.field_type.base_type,
                BaseType::Map(_, _) | BaseType::Message(_) | BaseType::OneOf(_)
            )
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
    metadata: &'a CodeGenMetadata,
}

impl<'a> MethodPlanner<'a> {
    pub fn try_new(method: &'a MethodMetadata, metadata: &'a CodeGenMetadata) -> Result<Self> {
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
            metadata,
        })
    }

    /// Classify the RPC as a standard CRUD operation per Google AIP 131-135.
    ///
    /// Each standard operation is identified by matching (verb, HTTP method, path shape,
    /// resource lookup). See:
    /// - [AIP-131](https://google.aip.dev/131) Get
    /// - [AIP-132](https://google.aip.dev/132) List
    /// - [AIP-133](https://google.aip.dev/133) Create
    /// - [AIP-134](https://google.aip.dev/134) Update
    /// - [AIP-135](https://google.aip.dev/135) Delete
    pub fn request_type(&self) -> RequestType {
        let snake_name = self.method.method_name.to_case(Case::Snake);
        let verb_resource = snake_name.split_once('_');

        if let Some((verb, resource)) = verb_resource {
            // Table of (verb, expected pattern, path must end with parameter?, lookup by plural?)
            let standard_ops: &[(&str, fn(&Pattern) -> bool, bool, bool, RequestType)] = &[
                (
                    "get",
                    |p| matches!(p, Pattern::Get(_)),
                    true,
                    false,
                    RequestType::Get,
                ),
                (
                    "list",
                    |p| matches!(p, Pattern::Get(_)),
                    false,
                    true,
                    RequestType::List,
                ),
                (
                    "create",
                    |p| matches!(p, Pattern::Post(_)),
                    false,
                    false,
                    RequestType::Create,
                ),
                (
                    "update",
                    |p| matches!(p, Pattern::Patch(_)),
                    true,
                    false,
                    RequestType::Update,
                ),
                (
                    "delete",
                    |p| matches!(p, Pattern::Delete(_)),
                    true,
                    false,
                    RequestType::Delete,
                ),
            ];

            for &(expected_verb, pattern_check, ends_with_param, use_plural, ref result_type) in
                standard_ops
            {
                if verb != expected_verb || !pattern_check(&self.pattern) {
                    continue;
                }
                if ends_with_param && self.path.ends_with_static() {
                    continue;
                }
                if !ends_with_param && self.path.ends_with_parameter() {
                    continue;
                }
                let found = if use_plural {
                    self.metadata.resource_from_plural(resource).is_some()
                } else {
                    self.metadata.resource_from_singular(resource).is_some()
                };
                if found {
                    return result_type.clone();
                }
            }
        }

        RequestType::Custom(self.pattern.clone())
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

/// Split body fields from a `MethodPlan` into required and optional subsets.
///
/// Delegates to [`BodyField::is_optional`] for the classification. Optional fields
/// become `with_*` setter methods; required fields become constructor parameters.
pub fn split_body_fields(plan: &MethodPlan) -> (Vec<&BodyField>, Vec<&BodyField>) {
    let mut required = Vec::new();
    let mut optional = Vec::new();
    for field in plan.body_fields() {
        if field.is_optional() {
            optional.push(field);
        } else {
            required.push(field);
        }
    }
    (required, optional)
}

/// Extract managed resources from service methods
pub fn extract_managed_resources(
    metadata: &CodeGenMetadata,
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
            if let Some(descriptor) = metadata.get_resource_descriptor(resource_type) {
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
