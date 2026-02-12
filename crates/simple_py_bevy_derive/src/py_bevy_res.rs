extern crate proc_macro;
extern crate quote;
use crate::backend::BEVY_WORLD_PTR_DELETED_ERROR_MSG;
use crate::expand_methods;
use quote::quote;

/// Derive a version of this struct that uses the bevy world as a accessor of the structs data
///
/// This needs to be different from the component version since you access components differently than resources
///
pub(crate) fn export_bevy_ref_impls(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let struct_name = ast.ident.clone();
    let py_bevy_ref_name = quote::format_ident!("{}BevyRef", ast.ident);

    let py_ref_get_set_fns = expand_methods::gen_get_set_for_fields_mapped_to_inner(&ast);

    let hash_py_fn_export = expand_methods::export_hash_py_fn(&ast.ident);

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
