use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::analysis::{GenerationPlan, RequestType};
use crate::google::api::FieldBehavior;
use crate::parsing::CodeGenMetadata;
use crate::parsing::types::BaseType;

use super::{CodeGenConfig, format_tokens};

/// Generate the `labels.rs` file containing `Resource` and `ObjectLabel` enums
/// derived from `google.api.resource` annotations on message types.
///
/// The package prefix is inferred from the service packages in `plan`: the longest
/// common dot-delimited prefix across all services, formatted as `".<prefix>."`.
/// The `super::` depth is always `1` since `labels.rs` is placed one level inside
/// the models subdirectory alongside the service `pub mod` blocks.
///
/// When `error_type_path` is `Some`, also emits:
/// - An inherent `Resource::resource_label()` method
/// - `From<T> for Resource` and `TryFrom<Resource> for T` impls for each resource type
///
/// When `config.generate_object_conversions` is `true`, also emits:
/// - A `::trestle_derive::object_conversions!` invocation for all resources
///   that have an `IDENTIFIER`-annotated field
/// - A `qualified_name()` inherent method on each resource type
pub(crate) fn generate_resource_enum(
    plan: &GenerationPlan,
    metadata: &CodeGenMetadata,
    config: &CodeGenConfig,
    error_type_path: Option<&str>,
) -> String {
    if !config.generate_resource_enum {
        return String::new();
    }

    // Infer package prefix from service packages (e.g. "unitycatalog.catalogs.v1" → ".unitycatalog.")
    let package_prefix = infer_package_prefix(
        &plan
            .services
            .iter()
            .map(|s| s.package.as_str())
            .collect::<Vec<_>>(),
    );

    // Collect all messages that have a resource annotation matching the inferred prefix
    let mut resources: Vec<ResourceEntry> = metadata
        .messages
        .iter()
        .filter_map(|(name, info)| {
            let rd = info.resource_descriptor.as_ref()?;
            // Only include packages matching the inferred prefix (excludes google/gnostic messages)
            if !name.starts_with(&package_prefix) {
                return None;
            }
            // Extract variant name from resource type (e.g. "unitycatalog.io/ExternalLocation" -> "ExternalLocation")
            let variant_name = rd.r#type.split('/').next_back()?.to_string();
            // labels.rs always lives one level inside the models subdir, so super:: reaches the subdir
            // module which has all the service pub mods as siblings.
            let rust_path = message_name_to_rust_path(name, &package_prefix, 1)?;

            // Find the IDENTIFIER-annotated field
            let id_field = info
                .fields
                .iter()
                .find(|f| f.field_behavior.contains(&FieldBehavior::Identifier));
            let (id_field_name, id_is_optional) = match id_field {
                Some(f) => (Some(f.name.clone()), f.unified_type.is_optional),
                None => (None, false),
            };

            // Derive path_names from the service plan for this resource.
            // A resource is hierarchical if its descriptor explicitly sets name_field = "full_name"
            // OR if the message has a full_name field (server-computed dot-joined composite).
            let message_has_full_name = info.fields.iter().any(|f| f.name == "full_name");
            let path_names = derive_path_names(
                &rd.singular,
                rd.name_field == "full_name" || message_has_full_name,
                plan,
            );

            // Compute field descriptors with roles for the resource registry.
            let known_managed_fields: &[&str] =
                &["created_at", "updated_at", "created_by", "updated_by"];
            let field_descriptors: Vec<FieldDescriptorEntry> = info
                .fields
                .iter()
                .map(|f| {
                    let role = if f.field_behavior.contains(&FieldBehavior::Identifier) {
                        FieldRoleEntry::Identifier
                    } else if f.is_sensitive {
                        FieldRoleEntry::Sensitive
                    } else if f.field_behavior.contains(&FieldBehavior::OutputOnly)
                        && known_managed_fields.contains(&f.name.as_str())
                    {
                        FieldRoleEntry::Managed
                    } else {
                        FieldRoleEntry::Data
                    };
                    FieldDescriptorEntry {
                        name: f.name.clone(),
                        role,
                    }
                })
                .collect();

            Some(ResourceEntry {
                variant_name,
                rust_path,
                singular: rd.singular.clone(),
                id_field: id_field_name,
                id_is_optional,
                path_names,
                has_full_name: message_has_full_name,
                field_descriptors,
            })
        })
        .collect();

    // Sort deterministically by singular name
    resources.sort_by(|a, b| a.singular.cmp(&b.singular));

    let resource_variants: Vec<TokenStream> = resources
        .iter()
        .map(|r| {
            let variant = format_ident!("{}", r.variant_name);
            let path: syn::Type = syn::parse_str(&r.rust_path)
                .unwrap_or_else(|e| panic!("Invalid rust path `{}`: {}", r.rust_path, e));
            quote! { #variant(#path) }
        })
        .collect();

    let label_variants: Vec<TokenStream> = resources
        .iter()
        .map(|r| {
            let variant = format_ident!("{}", r.variant_name);
            quote! { #variant }
        })
        .collect();

    // Inherent impl and From/TryFrom impls — only emitted when error_type_path is set
    let extra_impls: TokenStream = if let Some(error_path) = error_type_path {
        let error_ty: syn::Type = syn::parse_str(error_path)
            .unwrap_or_else(|e| panic!("Invalid error_type_path `{error_path}`: {e}"));

        let label_arms: Vec<TokenStream> = resources
            .iter()
            .map(|r| {
                let variant = format_ident!("{}", r.variant_name);
                quote! { Resource::#variant(_) => &ObjectLabel::#variant, }
            })
            .collect();

        let from_impls: Vec<TokenStream> = resources
            .iter()
            .map(|r| {
                let variant = format_ident!("{}", r.variant_name);
                let path: syn::Type = syn::parse_str(&r.rust_path)
                    .unwrap_or_else(|e| panic!("Invalid rust path `{}`: {}", r.rust_path, e));
                quote! {
                    impl From<#path> for Resource {
                        fn from(v: #path) -> Self {
                            Resource::#variant(v)
                        }
                    }

                    impl TryFrom<Resource> for #path {
                        type Error = #error_ty;

                        fn try_from(r: Resource) -> Result<Self, Self::Error> {
                            match r {
                                Resource::#variant(v) => Ok(v),
                                _ => Err(<#error_ty>::generic(concat!(
                                    "Resource is not a ",
                                    stringify!(#variant)
                                ))),
                            }
                        }
                    }
                }
            })
            .collect();

        quote! {
            impl Resource {
                /// Return the discriminant label for this resource.
                pub fn resource_label(&self) -> &ObjectLabel {
                    match self {
                        #(#label_arms)*
                    }
                }
            }

            #(#from_impls)*
        }
    } else {
        quote! {}
    };

    // object_conversions! macro invocation and qualified_name() methods
    let object_conversions_impl: TokenStream = if config.generate_object_conversions {
        let mut entries: Vec<TokenStream> = Vec::new();
        let mut qualified_name_impls: Vec<TokenStream> = Vec::new();

        for r in &resources {
            let Some(ref id_field) = r.id_field else {
                // No IDENTIFIER annotation — skip
                continue;
            };

            let path: syn::Type = syn::parse_str(&r.rust_path)
                .unwrap_or_else(|e| panic!("Invalid rust path `{}`: {}", r.rust_path, e));
            let label_expr: syn::Expr = syn::parse_str(&format!("ObjectLabel::{}", r.variant_name))
                .unwrap_or_else(|e| panic!("Invalid label expr: {e}"));
            let id_ident = format_ident!("{}", id_field);
            let is_optional = r.id_is_optional;

            let path_name_idents: Vec<proc_macro2::Ident> = r
                .path_names
                .iter()
                .map(|n| format_ident!("{}", n))
                .collect();

            entries.push(quote! {
                #path, #label_expr, #id_ident, [#(#path_name_idents),*], #is_optional
            });

            // qualified_name() impl
            let format_expr: TokenStream = build_qualified_name_expr(&r.path_names);
            qualified_name_impls.push(quote! {
                impl #path {
                    /// Returns the fully-qualified dot-separated name computed from component fields.
                    pub fn qualified_name(&self) -> String {
                        #format_expr
                    }
                }
            });
        }

        let derive_crate = format_ident!("{}", config.derive_crate_name);
        quote! {
            use crate::Error;
            use crate::models::object::Object;
            use crate::models::resources::{ResourceExt, ResourceIdent, ResourceName, ResourceRef};

            ::#derive_crate::object_conversions!(
                #(#entries);*
            );

            #(#qualified_name_impls)*
        }
    } else {
        quote! {}
    };

    let tokens = quote! {
        /// All resource types managed by Unity Catalog.
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, Debug, PartialEq)]
        pub enum Resource {
            #(#resource_variants),*
        }

        /// Discriminant label for each resource type.
        #[derive(
            ::strum::AsRefStr,
            ::strum::Display,
            ::strum::EnumIter,
            ::strum::EnumString,
            ::serde::Serialize,
            ::serde::Deserialize,
            Hash,
            Clone,
            Copy,
            Debug,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
        )]
        #[strum(serialize_all = "snake_case", ascii_case_insensitive)]
        #[serde(rename_all = "snake_case")]
        #[cfg_attr(feature = "sqlx", derive(::sqlx::Type))]
        #[cfg_attr(
            feature = "sqlx",
            sqlx(type_name = "object_label", rename_all = "snake_case")
        )]
        pub enum ObjectLabel {
            #(#label_variants),*
        }

        #extra_impls

        #object_conversions_impl
    };

    // Generate the resource descriptor registry and Label impl
    let registry_impl = generate_resource_registry(&resources, config);

    let all_tokens = quote! {
        #tokens

        #registry_impl
    };

    format_tokens(all_tokens)
}

