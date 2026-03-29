use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::parsing::CodeGenMetadata;

use super::format_tokens;

/// Generate the `labels.rs` file containing `Resource` and `ObjectLabel` enums
/// derived from `google.api.resource` annotations on Unity Catalog message types.
pub(crate) fn generate_resource_enum(metadata: &CodeGenMetadata) -> String {
    // Collect all UC messages that have a resource annotation
    let mut resources: Vec<ResourceEntry> = metadata
        .messages
        .iter()
        .filter_map(|(name, info)| {
            let rd = info.resource_descriptor.as_ref()?;
            // Only include unitycatalog.* packages (exclude google/gnostic messages)
            if !name.starts_with(".unitycatalog.") {
                return None;
            }
            // Extract variant name from resource type (e.g. "unitycatalog.io/ExternalLocation" -> "ExternalLocation")
            let variant_name = rd.r#type.split('/').next_back()?.to_string();
            // Derive Rust type path from message name
            // e.g. ".unitycatalog.catalogs.v1.Catalog" -> "super::catalogs::v1::Catalog"
            let rust_path = message_name_to_rust_path(name)?;
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

    // Generate snake_case variant names for the strum rename mapping
    // strum(serialize_all = "snake_case") handles this, but we need to verify
    // ExternalLocation -> "external_location" which strum handles via snake_case

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

/// Convert a fully-qualified protobuf message name to a Rust type path relative to the
/// `resources::labels` module (two `super::` hops up into `models/`).
///
/// Examples:
/// - `.unitycatalog.catalogs.v1.Catalog` → `super::super::catalogs::v1::Catalog`
/// - `.unitycatalog.external_locations.v1.ExternalLocation` → `super::super::external_locations::v1::ExternalLocation`
fn message_name_to_rust_path(name: &str) -> Option<String> {
    // Strip leading `.unitycatalog.`
    let without_prefix = name.strip_prefix(".unitycatalog.")?;
    // Split remaining parts and join with `::`
    let parts: Vec<&str> = without_prefix.split('.').collect();
    if parts.is_empty() {
        return None;
    }
    Some(format!("super::super::{}", parts.join("::")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_name_to_rust_path() {
        assert_eq!(
            message_name_to_rust_path(".unitycatalog.catalogs.v1.Catalog"),
            Some("super::super::catalogs::v1::Catalog".to_string())
        );
        assert_eq!(
            message_name_to_rust_path(".unitycatalog.external_locations.v1.ExternalLocation"),
            Some("super::super::external_locations::v1::ExternalLocation".to_string())
        );
        assert_eq!(message_name_to_rust_path(".google.api.Something"), None);
    }
}
