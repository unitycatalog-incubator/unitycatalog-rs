/// Creates a nested column name whose field names are all simple resource names (containing only
/// alphanumeric characters and underscores), delimited by dots. This macro is provided as a
/// convenience for the common case where the caller knows the column name contains only simple
/// field names and that splitting by periods is safe:
///
/// ```
/// # use unitycatalog_common::{resource_name, ResourceName};
/// assert_eq!(resource_name!("a.b.c"), ResourceName::new(["a", "b", "c"]));
/// ```
///
/// To avoid accidental misuse, the argument must be a string literal, so the compiler can validate
/// the safety conditions. Thus, the following uses would fail to compile:
///
/// ```fail_compile
/// # use unitycatalog_common::resource_name;
/// let s = "a.b";
/// let name = resource_name!(s); // not a string literal
/// ```
///
/// ```fail_compile
/// # use unitycatalog_common::resource_name;
/// let name = resource_name!("a b"); // non-alphanumeric character
/// ```
// NOTE: Macros are only public if exported, which defines them at the root of the crate. But we
// don't want it there. So, we export a hidden macro and pub use it here where we actually want it.
#[macro_export]
#[doc(hidden)]
macro_rules! __resource_name {
    ( $($name:tt)* ) => {
        $crate::models::ResourceName::new($crate::derive::parse_column_name!($($name)*))
    };
}
#[doc(inline)]
pub use __resource_name as resource_name;

#[cfg(test)]
mod test {
    use crate::resource_name;
    use trestle_derive::parse_column_name;
    use trestle_store::ResourceName;

    #[test]
    fn test_parse_column_name_macros() {
        assert_eq!(parse_column_name!("a"), ["a"]);

        assert_eq!(parse_column_name!("a"), ["a"]);
        assert_eq!(parse_column_name!("a.b"), ["a", "b"]);
        assert_eq!(parse_column_name!("a.b.c"), ["a", "b", "c"]);
    }

    #[test]
    fn test_column_name_macros() {
        assert_eq!(resource_name!("a"), ResourceName::new(["a"]));
        assert_eq!(resource_name!("a.b"), ResourceName::new(["a", "b"]));
        assert_eq!(resource_name!("a.b.c"), ResourceName::new(["a", "b", "c"]));
    }
}