struct ResourceEntry {
    variant_name: String,
    rust_path: String,
    singular: String,
    /// Field name carrying `FieldBehavior::Identifier`, if present.
    id_field: Option<String>,
    /// Whether the IDENTIFIER field is `optional`.
    id_is_optional: bool,
    /// Ordered list of field names used to build `ResourceName`, e.g. `["catalog_name", "schema_name", "name"]`.
    path_names: Vec<String>,
    /// Whether the message has a `full_name` field (used for `qualified_name()` generation).
    #[allow(dead_code)]
    has_full_name: bool,
    /// All fields with their computed roles for the resource descriptor registry.
    field_descriptors: Vec<FieldDescriptorEntry>,
}

/// A field entry for the generated resource descriptor registry.
struct FieldDescriptorEntry {
    name: String,
    role: FieldRoleEntry,
}

/// The computed role of a field, matching `trestle_store::FieldRole`.
enum FieldRoleEntry {
    Data,
    Identifier,
    Sensitive,
    Managed,
}

/// Derive the ordered list of field names used to build a `ResourceName` for a resource.
///
/// Uses the same two-signal logic as `derive_resource_accessor_params` in the Python codegen:
/// 1. `name_field = "full_name"` on the descriptor → resource has decomposable composite name
/// 2. Check the List method's required string-typed query params for parent names
///
/// Returns e.g. `["catalog_name", "schema_name", "name"]` for Table,
/// `["catalog_name", "name"]` for Schema, `["name"]` for Catalog.
fn derive_path_names(
    singular: &str,
    has_full_name_field: bool,
    plan: &GenerationPlan,
) -> Vec<String> {
    // Find the service whose singular resource name matches
    let service = plan.services.iter().find(|s| {
        s.managed_resources
            .iter()
            .any(|r| r.descriptor.singular == singular)
    });

    let Some(service) = service else {
        return vec!["name".to_string()];
    };

    // Get the Get method's path param name
    let get_path_param = service
        .methods
        .iter()
        .find(|m| m.request_type == RequestType::Get)
        .and_then(|m| m.path_parameters().next().map(|p| p.name.clone()));

    // Get the List method's required string query params (these are the parent hierarchy params)
    let parent_params: Vec<String> = service
        .methods
        .iter()
        .find(|m| m.request_type == RequestType::List)
        .map(|m| {
            m.parameters
                .iter()
                .filter(|p| !p.is_path_param() && !p.is_optional())
                .filter(|p| matches!(p.field_type().base_type, BaseType::String))
                .map(|p| p.name().to_string())
                .collect()
        })
        .unwrap_or_default();

    let should_decompose = has_full_name_field
        || (get_path_param.as_deref() == Some("name") && !parent_params.is_empty());

    if should_decompose {
        let mut params = parent_params;
        params.push(format!("{singular}_name"));
        // Replace the final `{singular}_name` with just `name` since the proto field is always `name`
        let last = params.last_mut().unwrap();
        *last = "name".to_string();
        params
    } else {
        vec!["name".to_string()]
    }
}

