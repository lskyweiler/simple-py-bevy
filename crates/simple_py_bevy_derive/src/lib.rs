/*
Macros to enable auto derivation of classes that expose bevy owned data to python

These are not meant to be robust production macros, so there may be sharp edges or poor error handling
*/
extern crate proc_macro;
extern crate quote;

use proc_macro::TokenStream;

#[cfg(feature = "pyo3")]
mod backend;
#[cfg(feature = "pyo3")]
mod py_ref;

mod py_bevy_comp;
mod py_bevy_config;
mod py_bevy_meth;
mod py_bevy_res;
mod simple_wrappers;

#[proc_macro_attribute]
pub fn py_bevy_config_res(attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = syn::parse(input).unwrap();
    match item {
        syn::Item::Struct(struct_) => py_bevy_config::py_bevy_config_res_struct_impl(attr, struct_),
        unsupported => {
            syn::Error::new_spanned(unsupported, "#[py_bevy_config] only supports structs")
                .into_compile_error()
                .into()
        }
    }
}

/// Auto generate a struct to expose this struct to python as a reference to bevy's world
#[proc_macro_attribute]
pub fn py_bevy_component(attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = syn::parse(input).unwrap();
    match item {
        syn::Item::Struct(struct_) => py_bevy_comp::py_bevy_comp_impl(attr, struct_),
        unsupported => {
            syn::Error::new_spanned(unsupported, "#[py_bevy_component] only supports structs")
                .into_compile_error()
                .into()
        }
    }
}

/// Simple wrapper macro to make creating a pyclass easier with auto stubs
/// Handles pyo3 feature
#[proc_macro_attribute]
pub fn simple_pyclass(attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = syn::parse(input).unwrap();
    match item {
        syn::Item::Struct(s) => simple_wrappers::simple_pyclass_impl(attr, s),
        syn::Item::Enum(e) => simple_wrappers::simple_enum_impl(attr, e),
        unsupported => {
            syn::Error::new_spanned(unsupported, "#[simple_pyclass] only supports structs or enums")
                .into_compile_error()
                .into()
        }
    }
}
/// Simple wrapper macro to make creating a pyclass's methods easier
/// Handles pyo3 feature
#[proc_macro_attribute]
pub fn simple_pymethods(attr: TokenStream, input: TokenStream) -> TokenStream {
    simple_wrappers::simple_pymethods_impl(attr, input)
}

/// Auto generate a struct to expose this struct to python as a reference to bevy's world
#[proc_macro_attribute]
pub fn py_bevy_resource(attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = syn::parse(input).unwrap();
    match item {
        syn::Item::Struct(struct_) => py_bevy_res::py_bevy_res_struct_impl(attr, struct_),
        unsupported => {
            syn::Error::new_spanned(unsupported, "#[py_bevy_resource] only supports structs")
                .into_compile_error()
                .into()
        }
    }
}

