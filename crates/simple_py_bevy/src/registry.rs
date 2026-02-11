use crate::{world_ref, BevyCompRefIntoPyAny, BevyResRefIntoPyAny, GetTypeHash};
use bevy::prelude::*;
use pyo3::prelude::*;
use std::collections::HashMap;

// All components and resources deriving #[py_bevy_component] and #[py_bevy_resource] will implement BevyCompRefIntoPyAny and BevyResRefIntoPyAny
pub type BevyRefFromWorldFn = fn(Python<'_>, world_ref::UnsafeWorldRef) -> Py<PyAny>;
pub type BevyCompFromWorldFn = fn(Python<'_>, world_ref::UnsafeWorldRef, Entity) -> Py<PyAny>;
pub type BevyEntHashCompFn = fn(world_ref::UnsafeWorldRef, Entity) -> PyResult<bool>;

/// Registry mapping py_classes to internal bevy references
///
/// This is complicated since we need to statically compile how to construct and extract these types,
///     while being able to get what type it is from a python object
///
/// I'm sure there is a better way to do this
///
#[derive(Resource)]
pub struct PyObjectRegistry {
    built_in_resources: HashMap<u128, BevyRefFromWorldFn>,

    // it would be nice to store Box<dyn BevyCompRefIntoPyAny>, but it's a static method and not Sized
    built_in_components: HashMap<u128, BevyCompFromWorldFn>,
    built_in_has_comps: HashMap<u128, BevyEntHashCompFn>,
}
impl PyObjectRegistry {
    pub fn new() -> Self {
        Self {
            built_in_resources: HashMap::new(),
            built_in_components: HashMap::new(),
            built_in_has_comps: HashMap::new(),
        }
    }
    pub fn register_res<T: GetTypeHash + BevyResRefIntoPyAny>(&mut self) {
        let hash = T::get_type_hash();
        self.built_in_resources
            .insert(hash, T::into_py_any_from_world);
    }
    pub fn register_comp<T: GetTypeHash + BevyCompRefIntoPyAny>(&mut self) {
        let hash = T::get_type_hash();
        self.built_in_components
            .insert(hash, T::into_py_any_from_world);
        self.built_in_has_comps.insert(hash, T::has_component);
    }
    pub fn create_bevy_res_ref<'py>(
        &self,
        py: Python<'py>,
        type_hash: u128,
        world: world_ref::UnsafeWorldRef,
    ) -> Option<Py<PyAny>> {
        let from_world = self.built_in_resources.get(&type_hash)?;
        let res = from_world(py, world);
        Some(res)
    }

    pub fn entity_has_comp(
        &self,
        type_hash: u128,
        world: world_ref::UnsafeWorldRef,
        entity: Entity,
    ) -> Result<bool, PyErr> {
        let has_comp = self.built_in_has_comps.get(&type_hash).unwrap();
        has_comp(world, entity)
    }
    pub fn create_bevy_comp_ref<'py>(
        &self,
        py: Python<'py>,
        type_hash: u128,
        world: world_ref::UnsafeWorldRef,
        entity: Entity,
    ) -> Option<Py<PyAny>> {
        let from_world = self.built_in_components.get(&type_hash)?;
        let res = from_world(py, world, entity);
        Some(res)
    }
}
