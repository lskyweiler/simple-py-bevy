use crate::{world_ref, BevyPyComp, BevyPyRes, GetTypeHash};
use bevy::prelude::*;
use pyo3::{exceptions::PyValueError, prelude::*};
use std::collections::HashMap;

// All components and resources deriving #[py_bevy_component] and #[py_bevy_resource] will implement BevyPyComp and BevyPyRes
type BevyRefFromWorldFn = fn(Python<'_>, world_ref::UnsafeWorldRef) -> Py<PyAny>;
type BevyResInsertFromBoundAny = fn(Bound<'_, PyAny>, world_ref::UnsafeWorldRef) -> PyResult<()>;
type BevyCompFromWorldFn = fn(Python<'_>, world_ref::UnsafeWorldRef, Entity) -> Py<PyAny>;
type RemoveCompAndReturnOwnedFromWorldFn =
    fn(Python<'_>, &mut world_ref::UnsafeWorldRef, Entity) -> PyResult<Option<Py<PyAny>>>;
type BevyEntHashCompFn = fn(world_ref::UnsafeWorldRef, Entity) -> PyResult<bool>;
type BevyCompInsertFromBoundAny =
    fn(Bound<'_, PyAny>, world_ref::UnsafeWorldRef, Entity) -> PyResult<()>;

/// Registry mapping py_classes to internal bevy components and resources
#[derive(Resource)]
pub struct PyObjectRegistry {
    // This is complicated since we need to statically compile how to construct and extract these types,
    //     while being able to get what type it is from a python object
    //
    // * There is probably a better way to do this
    //
    create_bevy_ref_res_fns: HashMap<u128, BevyRefFromWorldFn>,
    built_in_insert_res: HashMap<u128, BevyResInsertFromBoundAny>,

    // it would be better to store Box<dyn BevyPyComp>, but it's a static method and not Sized
    create_bevy_ref_comp_fns: HashMap<u128, BevyCompFromWorldFn>,
    remove_comp_and_return_fns: HashMap<u128, RemoveCompAndReturnOwnedFromWorldFn>,
    built_in_has_comps: HashMap<u128, BevyEntHashCompFn>,
    build_in_insert_comps: HashMap<u128, BevyCompInsertFromBoundAny>,
}
impl PyObjectRegistry {
    pub fn new() -> Self {
        Self {
            create_bevy_ref_res_fns: HashMap::new(),
            built_in_insert_res: HashMap::new(),
            create_bevy_ref_comp_fns: HashMap::new(),
            remove_comp_and_return_fns: HashMap::new(),
            built_in_has_comps: HashMap::new(),
            build_in_insert_comps: HashMap::new(),
        }
    }
    pub fn register_res<T: GetTypeHash + BevyPyRes>(&mut self) {
        let hash = T::get_type_hash();
        self.create_bevy_ref_res_fns
            .insert(hash, T::into_bevy_ref_py_any_from_world);
        self.built_in_insert_res
            .insert(hash, T::insert_into_world_from_bound_any);
    }
    pub fn register_comp<T: GetTypeHash + BevyPyComp>(&mut self) {
        let hash = T::get_type_hash();
        self.create_bevy_ref_comp_fns
            .insert(hash, T::into_bevy_ref_py_any_from_world);
        self.remove_comp_and_return_fns
            .insert(hash, T::remove_from_entity_and_return_owned_py_any);
        self.built_in_has_comps.insert(hash, T::has_component);
        self.build_in_insert_comps
            .insert(hash, T::insert_into_world_from_bound_any);
    }
    pub fn create_bevy_res_ref<'py>(
        &self,
        py: Python<'py>,
        type_hash: u128,
        world: world_ref::UnsafeWorldRef,
    ) -> Option<Py<PyAny>> {
        let from_world = self.create_bevy_ref_res_fns.get(&type_hash)?;
        let res = from_world(py, world);
        Some(res)
    }
    pub fn insert_res_from_py_any_bound(
        &mut self,
        comp: Bound<'_, PyAny>,
        type_hash: u128,
        world: world_ref::UnsafeWorldRef,
    ) -> PyResult<()> {
        if let Some(insert) = self.built_in_insert_res.get(&type_hash) {
            return insert(comp, world);
        } else {
            return Err(PyValueError::new_err("Component does not exist internally"));
        }
    }

    pub fn comp_exists(&self, type_hash: u128) -> bool {
        self.create_bevy_ref_comp_fns.contains_key(&type_hash)
    }

    pub fn entity_has_comp(
        &self,
        type_hash: u128,
        world: world_ref::UnsafeWorldRef,
        entity: Entity,
    ) -> PyResult<bool> {
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
        let from_world = self.create_bevy_ref_comp_fns.get(&type_hash)?;
        let res = from_world(py, world, entity);
        Some(res)
    }
    pub fn remove_comp<'py>(
        &self,
        py: Python<'py>,
        type_hash: u128,
        mut world: world_ref::UnsafeWorldRef,
        entity: Entity,
    ) -> Option<Option<Py<PyAny>>> {
        let rm_comp = self.remove_comp_and_return_fns.get(&type_hash)?;
        rm_comp(py, &mut world, entity).ok()
    }

    pub fn insert_comp_from_py_any_bound(
        &mut self,
        comp: Bound<'_, PyAny>,
        type_hash: u128,
        world: world_ref::UnsafeWorldRef,
        entity: Entity,
    ) -> PyResult<()> {
        if let Some(insert) = self.build_in_insert_comps.get(&type_hash) {
            return insert(comp, world, entity);
        } else {
            return Err(PyValueError::new_err("Component does not exist internally"));
        }
    }
}
