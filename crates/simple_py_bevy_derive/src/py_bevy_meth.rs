extern crate proc_macro;
extern crate quote;
#[cfg(feature = "pyo3")]
use crate::backend;
use proc_macro::TokenStream;
use quote::quote;
#[cfg(feature = "pyo3")]
use syn::ItemImpl;

#[cfg(feature = "pyo3")]
fn wrap_py_method(method: &syn::ImplItemFn) -> syn::ImplItemFn {
    let mut new_method = method.clone();
    let old_sig_name = &method.sig.ident;

    let old_arg_names = backend::get_function_argument_names(&method);

    match &method.sig.output {
        syn::ReturnType::Default => {
            new_method.block = syn::parse_quote!(
                {
                    self.get_inner_ref()?.#old_sig_name(#(#old_arg_names),*);
                    Ok(())
                }
            );
            new_method.sig.output = syn::parse2(quote! { -> pyo3::PyResult<()> }).unwrap();
        }
        syn::ReturnType::Type(_, ty) => {
            let original_r_type = quote! { #ty };
            new_method.block = syn::parse_quote!(
                {
                    Ok(self.get_inner_ref()?.#old_sig_name(#(#old_arg_names),*))
                }
            );
            new_method.sig.output = syn::parse2(quote! { -> pyo3::PyResult<#original_r_type> })
                .expect("Failed to set new return type");
        }
    };
    new_method
}

#[cfg(feature = "pyo3")]
fn impl_ref_methods(input: &mut ItemImpl) -> proc_macro2::TokenStream {
    let mut generated_methods = Vec::new();

    let struct_name = backend::get_struct_name_from_impl(&input);
    let py_bevy_ref_name = quote::format_ident!("{}BevyRef", struct_name);

    for item in &mut input.items {
        match item {
            syn::ImplItem::Fn(method) => {
                let fn_has_new = backend::fn_has_attr_name(&method, "new");
                let fn_has_staticmeth = backend::fn_has_attr_name(&method, "staticmethod");
                let fn_has_classattr = backend::fn_has_attr_name(&method, "classattr");
                if fn_has_new || fn_has_staticmeth || fn_has_classattr {
                    // you cant create a reference from python anyway, so ignore pyo3 constructors and static methods for now (#[new, staticmethod] attributes)
                    continue;
                }
                let new_method = wrap_py_method(&method);
                generated_methods.push(new_method);
            }
            _ => {}
        }
    }

    let functions = quote! {
        #(#generated_methods)*
    };

    proc_macro2::TokenStream::from(quote! {
        #[pyo3::pymethods]
        impl #py_bevy_ref_name {
            #functions
        }
    })
}

#[cfg(feature = "pyo3")]
pub(crate) fn export_hash_py_fn(struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        impl simple_py_bevy::GetTypeHash for #struct_name {
            fn get_type_hash() -> u128 {
                let tid = std::any::TypeId::of::<Self>();
                // SAFETY: TypeId is just a struct with a u64/u128,
                // we use unsafe to turn it into a u64 for const evaluation.
                unsafe { std::mem::transmute::<std::any::TypeId, u128>(tid) }
            }
        }

        #[pyo3::pymethods]
        impl #struct_name {
            #[classattr]
            fn __simple_type_hash__() -> u128 {
                Self::get_type_hash()
            }
        }
    }
    .into()
}

pub(crate) fn py_bevy_methods_impl(_attr: TokenStream, input: TokenStream) -> TokenStream {
    #[allow(unused_mut)]
    let mut ast = syn::parse_macro_input!(input as syn::ItemImpl);

    #[cfg(feature = "pyo3")]
    {
        let expanded = impl_ref_methods(&mut ast);
        quote!(
            #[pyo3::pymethods]
            #[pyo3_stub_gen::derive::gen_stub_pymethods]
            #ast
            #expanded
        )
        .into()
    }

    #[cfg(not(feature = "pyo3"))]
    {
        quote!(
            #ast
        )
        .into()
    }
}
