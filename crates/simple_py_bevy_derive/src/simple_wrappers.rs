extern crate proc_macro;
extern crate quote;
#[cfg(feature = "minimal-pyo3")]
use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn export_to_owned_stubs(
    _struct_name: &syn::Ident,
    _py_name: &str,
) -> proc_macro2::TokenStream {
    #[cfg(feature = "gen-to-owned-stubs")]
    {
        quote! {
            pyo3_stub_gen::inventory::submit! {
                pyo3_stub_gen::type_info::PyMethodsInfo {
                    struct_id: std::any::TypeId::of::<#_struct_name>,
                    attrs: &[],
                    getters: &[],
                    setters: &[],
                    methods: &[
                        pyo3_stub_gen::type_info::MethodInfo {
                            name: "to_owned",
                            r#return: || pyo3_stub_gen::TypeInfo {
                                name: #_py_name.to_string(),
                                source_module: None,
                                import: std::collections::HashSet::new(),
                                type_refs: std::collections::HashMap::new()
                            },
                            doc: "Convert this reference to an owned value by cloning it",
                            parameters: &[],
                            is_async: false,
                            r#type: pyo3_stub_gen::type_info::MethodType::Instance,
                            type_ignored: None,
                            is_overload: false,
                            deprecated: None
                        },
                    ],
                    file: "",
                    line: 0,
                    column: 0
                }
            }
        }
        .into()
    }
    #[cfg(not(feature = "gen-to-owned-stubs"))]
    {
        quote! {}.into()
    }
}

#[cfg(feature = "minimal-pyo3")]
#[derive(Debug, FromMeta)]
#[darling(derive_syn_parse)]
struct ConfigStructArgs {
    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    stub_gen_module: Option<String>,
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
        let stub_gen_attr = match &args.stub_gen_module {
            Some(module) => {
                quote! { #[pyo3_stub_gen::derive::gen_stub_pyclass(module = #module)] }
            }
            None => {
                quote! { #[pyo3_stub_gen::derive::gen_stub_pyclass] }
            }
        };

        let to_owned_stubs = export_to_owned_stubs(struct_name, &new_name);

        quote!(
            #stub_gen_attr
            #[pyo3::pyclass(name = #new_name)]
            #ast

            #to_owned_stubs
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
        let stub_gen_attr = match &args.stub_gen_module {
            Some(module) => {
                quote! { #[pyo3_stub_gen::derive::gen_stub_pyclass_enum(module = #module)] }
            }
            None => {
                quote! { #[pyo3_stub_gen::derive::gen_stub_pyclass_enum] }
            }
        };

        quote!(
            #stub_gen_attr
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
