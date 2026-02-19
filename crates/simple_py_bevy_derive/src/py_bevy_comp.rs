extern crate proc_macro;
extern crate quote;
use crate::backend::BEVY_WORLD_PTR_DELETED_ERROR_MSG;
use crate::expand_methods;
use quote::quote;

pub(crate) fn derive_py_bevy_comp_struct_impl(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let struct_name = ast.ident.clone();
    let py_bevy_ref_name = quote::format_ident!("{}BevyRef", ast.ident);

    let py_ref_get_set_fns = expand_methods::gen_get_set_for_fields_mapped_to_inner(&ast);

    // generate a hash function on the original struct to make lookup easier
    let hash_py_fn_export = expand_methods::export_hash_py_fn(&ast.ident);

    quote!(
        #[pyo3::pyclass(unsendable)]
        pub struct #py_bevy_ref_name {
            world: simple_py_bevy::UnsafeWorldRef,
            entity: simple_py_bevy::Entity,
            alive_ptr: std::sync::Weak<bool>
        }
        impl #py_bevy_ref_name {
            pub fn from_world(world: &mut simple_py_bevy::World,  entity: simple_py_bevy::Entity) -> Self {
                let world_ref = simple_py_bevy::UnsafeWorldRef::new(world);
                Self::from_world_ref(world_ref, entity)
            }
            pub fn from_world_ref(world_ref: simple_py_bevy::UnsafeWorldRef,  entity: simple_py_bevy::Entity) -> Self {
                let alive_ptr = world_ref.get_world_alive_ptr();
                Self {
                    world: world_ref,
                    entity: entity,
                    alive_ptr: alive_ptr
                }
            }
            pub fn create_py_bevy_ref<'py>(
                py: pyo3::prelude::Python<'py>,
                world: simple_py_bevy::UnsafeWorldRef,
                entity: simple_py_bevy::Entity
            ) -> pyo3::prelude::Py<pyo3::prelude::PyAny> {
                let alive_ptr = world.get_world_alive_ptr();
                let r_val = Self {
                    world: world,
                    entity: entity,
                    alive_ptr: alive_ptr
                };
                pyo3::prelude::Py::new(py, r_val).unwrap().into_any()
            }
            pub fn get_inner_ref(&self) -> pyo3::prelude::PyResult<&#struct_name> {
                self.world.get_comp::<#struct_name>(&self.entity)
            }
            pub fn get_inner_ref_mut(&self) -> pyo3::prelude::PyResult<simple_py_bevy::Mut<'_, #struct_name>> {
                self.world.get_comp_mut::<#struct_name>(&self.entity)
            }

            fn map_to_inner<'a, F, U>(&self, f: F) -> pyo3::PyResult<U>
            where
                F: FnOnce(std::ptr::NonNull<#struct_name>) -> pyo3::PyResult<U>,
            {
                match self.alive_ptr.upgrade() {
                    Some(_) => {
                        let mut inner = self.get_inner_ref_mut()?;
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

        impl simple_py_bevy::BevyPyComp for #struct_name {
            fn into_py_any_from_world<'py>(
                py: pyo3::prelude::Python<'py>,
                world_ref: simple_py_bevy::UnsafeWorldRef,
                entity: simple_py_bevy::Entity
            ) -> pyo3::prelude::Py<pyo3::prelude::PyAny> {
                let bevy_ref = #py_bevy_ref_name::from_world_ref(world_ref, entity);
                pyo3::prelude::Py::new(py, bevy_ref).unwrap().into_any()
            }
            fn has_component(
                world_ref: simple_py_bevy::UnsafeWorldRef,
                entity: simple_py_bevy::Entity
            ) -> pyo3::prelude::PyResult<bool> {
                world_ref.entity_has_comp::<#struct_name>(&entity)
            }

            fn insert_into_world_from_bound_any(
                comp: pyo3::prelude::Bound<'_, pyo3::prelude::PyAny>,
                world_ref: simple_py_bevy::UnsafeWorldRef,
                entity: simple_py_bevy::Entity,
            ) -> pyo3::prelude::PyResult<()> {
                use pyo3::types::PyAnyMethods; // ensures that extract is in scope

                world_ref.map_to_world(|world| {
                    let extracted: Self = comp.extract()?;
                    let mut e = world.entity_mut(entity);
                    e.insert(extracted);
                    Ok(())
                })?;
                Ok(())
            }
        }
    )
    .into()
}
