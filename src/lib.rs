#[cfg(feature = "py-bevy")]
mod plugin;
#[cfg(feature = "py-bevy")]
mod registry;
#[cfg(feature = "py-bevy")]
mod world_ref;

// public re-exports
#[cfg(feature = "py-bevy")]
pub use plugin::PyBevyPlugin;
#[cfg(feature = "py-bevy")]
pub use registry::PyObjectRegistry;
#[cfg(feature = "py-bevy")]
pub use world_ref::{BevyHealthCheckPtr, UnsafeWorldRef};

pub use simple_py_bevy_derive::*;

#[cfg(feature = "py-bevy")]
pub use bevy::prelude::*;

pub trait MakePathsAbsolute {
    fn make_paths_absolute(&mut self, _parent_path: &std::path::PathBuf) {}
}

#[cfg(feature = "py-ref")]
mod ref_traits {
    pub trait FromParent<P> {
        type Output;

        fn from_parent(
            parent: std::ptr::NonNull<P>,
            alive_ptr: std::sync::Weak<bool>,
        ) -> Self::Output;
    }
}
#[cfg(feature = "py-ref")]
pub use ref_traits::*;

#[cfg(feature = "py-bevy")]
mod pyo3_traits {
    use super::UnsafeWorldRef;
    use bevy::prelude::Entity;
    use pyo3::prelude::*;

    pub trait UnwrapOrFromYamlEnv<T> {
        fn unwrap_or_from_yaml_env(self) -> Result<T, Box<dyn std::error::Error>>;
    }

    pub trait BevyPyRes {
        fn into_py_any_from_world<'py>(py: Python<'py>, world_ref: UnsafeWorldRef) -> Py<PyAny>;
        fn insert_into_world_from_bound_any(
            res: Bound<'_, PyAny>,
            world_ref: UnsafeWorldRef,
        ) -> PyResult<()>;
    }

    pub trait BevyPyComp {
        fn into_py_any_from_world<'py>(
            py: Python<'py>,
            world_ref: UnsafeWorldRef,
            entity: Entity,
        ) -> Py<PyAny>;

        fn has_component(world_ref: UnsafeWorldRef, entity: Entity) -> PyResult<bool>;

        fn insert_into_world_from_bound_any(
            comp: Bound<'_, PyAny>,
            world_ref: UnsafeWorldRef,
            entity: Entity,
        ) -> PyResult<()>;
    }
    pub trait GetTypeHash {
        fn get_type_hash() -> u128;
    }
}
#[cfg(feature = "py-bevy")]
pub use pyo3_traits::*;

// #[cfg(feature = "testing")]
// mod testing {
//     use super::*;
//     use crate::FromParent;
//
//     #[derive(Clone, PyStructRef)]
//     #[pyclass]
//     #[pyo3_stub_gen::derive::gen_stub_pyclass]
//     pub struct MyInnerComp {
//         a: f32,
//         b: i32,
//     }
//     #[py_ref_methods]
//     impl MyInnerComp {
//         #[new]
//         fn py_new(a: f32, b: i32) -> Self {
//             Self { a, b }
//         }
//         fn foo_bar(&self) -> f32 {
//             self.a + self.b as f32
//         }
//
//         fn res(&self) -> PyResult<f32> {
//             Ok(100.)
//         }
//     }
//
//     #[derive(Clone)]
//     #[py_bevy_component]
//     pub struct MyComp {
//         a: f64,
//         #[py_bevy(get_ref = MyInnerCompRef)]
//         inner: MyInnerComp,
//     }
//     #[py_bevy_methods]
//     impl MyComp {
//         #[new]
//         fn py_new(a: f64, inner: MyInnerComp) -> Self {
//             Self { a, inner }
//         }
//
//         #[allow(unused_variables)]
//         fn foo_bar(&self, a: Vec<i32>, c: MyComp) -> MyComp {
//             self.clone()
//         }
//     }
//
//     #[py_bevy_resource]
//     pub struct MyRes {
//         #[py_bevy(skip)]
//         a: f64,
//     }
//     #[py_bevy_methods]
//     impl MyRes {
//         #[new]
//         fn py_new(a: f64) -> Self {
//             Self { a }
//         }
//
//         #[allow(unused_variables)]
//         fn foo_bar(&self, a: Vec<i32>, c: MyComp) -> MyRes {
//             self.clone()
//         }
//
//         #[getter]
//         fn get_a(&self) -> f64 {
//             self.a
//         }
//         #[setter]
//         fn set_a(&mut self, b: f64) {
//             self.a = b;
//         }
//     }
//
//     /// Simple test harness to allow us to unit test rust-owned views from python
//     #[allow(dead_code)]
//     #[pyclass(unsendable)]
//     pub struct TestPrototypeContext {
//         app: App,
//         e: Entity,
//     }
//
//     fn system(mut query: Query<&mut MyComp>, mut my_res: ResMut<MyRes>) {
//         for mut my_c in &mut query {
//             my_c.a += 1.
//         }
//         my_res.a += 1.;
//     }
//
//     #[pymethods]
//     impl TestPrototypeContext {
//         #[new]
//         fn py_new(my_comp: MyComp, my_res: MyRes) -> Self {
//             let mut app = App::new();
//             app.add_systems(Update, system)
//                 .add_plugins(PyBevyPlugin)
//                 .insert_resource(my_res);
//             let world = app.world_mut();
//             let e_id = world.spawn(my_comp).id().clone();
//
//             Self { app: app, e: e_id }
//         }
//         fn step(&mut self) {
//             self.app.update();
//         }
//
//         fn get_comp_ref(&mut self) -> MyCompBevyRef {
//             let world = self.app.world_mut();
//             MyCompBevyRef::from_world(world, self.e)
//         }
//         fn get_res_ref(&mut self) -> MyResBevyRef {
//             let world = self.app.world_mut();
//             MyResBevyRef::from_world(world)
//         }
//     }
// }
//
// #[cfg(feature = "testing")]
// use pyo3::prelude::*;
//
// #[cfg(feature = "testing")]
// #[pymodule]
// fn simple_py_bevy<'py>(m: &Bound<'_, PyModule>) -> PyResult<()> {
//     m.add_class::<testing::TestPrototypeContext>()?;
//     m.add_class::<testing::MyComp>()?;
//     m.add_class::<testing::MyRes>()?;
//     m.add_class::<testing::MyInnerComp>()?;
//
//     Ok(())
// }
