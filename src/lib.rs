#[cfg(feature = "pyo3")]
mod plugin;
#[cfg(feature = "pyo3")]
mod registry;
#[cfg(feature = "pyo3")]
mod world_ref;

// public re-exports
#[cfg(feature = "pyo3")]
pub use plugin::PyBevyPlugin;
#[cfg(feature = "pyo3")]
pub use registry::PyObjectRegistry;
#[cfg(feature = "pyo3")]
pub use world_ref::{BevyHealthCheckPtr, UnsafeWorldRef};

pub use simple_py_bevy_derive::*;

#[cfg(feature = "testing")]
pub mod testing;

pub trait MakePathsAbsolute {
    fn make_paths_absolute(&mut self, _parent_path: &std::path::PathBuf) {}
}

#[cfg(feature = "pyo3")]
mod pyo3_traits {
    use pyo3::prelude::*;
    use super::UnsafeWorldRef;
    use bevy::prelude::Entity;

    pub trait UnwrapOrFromYamlEnv<T> {
        fn unwrap_or_from_yaml_env(self) -> Result<T, Box<dyn std::error::Error>>;
    }

    pub trait FromParent<P> {
        type Output;

        fn from_parent(
            parent: std::ptr::NonNull<P>,
            alive_ptr: std::sync::Weak<bool>,
        ) -> Self::Output;
    }

    pub trait BevyResRefIntoPyAny {
        fn into_py_any_from_world<'py>(py: Python<'py>, world_ref: UnsafeWorldRef) -> Py<PyAny>;
    }

    pub trait BevyCompRefIntoPyAny {
        fn into_py_any_from_world<'py>(
            py: Python<'py>,
            world_ref: UnsafeWorldRef,
            entity: Entity,
        ) -> Py<PyAny>;

        fn has_component(world_ref: UnsafeWorldRef, entity: Entity) -> PyResult<bool>;
    }
    pub trait GetTypeHash {
        fn get_type_hash() -> u128;
    }
}
#[cfg(feature = "pyo3")]
pub use pyo3_traits::*;
