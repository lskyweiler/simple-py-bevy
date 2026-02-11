extern crate proc_macro;
extern crate quote;
#[cfg(feature = "pyo3")]
use crate::{backend::BEVY_WORLD_PTR_DELETED_ERROR_MSG, py_bevy_meth, py_ref};
#[cfg(feature = "pyo3")]
use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;

#[cfg(feature = "pyo3")]
#[derive(Debug, FromMeta)]
#[darling(derive_syn_parse)]
struct ConfigStructArgs {
    #[darling(default)]
    name: Option<String>,
}

/// Derive a version of this struct that uses the bevy world as a accessor of the structs data
///
/// This needs to be different from the component version since you access components differently than resources
///
#[cfg(feature = "pyo3")]
pub(crate) fn export_bevy_ref_impls(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let struct_name = ast.ident.clone();
    let py_bevy_ref_name = quote::format_ident!("{}BevyRef", ast.ident);

    let py_ref_get_set_fns = py_ref::transform_py_ref_fields(&ast);

    let hash_py_fn_export = py_bevy_meth::export_hash_py_fn(&ast.ident);

    quote! {
        #[pyo3::pyclass(unsendable)]
        pub struct #py_bevy_ref_name {
            world: simple_py_bevy::UnsafeWorldRef,
            alive_ptr: std::sync::Weak<bool>
        }
        impl #py_bevy_ref_name {
            pub fn from_world(world: &mut simple_py_bevy::World) -> Self {
                let world_ref = simple_py_bevy::UnsafeWorldRef::new(world);
                Self::from_world_ref(world_ref)
            }
            pub fn from_world_ref(world_ref: simple_py_bevy::UnsafeWorldRef) -> Self {
                let alive_ptr = world_ref.get_world_alive_ptr();
                Self {
                    world: world_ref,
                    alive_ptr: alive_ptr
                }
            }
            fn get_inner_ref(&self) -> pyo3::prelude::PyResult<simple_py_bevy::Mut<'_, #struct_name>> {
                self.world.get_res_mut::<#struct_name>()
            }
            fn map_to_inner<'a, F, U>(&self, f: F) -> pyo3::PyResult<U>
            where
                F: FnOnce(std::ptr::NonNull<#struct_name>) -> pyo3::PyResult<U>,
            {
                match self.alive_ptr.upgrade() {
                    Some(_) => {
                        let mut inner = self.get_inner_ref()?;
                        let parent_ptr = std::ptr::NonNull::new(&mut (*inner)).unwrap();
                        f(parent_ptr.clone())
                    }
                    None => Err(pyo3::exceptions::PyValueError::new_err(#BEVY_WORLD_PTR_DELETED_ERROR_MSG)),
                }
            }
        }

        #hash_py_fn_export

        #[pyo3::pymethods]
        impl #py_bevy_ref_name {
            #py_ref_get_set_fns
        }

        impl simple_py_bevy::BevyResRefIntoPyAny for #struct_name {
            fn into_py_any_from_world<'py>(
                py: pyo3::prelude::Python<'py>,
                world_ref: simple_py_bevy::UnsafeWorldRef
            ) -> pyo3::prelude::Py<pyo3::prelude::PyAny> {
                let bevy_ref = #py_bevy_ref_name::from_world_ref(world_ref);
                pyo3::prelude::Py::new(py, bevy_ref).unwrap().into_any()
            }
        }
    }
    .into()
}

pub(crate) fn py_bevy_res_struct_impl(_args: TokenStream, ast: syn::ItemStruct) -> TokenStream {
    #[cfg(feature = "pyo3")]
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
            #[derive(simple_py_bevy::Resource, Clone, PyBevyResRef)]
            #[pyo3::pyclass(name = #new_name)]
            #[pyo3_stub_gen::derive::gen_stub_pyclass]
            #ast
        )
        .into()
    }

    #[cfg(not(feature = "pyo3"))]
    {
        quote!(
            #[derive(simple_py_bevy::Resource, Clone, DummyPyO3, DummyPyBevy)]
            #ast
        )
        .into()
    }
}
