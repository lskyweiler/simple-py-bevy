extern crate proc_macro;
extern crate quote;
#[cfg(feature = "py-bevy")]
use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;

#[cfg(feature = "py-bevy")]
#[derive(Debug, FromMeta)]
#[darling(derive_syn_parse)]
struct ConfigStructArgs {
    #[darling(default)]
    name: Option<String>,
}

pub(crate) fn simple_pyclass_impl(_args: TokenStream, ast: syn::ItemStruct) -> TokenStream {
    #[cfg(feature = "py-bevy")]
    {
        let struct_name = &ast.ident;
        let args: ConfigStructArgs = match syn::parse(_args) {
            Ok(v) => v,
            Err(e) => {
                return e.to_compile_error().into();
            }
        };
        let new_name = match &args.name {
            Some(n) => format!(r#"{}"#, n),
            None => format!(r#"{}"#, struct_name),
        };
        quote!(
            #[pyo3::pyclass(name = #new_name)]
            #[pyo3_stub_gen::derive::gen_stub_pyclass]
            #ast
        )
        .into()
    }

    #[cfg(not(feature = "py-bevy"))]
    {
        quote!(
            #[derive(DummyPyO3)]
            #ast
        )
        .into()
    }
}
pub(crate) fn simple_enum_impl(_args: TokenStream, ast: syn::ItemEnum) -> TokenStream {
    #[cfg(feature = "py-bevy")]
    {
        let struct_name = &ast.ident;
        let args: ConfigStructArgs = match syn::parse(_args) {
            Ok(v) => v,
            Err(e) => {
                return e.to_compile_error().into();
            }
        };
        let new_name = match &args.name {
            Some(n) => format!(r#"{}"#, n),
            None => format!(r#"{}"#, struct_name),
        };
        quote!(
            #[pyo3::pyclass(name = #new_name, eq)]  // the only real difference between the enum and pyclass impls
            #[pyo3_stub_gen::derive::gen_stub_pyclass_enum]
            #ast
        )
        .into()
    }

    #[cfg(not(feature = "py-bevy"))]
    {
        quote!(
            #[derive(DummyPyO3)]
            #ast
        )
        .into()
    }
}

pub(crate) fn simple_pymethods_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::ItemImpl);

    #[cfg(feature = "py-bevy")]
    {
        quote!(
            #[pyo3::pymethods]
            #[pyo3_stub_gen::derive::gen_stub_pymethods]
            #ast
        )
        .into()
    }

    #[cfg(not(feature = "py-bevy"))]
    {
        quote!(
            #ast
        )
        .into()
    }
}
