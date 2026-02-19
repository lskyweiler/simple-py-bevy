extern crate proc_macro;
extern crate quote;
#[cfg(feature = "minimal-pyo3")]
use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;

#[cfg(feature = "minimal-pyo3")]
#[derive(Debug, FromMeta)]
#[darling(derive_syn_parse)]
struct ConfigStructArgs {
    #[darling(default)]
    name: Option<String>,
}

pub(crate) fn simple_pyclass_impl(_args: TokenStream, ast: syn::ItemStruct) -> TokenStream {
    #[cfg(feature = "minimal-pyo3")]
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
            #[pyo3_stub_gen::derive::gen_stub_pyclass]
            #[pyo3::pyclass(name = #new_name)]
            #ast
        )
        .into()
    }

    #[cfg(not(feature = "minimal-pyo3"))]
    {
        quote!(
            #[derive(DummyPyO3)]
            #ast
        )
        .into()
    }
}
pub(crate) fn simple_enum_impl(_args: TokenStream, ast: syn::ItemEnum) -> TokenStream {
    #[cfg(feature = "minimal-pyo3")]
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
            #[pyo3_stub_gen::derive::gen_stub_pyclass_enum]
            #[pyo3::pyclass(name = #new_name, eq)]  // the only real difference between the enum and pyclass impls
            #ast
        )
        .into()
    }

    #[cfg(not(feature = "minimal-pyo3"))]
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

    #[cfg(feature = "minimal-pyo3")]
    {
        quote!(
            #[pyo3_stub_gen::derive::gen_stub_pymethods]
            #[pyo3::pymethods]
            #ast
        )
        .into()
    }

    #[cfg(not(feature = "minimal-pyo3"))]
    {
        quote!(
            #ast
        )
        .into()
    }
}
