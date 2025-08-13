use quote::{quote, quote_spanned};
use syn::{Error, parse_macro_input};

use conversions::{ObjectDefs, from_object, resource_impl, to_object, to_resource};

mod conversions;
/// Parser for macro parameters
mod parsing;

/// Parses a dot-delimited column name into an array of field names. See
/// `delta_kernel::expressions::column_name::column_name` macro for details.
#[proc_macro]
pub fn parse_column_name(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let is_valid = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '.';
    let err = match syn::parse(input) {
        Ok(syn::Lit::Str(name)) => match name.value().chars().find(|c| !is_valid(*c)) {
            Some(bad_char) => Error::new(name.span(), format!("Invalid character: {bad_char:?}")),
            _ => {
                let path = name.value();
                let path = path.split('.').map(proc_macro2::Literal::string);
                return quote_spanned! { name.span()=> [#(#path),*] }.into();
            }
        },
        Ok(lit) => Error::new(lit.span(), "Expected a string literal"),
        Err(err) => err,
    };
    err.into_compile_error().into()
}

#[proc_macro]
pub fn object_conversions(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ObjectDefs);

    let to_object_impls = input.defs.iter().map(to_object);
    let from_object_impls = input.defs.iter().map(from_object);

    // Generate resource impls
    let resource_impls = input.defs.iter().map(resource_impl);

    let to_resource_impls = input.defs.iter().map(to_resource);

    let expanded = quote! {
        #(#to_object_impls)*
        #(#from_object_impls)*
        #(#resource_impls)*
        #(#to_resource_impls)*
    };

    proc_macro::TokenStream::from(expanded)
}
