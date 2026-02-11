use bevy::prelude::*;
use pyglam;
use pyo3::{pyclass, pymethods};
use simple::prelude::*;

#[derive(Clone)]
#[py_bevy_component]
pub struct MyInnerComp {
    a: f32,
    b: i32,
}
#[py_bevy_methods]
impl MyInnerComp {
    #[new]
    fn py_new(a: f32, b: i32) -> Self {
        Self { a, b }
    }
    fn foo_bar(&self) -> f32 {
        self.a + self.b as f32
    }
}

#[derive(Clone)]
#[py_bevy_component]
pub struct MyComp {
    a: f64,
    #[py_bevy(get_ref = MyInnerCompRef)]
    inner: MyInnerComp,
    // this currently returns a clone of vec
    vec: pyglam::DVec3,
}
#[py_bevy_methods]
impl MyComp {
    #[new]
    fn py_new(a: f64, inner: MyInnerComp, vec: pyglam::DVec3) -> Self {
        Self { a, inner, vec }
    }

    #[allow(unused_variables)]
    fn foo_bar(&self, a: Vec<i32>, b: pyglam::DVec3, c: MyComp) -> MyComp {
        self.clone()
    }
}

#[py_bevy_resource]
pub struct MyRes {
    #[py_bevy(skip)]
    a: f64,
    #[py_bevy(skip)]
    vec: pyglam::DVec3,
}
#[py_bevy_methods]
impl MyRes {
    #[new]
    fn py_new(a: f64, vec: pyglam::DVec3) -> Self {
        Self { a, vec }
    }

    #[allow(unused_variables)]
    fn foo_bar(&self, a: Vec<i32>, b: pyglam::DVec3, c: MyComp) -> MyRes {
        self.clone()
    }

    #[getter]
    fn get_a(&self) -> f64 {
        self.a
    }
    #[setter]
    fn set_a(&mut self, b: f64) {
        self.a = b;
    }
    #[getter]
    fn get_vec(&self) -> pyglam::DVec3 {
        self.vec
    }
}

pub mod testing {
    use super::*;
    use simple::prelude::PyBevyPlugin;

    /// Simple test harness to allow us to unit test rust-owned views from python
    #[allow(dead_code)]
    #[pyclass(unsendable)]
    pub struct TestPrototypeContext {
        app: App,
        e: Entity,
    }

    fn system(mut query: Query<&mut MyComp>, mut my_res: ResMut<MyRes>) {
        for mut my_c in &mut query {
            my_c.a += 1.
        }
        my_res.a += 1.;
    }

    #[pymethods]
    impl TestPrototypeContext {
        #[new]
        fn py_new(my_comp: MyComp, my_res: MyRes) -> Self {
            let mut app = App::new();
            app.add_systems(Update, system)
                .add_plugins(PyBevyPlugin)
                .insert_resource(my_res);
            let world = app.world_mut();
            let e_id = world.spawn(my_comp).id().clone();

            Self { app: app, e: e_id }
        }
        fn step(&mut self) {
            self.app.update();
        }

        fn get_comp_ref(&mut self) -> MyCompBevyRef {
            let world = self.app.world_mut();
            MyCompBevyRef::from_world(world, self.e)
        }
        fn get_res_ref(&mut self) -> MyResBevyRef {
            let world = self.app.world_mut();
            MyResBevyRef::from_world(world)
        }
    }
}
