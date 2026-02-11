/*
    Generic utilities for macros
*/

use syn;

pub const BEVY_WORLD_PTR_DELETED_ERROR_MSG: &'static str = "Underlying world has been deleted";

/// Get the name of a struct from an impl block
///
/// # Examples
///
/// ```
/// impl Foo {
///     ...
/// }
/// > Foo
///
/// ```
pub fn get_struct_name_from_impl(input: &syn::ItemImpl) -> syn::Ident {
    let self_type = &input.self_ty;
    let struct_name = if let syn::Type::Path(type_path) = &**self_type {
        // Get the last segment of the path, which should be the struct name
        &type_path
            .path
            .segments
            .last()
            .expect("Expected a path segment")
            .ident
    } else {
        // Handle cases where the type is not a simple path (e.g., references, tuples, etc.)
        // This is a simplified example; a real macro might need more robust handling.
        panic!("Macro only supports simple struct types for impl blocks.");
    };
    return struct_name.clone();
}

/// Get the names of the variables to a function omitting self
///
/// # Examples
/// ```
/// fn foo(a: f64, b: Vec<i32>) -> ...
/// > [a, b]
/// ```
pub fn get_function_argument_names(input: &syn::ImplItemFn) -> Vec<&syn::Pat> {
    input
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                Some(&*pat_type.pat)
            // } else if let syn::FnArg::Receiver(receiver) = arg {
            //     // Handle `&self` or `self`
            //     receiver.pat.as_ref().map(|p| &**p)
            } else {
                None
            }
        })
        .collect()
}

/// Determine if this function has a given attribute tag
/// ex: #[new], #[staticmethod]
pub fn fn_has_attr_name(item: &syn::ImplItemFn, target_attr_name: &str) -> bool {
    item.attrs
        .iter()
        .any(|attr: &syn::Attribute| attr.path().is_ident(target_attr_name))
}
