extern crate proc_macro;
extern crate quote;
use proc_macro::TokenStream;
use quote::quote;

use crate::expand_methods;

pub(crate) fn py_bevy_methods_impl(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = syn::parse_macro_input!(input as syn::ItemImpl);

    let expanded = expand_methods::wrap_all_methods_with_get_inner(&mut ast, "BevyRef".to_string());
    quote!(
        #ast
        #expanded
    )
    .into()
}
