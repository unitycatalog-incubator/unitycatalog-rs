use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::analysis::GenerationPlan;
use crate::parsing::CodeGenMetadata;

use super::{CodeGenConfig, format_tokens};

/// Generate the `labels.rs` file containing `Resource` and `ObjectLabel` enums
/// derived from `google.api.resource` annotations on message types.
///
/// The package prefix is inferred from the service packages in `plan`: the longest
/// common dot-delimited prefix across all services, formatted as `".<prefix>."`.
/// The `super::` depth is always `1` since `labels.rs` is placed one level inside
/// the models subdirectory alongside the service `pub mod` blocks.
pub(crate) fn generate_resource_enum(
    plan: &GenerationPlan,
    metadata: &CodeGenMetadata,
    config: &CodeGenConfig,
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
            Some(ResourceEntry {
                variant_name,
                rust_path,
                singular: rd.singular.clone(),
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
    };

    format_tokens(tokens)
}

struct ResourceEntry {
    variant_name: String,
    rust_path: String,
    singular: String,
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
}
