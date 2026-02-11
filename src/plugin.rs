use crate::{registry, world_ref};
use bevy::prelude::*;

pub struct PyBevyPlugin;
impl Plugin for PyBevyPlugin {
    fn build(&self, app: &mut App) {
        let new_reg = registry::PyObjectRegistry::new();

        // todo: auto register all built-in components using inventory! crate
        app.init_resource::<world_ref::BevyHealthCheckPtr>()
            .insert_resource(new_reg);
    }
}
