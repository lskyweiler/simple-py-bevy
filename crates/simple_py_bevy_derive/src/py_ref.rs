extern crate proc_macro;
extern crate quote;
use crate::expand_methods;
use crate::backend::BEVY_WORLD_PTR_DELETED_ERROR_MSG;

/// Auto generate a struct with a reference to the original type
/// Also generate pyo3 getters and setters for all members without the skip attribute
pub(crate) fn py_ref_struct_impl(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let struct_name = ast.ident.clone();
    let py_ref_name = quote::format_ident!("{}Ref", ast.ident);

    let py_ref_get_set_fns = expand_methods::gen_get_set_for_fields_mapped_to_inner(&ast);

    quote::quote!(
        #[derive(Clone)]
        #[pyo3::pyclass(unsendable)]
        pub struct #py_ref_name {
            parent_ref: std::ptr::NonNull<#struct_name>,
            alive_ptr: std::sync::Weak<bool>
        }
        impl #py_ref_name {
            fn map_to_inner<'a, F, U>(&self, f: F) -> pyo3::PyResult<U>
            where
                F: FnOnce(std::ptr::NonNull<#struct_name>) -> pyo3::PyResult<U>,
            {
                match self.alive_ptr.upgrade() {
                    Some(_) => {
                        f(self.parent_ref.clone())
                    }
                    None => Err(pyo3::exceptions::PyValueError::new_err(#BEVY_WORLD_PTR_DELETED_ERROR_MSG)),
                }
            }
            pub fn get_inner_ref(&self) -> pyo3::prelude::PyResult<&#struct_name> {
                match self.alive_ptr.upgrade() {
                    Some(_) => {
                        Ok(unsafe { self.parent_ref.clone().as_ref() })
                    }
                    None => Err(pyo3::exceptions::PyValueError::new_err(#BEVY_WORLD_PTR_DELETED_ERROR_MSG)),
                }
            }
            pub fn get_inner_ref_mut(&self) -> pyo3::prelude::PyResult<&mut #struct_name> {
                match self.alive_ptr.upgrade() {
                    Some(_) => {
                        Ok(unsafe { self.parent_ref.clone().as_mut() })
                    }
                    None => Err(pyo3::exceptions::PyValueError::new_err(#BEVY_WORLD_PTR_DELETED_ERROR_MSG)),
                }
            }
        }

        impl simple_py_bevy::FromParent<#struct_name> for #py_ref_name {
            type Output = #py_ref_name;

            fn from_parent(parent: std::ptr::NonNull<#struct_name>, alive_ptr: std::sync::Weak<bool>) -> Self::Output {
                #py_ref_name {
                    parent_ref: parent,
                    alive_ptr: alive_ptr
                }
            }
        }

        #[pyo3::pymethods]
        impl #py_ref_name {
            #py_ref_get_set_fns
        }
    )
    .into()
}

pub(crate) fn py_ref_methods_impl(_attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut ast = syn::parse_macro_input!(input as syn::ItemImpl);

    let expanded = expand_methods::wrap_all_methods_with_get_inner(&mut ast, "Ref".to_string());
    quote::quote!(
        #ast
        #expanded
    )
    .into()
}

