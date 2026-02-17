/*
Macros to enable auto derivation of classes that expose bevy owned data to python

These are not meant to be robust production macros, so there may be sharp edges or poor error handling
*/
extern crate proc_macro;
extern crate quote;

use proc_macro::TokenStream;

#[cfg(feature = "py-ref")]
mod backend;
#[cfg(feature = "py-ref")]
mod py_ref;

#[cfg(feature = "py-ref")]
mod expand_methods;
#[cfg(feature = "py-bevy")]
mod py_bevy_comp;
mod py_bevy_config;
#[cfg(feature = "py-bevy")]
mod py_bevy_meth;
#[cfg(feature = "py-bevy")]
mod py_bevy_res;
mod simple_wrappers;

/// Auto generate a BevyRef and a Ref version of this struct and add traits to load this object from yaml
// todo: need to put yaml impl into a derive macro and remove this in favor of explicit derives
#[proc_macro_attribute]
pub fn py_bevy_config_res(attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = syn::parse(input).unwrap();
    match item {
        syn::Item::Struct(struct_) => py_bevy_config::py_bevy_config_res_struct_impl(attr, struct_),
        unsupported => {
            syn::Error::new_spanned(unsupported, "#[py_bevy_config] only supports structs")
                .into_compile_error()
                .into()
        }
    }
}

/// Simple wrapper macro to make creating a pyclass easier with auto stubs
/// Handles pyo3 feature
#[proc_macro_attribute]
pub fn simple_pyclass(attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = syn::parse(input).unwrap();
    match item {
        syn::Item::Struct(s) => simple_wrappers::simple_pyclass_impl(attr, s),
        syn::Item::Enum(e) => simple_wrappers::simple_enum_impl(attr, e),
        unsupported => syn::Error::new_spanned(
            unsupported,
            "#[simple_pyclass] only supports structs or enums",
        )
        .into_compile_error()
        .into(),
    }
}
/// Simple wrapper macro to make creating a pyclass's methods easier
/// Handles pyo3 feature
#[proc_macro_attribute]
pub fn simple_pymethods(attr: TokenStream, input: TokenStream) -> TokenStream {
    simple_wrappers::simple_pymethods_impl(attr, input)
}

/// Auto generate methods to expose this struct's methods to python
#[proc_macro_attribute]
pub fn py_bevy_methods(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    #[cfg(feature = "py-bevy")]
    {
        py_bevy_meth::py_bevy_methods_impl(_attr, _input)
    }
    #[cfg(not(feature = "py-bevy"))]
    {
        dummy_pyo3::passthrough(_input)
    }
}
#[proc_macro_attribute]
pub fn py_ref_methods(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    #[cfg(feature = "py-ref")]
    {
        py_ref::py_ref_methods_impl(_attr, _input)
    }
    #[cfg(not(feature = "py-ref"))]
    {
        dummy_pyo3::passthrough(_input)
    }
}

/// Generate a BevyCompRef version of this struct
#[proc_macro_derive(PyBevyCompRef, attributes(py_bevy))]
pub fn derive_py_bevy_comp_structs(_input: TokenStream) -> TokenStream {
    #[cfg(feature = "py-bevy")]
    {
        let ast = syn::parse_macro_input!(_input as syn::DeriveInput);

        let py_bevy_expand =  py_bevy_comp::derive_py_bevy_comp_struct_impl(&ast);

        quote::quote! {
            #py_bevy_expand

        }
        .into()
    }
    #[cfg(not(feature = "py-bevy"))]
    {
        dummy_pyo3::erase_input()
    }
}

/// Generate a BevyResRef version of this struct
#[proc_macro_derive(PyBevyResRef, attributes(py_bevy))]
pub fn derive_py_bevy_res_structs(_input: TokenStream) -> TokenStream {
    #[cfg(feature = "py-bevy")]
    {
        let ast = syn::parse_macro_input!(_input as syn::DeriveInput);

        let py_bevy_expand = py_bevy_res::export_bevy_ref_impls(&ast);

        quote::quote! {
            #py_bevy_expand

        }
        .into()
    }
    #[cfg(not(feature = "py-bevy"))]
    {
        dummy_pyo3::erase_input()
    }
}

/// Generate a Ref version of this struct
#[proc_macro_derive(PyStructRef, attributes(py_bevy))]
pub fn derive_py_ref_struct(_input: TokenStream) -> TokenStream {
    #[cfg(feature = "py-ref")]
    {
        let ast = syn::parse_macro_input!(_input as syn::DeriveInput);

        let py_bevy_expand = py_ref::py_ref_struct_impl(&ast);

        quote::quote! {
            #py_bevy_expand
        }
        .into()
    }
    #[cfg(not(feature = "py-ref"))]
    {
        dummy_pyo3::erase_input()
    }
}

/// Needed to mock pyo3 macro attributes in case we're not using the pyo3 feature
#[cfg(not(feature = "py-bevy"))]
#[proc_macro_derive(DummyPyO3, attributes(pyo3))]
pub fn derive_dummy_pyo3(_input: TokenStream) -> TokenStream {
    dummy_pyo3::erase_input()
}
#[cfg(not(feature = "py-bevy"))]
#[proc_macro_attribute]
pub fn new(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    dummy_pyo3::erase_input()
}
#[cfg(not(feature = "py-bevy"))]
#[proc_macro_attribute]
pub fn getter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(item as syn::Item);
    dummy_pyo3::strip_attributes(&ast)
}
#[cfg(not(feature = "py-bevy"))]
#[proc_macro_attribute]
pub fn setter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(item as syn::Item);
    dummy_pyo3::strip_attributes(&ast)
}
#[cfg(not(feature = "py-bevy"))]
#[proc_macro_attribute]
pub fn staticmethod(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(item as syn::Item);
    dummy_pyo3::strip_attributes(&ast)
}
#[cfg(not(feature = "py-bevy"))]
#[proc_macro_attribute]
pub fn classattr(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    dummy_pyo3::erase_input()
}

#[cfg(not(feature = "py-bevy"))]
mod dummy_pyo3 {
    use super::*;

    pub fn strip_attributes(ast: &syn::Item) -> TokenStream {
        quote::quote! {
            #ast
        }
        .into()
    }
    pub fn erase_input() -> TokenStream {
        quote::quote! {}.into()
    }

    pub fn passthrough(input: TokenStream) -> TokenStream {
        input
    }
}