/// Build a `qualified_name()` return expression from an ordered list of path field names.
///
/// - `["name"]` → `self.name.clone()`
/// - `["catalog_name", "name"]` → `format!("{}.{}", self.catalog_name, self.name)`
/// - `["catalog_name", "schema_name", "name"]` → `format!("{}.{}.{}", ...)`
fn build_qualified_name_expr(path_names: &[String]) -> TokenStream {
    if path_names.len() == 1 {
        let field = format_ident!("{}", &path_names[0]);
        return quote! { self.#field.clone() };
    }
    let format_str = path_names
        .iter()
        .map(|_| "{}")
        .collect::<Vec<_>>()
        .join(".");
    let field_refs: Vec<TokenStream> = path_names
        .iter()
        .map(|n| {
            let ident = format_ident!("{}", n);
            quote! { self.#ident }
        })
        .collect();
    quote! { format!(#format_str, #(#field_refs),*) }
}

/// Infer the package prefix from a list of proto package names.
///
/// Finds the longest common leading dot-segment and returns it as `".<prefix>."`.
///
/// Examples:
/// - `["unitycatalog.catalogs.v1", "unitycatalog.tables.v1"]` → `".unitycatalog."`
/// - `["example.catalog.v1"]` → `".example."`
fn infer_package_prefix(packages: &[&str]) -> String {
    if packages.is_empty() {
        return String::new();
    }
    let first_parts: Vec<&str> = packages[0].split('.').collect();
    let common_len = first_parts
        .iter()
        .enumerate()
        .take_while(|(i, seg)| {
            packages
                .iter()
                .skip(1)
                .all(|p| p.split('.').nth(*i) == Some(seg))
        })
        .count();
    // Take only the top-level shared segment (one dot-level), not the full common prefix,
    // so version segments like "v1" don't get included when all packages share them.
    // Use the first segment as the meaningful namespace prefix.
    let prefix_seg = if common_len > 0 {
        first_parts[0]
    } else {
        first_parts[0]
    };
    format!(".{}.", prefix_seg)
}

/// Convert a fully-qualified protobuf message name to a Rust type path relative to
/// `labels.rs` inside the models subdirectory.
///
/// `prefix` is stripped from the message name (e.g. `".unitycatalog."`).
/// One `super::` hop is prepended since `labels.rs` is a sibling of the service modules
/// inside the same generated subdirectory.
///
/// Examples (prefix = `".unitycatalog."`):
/// - `.unitycatalog.catalogs.v1.Catalog` → `super::catalogs::v1::Catalog`
/// - `.unitycatalog.external_locations.v1.ExternalLocation` → `super::external_locations::v1::ExternalLocation`
fn message_name_to_rust_path(name: &str, prefix: &str, super_levels: u32) -> Option<String> {
    // Strip leading prefix (e.g. ".unitycatalog.")
    let without_prefix = name.strip_prefix(prefix)?;
    // Split remaining parts and join with `::`
    let parts: Vec<&str> = without_prefix.split('.').collect();
    if parts.is_empty() {
        return None;
    }
    let super_prefix = "super::".repeat(super_levels as usize);
    Some(format!("{}{}", super_prefix, parts.join("::")))
}

/// Generate the `RESOURCE_DESCRIPTORS` static registry and `Label` impl for `ObjectLabel`.
///
/// This emits:
/// 1. `impl trestle_store::Label for ObjectLabel` — making the generated
///    label type compatible with the generic resource store.
/// 2. `pub static RESOURCE_DESCRIPTORS: &[ResourceTypeDescriptor]` — a static registry
///    of all resource types with field roles, path names, and parent relationships.
fn generate_resource_registry(resources: &[ResourceEntry], config: &CodeGenConfig) -> TokenStream {
    let store_crate = format_ident!("{}", config.resource_store_crate_name);

    // --- Label impl for ObjectLabel ---
    let label_impl = quote! {
        impl ::#store_crate::Label for ObjectLabel {
            fn as_str(&self) -> &str {
                // strum's AsRefStr gives us the snake_case string
                self.as_ref()
            }
        }
    };

    // --- RESOURCE_DESCRIPTORS static ---
    // Compute parent_label for each resource by cross-referencing path_names.
    // A resource with path_names ["catalog_name", "schema_name", "name"] has parent
    // equal to the resource whose path_names length is one less and whose singular
    // name matches the second-to-last path component (minus the "_name" suffix).
    let parent_labels: Vec<Option<String>> = resources
        .iter()
        .map(|r| {
            if r.path_names.len() <= 1 {
                return None;
            }
            // The second-to-last path component holds the parent's singular name + "_name"
            let parent_path_component = &r.path_names[r.path_names.len() - 2];
            let parent_singular = parent_path_component
                .strip_suffix("_name")
                .unwrap_or(parent_path_component);
            // Find the resource with that singular name
            resources.iter().find_map(|candidate| {
                if candidate.singular == parent_singular {
                    Some(candidate.variant_name.clone())
                } else {
                    None
                }
            })
        })
        .collect();

    let descriptor_entries: Vec<TokenStream> = resources
        .iter()
        .zip(parent_labels.iter())
        .map(|(r, parent)| {
            let label_variant = format_ident!("{}", r.variant_name);

            let field_entries: Vec<TokenStream> = r
                .field_descriptors
                .iter()
                .map(|fd| {
                    let name = &fd.name;
                    let role = match fd.role {
                        FieldRoleEntry::Data => {
                            quote! { ::#store_crate::FieldRole::Data }
                        }
                        FieldRoleEntry::Identifier => {
                            quote! { ::#store_crate::FieldRole::Identifier }
                        }
                        FieldRoleEntry::Sensitive => {
                            quote! { ::#store_crate::FieldRole::Sensitive }
                        }
                        FieldRoleEntry::Managed => {
                            quote! { ::#store_crate::FieldRole::Managed }
                        }
                    };
                    quote! {
                        ::#store_crate::ResourceFieldDescriptor {
                            name: #name,
                            role: #role,
                        }
                    }
                })
                .collect();

            let path_name_strs: Vec<&str> = r.path_names.iter().map(|s| s.as_str()).collect();

            let parent_expr = match parent {
                Some(parent_name) => {
                    let parent_variant = format_ident!("{}", parent_name);
                    quote! { Some(ObjectLabel::#parent_variant) }
                }
                None => quote! { None },
            };

            quote! {
                ::#store_crate::ResourceTypeDescriptor {
                    label: ObjectLabel::#label_variant,
                    fields: &[#(#field_entries),*],
                    path_names: &[#(#path_name_strs),*],
                    parent_label: #parent_expr,
                }
            }
        })
        .collect();

    let registry = quote! {
        /// Static resource type descriptors derived from proto annotations.
        ///
        /// Each entry describes a resource type's fields (with roles: data, identifier,
        /// sensitive, managed), hierarchical name components, and parent relationship.
        ///
        /// Use `ResourceRegistry::from_static` to build a runtime registry from this data.
        pub static RESOURCE_DESCRIPTORS: &[::#store_crate::ResourceTypeDescriptor<ObjectLabel>] = &[
            #(#descriptor_entries),*
        ];
    };

    quote! {
        #label_impl
        #registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_name_to_rust_path() {
        assert_eq!(
            message_name_to_rust_path(".unitycatalog.catalogs.v1.Catalog", ".unitycatalog.", 1),
            Some("super::catalogs::v1::Catalog".to_string())
        );
        assert_eq!(
            message_name_to_rust_path(
                ".unitycatalog.external_locations.v1.ExternalLocation",
                ".unitycatalog.",
                1
            ),
            Some("super::external_locations::v1::ExternalLocation".to_string())
        );
        assert_eq!(
            message_name_to_rust_path(".google.api.Something", ".unitycatalog.", 1),
            None
        );
    }

    #[test]
    fn test_infer_package_prefix() {
        assert_eq!(
            infer_package_prefix(&["unitycatalog.catalogs.v1", "unitycatalog.tables.v1"]),
            ".unitycatalog."
        );
        assert_eq!(infer_package_prefix(&["example.catalog.v1"]), ".example.");
        assert_eq!(
            infer_package_prefix(&["example.catalog.v1", "example.items.v1"]),
            ".example."
        );
    }

    #[test]
    fn test_build_qualified_name_expr_flat() {
        let expr = build_qualified_name_expr(&["name".to_string()]);
        let s = expr.to_string();
        assert!(s.contains("self"), "expr: {s}");
        assert!(s.contains("name"), "expr: {s}");
        assert!(s.contains("clone"), "expr: {s}");
    }

    #[test]
    fn test_build_qualified_name_expr_hierarchical() {
        let expr = build_qualified_name_expr(&[
            "catalog_name".to_string(),
            "schema_name".to_string(),
            "name".to_string(),
        ]);
        let s = expr.to_string();
        assert!(s.contains("format"), "expr: {s}");
        assert!(s.contains("catalog_name"), "expr: {s}");
        assert!(s.contains("schema_name"), "expr: {s}");
    }
}
