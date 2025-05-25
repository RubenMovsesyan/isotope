use std::{
    fmt::Debug,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Instant,
};

use cgmath::Vector3;
use log::info;

pub mod rigid_body;

#[derive(Debug)]
pub struct BosonObject(Arc<RwLock<dyn DynamicObject>>);

impl Clone for BosonObject {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl BosonObject {
    pub fn new(object: impl DynamicObject) -> Self {
        Self(Arc::new(RwLock::new(object)))
    }

    pub fn as_linkable(&self) -> Arc<RwLock<dyn Linkable>> {
        self.0.clone()
    }

    unsafe fn inner(&self) -> RwLockReadGuard<dyn DynamicObject> {
        unsafe { self.0.read().unwrap_unchecked() }
    }

    unsafe fn inner_mut(&self) -> RwLockWriteGuard<dyn DynamicObject> {
        unsafe { self.0.write().unwrap_unchecked() }
    }
}

// Generic trait for all physics objects
pub trait DynamicObject: Linkable + 'static {
    // Objects default behaviour doesn't do anything with force
    #[allow(unused_variables)]
    fn apply_force(&mut self, force: Vector3<f32>, delta_t: &Instant) {}

    // Objects start with default mass of 0
    fn get_mass(&self) -> f32 {
        0.0
    }
}

// Trait that ensures objects are linkable in isotope
pub trait Linkable: Debug + Send + Sync {
    fn get_position(&self) -> Vector3<f32>;
}

#[derive(Debug)]
pub struct Boson {
    objects: Vec<BosonObject>,
    gravity: Vector3<f32>,
}

impl Boson {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            gravity: Vector3 {
                x: 0.0,
                y: -9.81, // default for now
                z: 0.0,
            },
        }
    }

    pub fn add_dynamic_object(&mut self, object: BosonObject) {
        self.objects.push(object);
        info!("Added Object to Boson");
    }

    pub fn step(&mut self, delta_t: &Instant) {
        for object in self.objects.iter_mut() {
            unsafe {
                let mass = object.inner().get_mass();
                object.inner_mut().apply_force(mass * self.gravity, delta_t);
            }
        }
    }
}
