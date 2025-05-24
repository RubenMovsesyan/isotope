use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
    time::Instant,
};

use cgmath::Vector3;
use log::info;

pub mod rigid_body;

// Generic trait for all physics objects
pub trait DynamicObject: Debug + Send + Sync + 'static {
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
    objects: Vec<Arc<RwLock<dyn DynamicObject>>>,
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

    pub fn add_dynamic_object(&mut self, object: Arc<RwLock<dyn DynamicObject>>) {
        self.objects.push(object);
        info!("Added Object to Boson");
    }

    pub fn step(&mut self, delta_t: &Instant) {
        for object in self.objects.iter_mut() {
            if let Ok(mut object) = object.write() {
                let mass = object.get_mass();
                object.apply_force(mass * self.gravity, delta_t);
            }
        }
    }
}