/// Auto generate methods to expose this struct's methods to python
///
/// Example output:
/// ```
/// #[py_bevy_component]
/// pub struct MyComp {
///     a: f64,
///     vec: map3d::DVec3,
/// }
/// #[proc_macro_attribute]
/// impl MyComp {
///     #[new]
///     fn py_new(a: f64, vec: map3d::DVec3) -> Self {
///         Self { a, vec }
///     }
///
///     #[getter]
///     fn get_a(&self) -> f64 {
///         self.a
///     }
///     #[setter]
///     fn set_a(&mut self, a: f64) {
///         self.a = a;
///     }
///     #[getter]
///     fn get_vec(&self) -> map3d::DVec3 {
///         self.vec
///     }
/// }
///
/// #[pyclass(unsendable)]
/// struct MyCompBevyRef {
///     world: Arc<Mutex<Option<NonNull<World>>>>,
///     entity: Entity,
///     world_alive_ptr: Weak<bool>,
/// }
/// impl MyCompBevyRef {
///     fn from_world(world: &mut World, entity: Entity) -> Self {
///         let bevy_health = world.resource::<BevyHealthCheckPtr>().clone();
///         Self {
///             world: Arc::new(Mutex::new(NonNull::new(world))),
///             entity: entity,
///             world_alive_ptr: bevy_health.downgrade(),
///         }
///     }
///     fn get_world_mut(&self) -> &mut World {
///         unsafe { self.world.lock().unwrap().unwrap().as_mut() }
///     }
///     fn get_comp_ref_mut(&self) -> PyResult<Mut<'_, MyComp>> {
///         match self.world_alive_ptr.upgrade() {
///             Some(_) => {
///                 let world = self.get_world_mut();
///                 let comp = world.get_mut::<MyComp>(self.entity).unwrap();
///                 Ok(comp)
///             }
///             None => Err(PyValueError::new_err("Underlying world has been deleted")),
///         }
///     }
/// }
/// #[pymethods]
/// impl MyCompBevyRef {
///     #[getter]
///     fn get_vec(&self) -> PyResult<map3d::DVec3> {
///         Ok(self.get_comp_ref_mut()?.get_vec())
///     }
///     #[getter]
///     fn get_a(&self) -> PyResult<f64> {
///         Ok(self.get_comp_ref_mut()?.a)
///     }
///     #[setter]
///     fn set_a(&mut self, a: f64) -> PyResult<()> {
///         self.get_comp_ref_mut()?.a = a;
///         Ok(())
///     }
/// }
/// impl Into<MyComp> for MyCompBevyRef {
///     fn into(self) -> MyComp {
///         let comp_ref = self
///             .get_comp_ref_mut()
///             .expect("Underlying world has been deleted");
///         (*comp_ref).clone()
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn py_bevy_methods(attr: TokenStream, input: TokenStream) -> TokenStream {
    py_bevy_meth::py_bevy_methods_impl(attr, input)
}

#[cfg(feature = "pyo3")]
#[proc_macro_derive(PyBevyCompRef, attributes(py_bevy))]
pub fn derive_py_bevy_comp_structs(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let py_ref_expand = py_ref::py_ref_struct_impl(&ast);
    let py_bevy_expand = py_bevy_comp::derive_py_bevy_comp_struct_impl(&ast);

    quote::quote! {
        #py_bevy_expand

        #py_ref_expand

    }
    .into()
}
#[cfg(not(feature = "pyo3"))]
#[proc_macro_derive(DummyPyBevy, attributes(py_bevy))]
pub fn derive_py_bevy_comp_structs(_input: TokenStream) -> TokenStream {
    dummy_pyo3::erase_input()
}

#[cfg(feature = "pyo3")]
#[proc_macro_derive(PyBevyResRef, attributes(py_bevy))]
pub fn derive_py_bevy_res_structs(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let py_ref_expand = py_ref::py_ref_struct_impl(&ast);
    let py_bevy_expand = py_bevy_res::export_bevy_ref_impls(&ast);

    quote::quote! {
        #py_bevy_expand

        #py_ref_expand

    }
    .into()
}

/// Needed to mock pyo3 macro attributes in case we're not using the pyo3 feature
#[cfg(not(feature = "pyo3"))]
#[proc_macro_derive(DummyPyO3, attributes(pyo3))]
pub fn derive_dummy_pyo3(_input: TokenStream) -> TokenStream {
    dummy_pyo3::erase_input()
}
#[cfg(not(feature = "pyo3"))]
#[proc_macro_attribute]
pub fn new(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    dummy_pyo3::erase_input()
}
#[cfg(not(feature = "pyo3"))]
#[proc_macro_attribute]
pub fn getter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(item as syn::Item);
    dummy_pyo3::strip_attributes(&ast)
}
#[cfg(not(feature = "pyo3"))]
#[proc_macro_attribute]
pub fn setter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(item as syn::Item);
    dummy_pyo3::strip_attributes(&ast)
}
#[cfg(not(feature = "pyo3"))]
#[proc_macro_attribute]
pub fn staticmethod(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(item as syn::Item);
    dummy_pyo3::strip_attributes(&ast)
}
#[cfg(not(feature = "pyo3"))]
#[proc_macro_attribute]
pub fn classattr(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    dummy_pyo3::erase_input()
}

#[cfg(not(feature = "pyo3"))]
mod dummy_pyo3 {
    use super::*;

    pub fn strip_attributes(ast: &syn::Item) -> TokenStream {
        quote::quote! {
            #ast
        }
        .into()
    }
    pub fn erase_input() -> TokenStream {
        quote::quote! {}.into()
    }
}
