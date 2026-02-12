extern crate proc_macro;
extern crate quote;
use darling::FromField;
use quote::{format_ident, quote};
use crate::expand_methods;

const BEVY_WORLD_PTR_DELETED_ERROR_MSG: &'static str = "Underlying world has been deleted";

#[derive(Debug, FromField)]
#[darling(attributes(py_bevy))]
struct PyRefFieldAttrs {
    // Specify how to transform the data into a refernce
    #[darling(default)]
    get_ref: Option<syn::TypePath>,
    #[darling(default)]
    skip: bool,
    #[darling(default)]
    get_only: bool,
}

fn transform_getter(attrs: &PyRefFieldAttrs, field: &syn::Field) -> proc_macro2::TokenStream {
    if attrs.skip {
        return quote! {}.into();
    }

    let field_name = field.ident.as_ref().unwrap();
    let getter_name = format_ident!("get_{}", field_name);

    let inner_name: syn::ExprField = syn::parse_quote! {
        parent.#field_name
    };

    let mut ret_val = field.ty.clone();

    if let Some(transform_ref_class) = &attrs.get_ref {
        ret_val = syn::Type::Path(transform_ref_class.clone());
        quote! {
            #[getter]
            fn #getter_name(&mut self) -> pyo3::PyResult<#ret_val> {
                self.map_to_inner(|mut inner| {
                    unsafe {
                        let mut parent = inner.as_mut();
                        let parent_ptr = std::ptr::NonNull::new(&mut #inner_name).unwrap();
                        Ok(#ret_val::from_parent(parent_ptr, self.alive_ptr.clone()))
                    }
                })
            }
        }
        .into()
    } else {
        quote! {
            #[getter]
            fn #getter_name(&mut self) -> pyo3::PyResult<#ret_val> {
                self.map_to_inner(|mut inner| {
                    unsafe {
                        let mut parent = inner.as_mut();
                        Ok(#inner_name.clone())
                    }
                })
            }
        }
        .into()
    }
}
fn transform_setter(attrs: &PyRefFieldAttrs, field: &syn::Field) -> proc_macro2::TokenStream {
    if attrs.skip || attrs.get_only {
        return quote! {}.into();
    }

    let field_name = field.ident.as_ref().unwrap();
    let setter_name = format_ident!("set_{}", field_name);

    let inner_name: syn::ExprField = syn::parse_quote! {
        parent.#field_name
    };

    let field_type = field.ty.clone();

    quote! {
        #[setter]
        fn #setter_name(&mut self, val: #field_type) -> pyo3::PyResult<()> {
            self.map_to_inner(|mut inner| {
                unsafe {
                    let mut parent = inner.as_mut();
                    #inner_name = val;
                    Ok(())
                }
            })
        }
    }
    .into()
}

/// Auto generate pyo3 getters and setters for all fields in the struct
pub(crate) fn transform_py_ref_fields(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let mut transformed_fns = Vec::new();

    if let syn::Data::Struct(data) = &ast.data {
        for field in &data.fields {
            let attrs =
                PyRefFieldAttrs::from_field(field).expect("Failed to parse field attributes");

            let getter = transform_getter(&attrs, &field);
            let setter = transform_setter(&attrs, &field);

            transformed_fns.push(getter);
            transformed_fns.push(setter);
        }
    }

    quote! {
        #(#transformed_fns)*
    }
    .into()
}

/// Auto generate a struct with a reference to the original type
/// Also generate pyo3 getters and setters for all members without the skip attribute
pub(crate) fn py_ref_struct_impl(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let struct_name = ast.ident.clone();
    let py_ref_name = quote::format_ident!("{}Ref", ast.ident);

    let py_ref_get_set_fns = transform_py_ref_fields(&ast);

    quote!(
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
            fn get_inner_ref(&self) -> pyo3::prelude::PyResult<&mut #struct_name> {
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
    quote!(
        #ast
        #expanded
    )
    .into()
}

