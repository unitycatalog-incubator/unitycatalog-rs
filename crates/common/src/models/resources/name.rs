/// Creates a nested column name whose field names are all simple resource names (containing only
/// alphanumeric characters and underscores), delimited by dots. This macro is provided as a
/// convenience for the common case where the caller knows the column name contains only simple
/// field names and that splitting by periods is safe:
///
/// ```
/// # use unitycatalog_common::{resource_name, ResourceName};
/// assert_eq!(resource_name!("a.b.c"), ResourceName::new(["a", "b", "c"]));
/// ```
// NOTE: Macros are only public if exported, which defines them at the root of the crate. But we
// don't want it there. So, we export a hidden macro and pub use it here where we actually want it.
#[macro_export]
#[doc(hidden)]
macro_rules! __resource_name {
    ( $name:literal ) => {{
        const _: () = {
            let s: &str = $name;
            let bytes = s.as_bytes();
            let mut i = 0;
            while i < bytes.len() {
                let b = bytes[i];
                assert!(
                    b.is_ascii_alphanumeric() || b == b'_' || b == b'.',
                    "resource_name! contains invalid character (only alphanumeric, '_', '.' allowed)"
                );
                i += 1;
            }
        };
        $crate::models::ResourceName::from_naive_str_split($name)
    }};
}
#[doc(inline)]
pub use __resource_name as resource_name;

#[cfg(test)]
mod test {
    use crate::resource_name;
    use olai_store::ResourceName;

    #[test]
    fn test_column_name_macros() {
        assert_eq!(resource_name!("a"), ResourceName::new(["a"]));
        assert_eq!(resource_name!("a.b"), ResourceName::new(["a", "b"]));
        assert_eq!(resource_name!("a.b.c"), ResourceName::new(["a", "b", "c"]));
    }
}
