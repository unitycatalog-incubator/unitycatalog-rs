use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::str::FromStr;

/// A type-safe label for resource types.
///
/// Implementations discriminate between different kinds of resources in the store.
/// Typically generated from protobuf `google.api.resource` annotations (e.g., `ObjectLabel`).
///
/// The label is used as a discriminant in the `Object` type and for routing
/// operations to the correct backend or handler.
pub trait Label:
    Display + Debug + FromStr + Hash + Eq + Clone + Copy + Send + Sync + 'static
{
    /// Returns the string representation of this label.
    fn as_str(&self) -> &str;
}
