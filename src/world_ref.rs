use bevy::{ecs::component::Mutable, prelude::*};
use pyo3::{exceptions::PyValueError, prelude::*};
use std::{
    ptr::NonNull,
    sync::{Arc, Mutex, Weak},
};

#[derive(Resource, Clone)]
pub struct BevyHealthCheckPtr(Arc<bool>);
impl BevyHealthCheckPtr {
    pub fn downgrade(&self) -> Weak<bool> {
        Arc::downgrade(&self.0)
    }
}
impl FromWorld for BevyHealthCheckPtr {
    fn from_world(_: &mut World) -> Self {
        Self::new()
    }
}
impl BevyHealthCheckPtr {
    pub fn new() -> Self {
        Self(Arc::new(true))
    }
}

const BEVY_WORLD_PTR_DELETED_ERROR_MSG: &'static str = "Underlying world has been deleted";

#[derive(Clone)]
pub struct UnsafeWorldRef {
    // todo: is there even a reason to make this arc mutex? this is unsendable anyway
    world_ptr: Arc<Mutex<Option<NonNull<World>>>>,
    world_alive_ptr: Weak<bool>,
}
impl UnsafeWorldRef {
    pub fn new(world: &mut World) -> Self {
        let bevy_health = world.resource::<BevyHealthCheckPtr>().clone();
        Self {
            world_ptr: Arc::new(Mutex::new(NonNull::new(world))),
            world_alive_ptr: bevy_health.downgrade(),
        }
    }
    fn get_mut<'w>(&self) -> &'w mut bevy::prelude::World {
        unsafe { self.world_ptr.lock().unwrap().unwrap().as_mut() }
    }
    pub fn get_world_alive_ptr(&self) -> Weak<bool> {
        self.world_alive_ptr.clone()
    }

    pub fn map_to_world<'w, F, U>(&self, f: F) -> PyResult<U>
    where
        F: FnOnce(&'w mut World) -> PyResult<U>,
    {
        match self.world_alive_ptr.upgrade() {
            Some(_) => {
                let world = self.get_mut();
                f(world)
            }
            None => Err(PyValueError::new_err(BEVY_WORLD_PTR_DELETED_ERROR_MSG)),
        }
    }

    pub fn get_comp_mut<'w, C: Component<Mutability = Mutable>>(
        &self,
        entity: &Entity,
    ) -> PyResult<Mut<'w, C>> {
        self.map_to_world(|world| match world.get_mut::<C>(*entity) {
            Some(comp) => Ok(comp),
            None => Err(PyValueError::new_err(format!(
                "Entity {entity} doesn't have component C"
            ))),
        })
    }
    pub fn get_comp<'w, C: Component>(&self, entity: &Entity) -> PyResult<&C> {
        self.map_to_world(|world| match world.get::<C>(*entity) {
            Some(comp) => Ok(comp),
            None => Err(PyValueError::new_err(format!(
                "Entity {entity} doesn't have component C"
            ))),
        })
    }
    pub fn entity_has_comp<'w, C: Component<Mutability = Mutable>>(
        &self,
        entity: &Entity,
    ) -> PyResult<bool> {
        self.map_to_world(|world| Ok(world.get_mut::<C>(*entity).is_some()))
    }
    pub fn get_res_mut<R: Resource>(&self) -> PyResult<Mut<'_, R>> {
        self.map_to_world(|world| match world.get_resource_mut::<R>() {
            Some(comp) => Ok(comp),
            None => Err(PyValueError::new_err(format!(
                "World does not contain resource R"
            ))),
        })
    }
    pub fn get_res<R: Resource>(&self) -> PyResult<&R> {
        self.map_to_world(|world| match world.get_resource::<R>() {
            Some(comp) => Ok(comp),
            None => Err(PyValueError::new_err(format!(
                "World does not contain resource R"
            ))),
        })
    }
}
