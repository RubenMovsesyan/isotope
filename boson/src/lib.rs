use parking_lot::RwLock;
use std::{
    sync::{Arc, atomic::AtomicU32},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use cgmath::Vector3;
use gpu_controller::GpuController;
use log::info;
pub use point_mass::PointMass;
use properties::gravity::{Gravitational, Gravity};
pub use rigid_body::RigidBody;
pub use static_collider::StaticCollider;

mod point_mass;
mod properties;
mod rigid_body;
mod static_collider;

const DEFAULT_TICKRATE: Duration = Duration::from_micros(50);

pub struct BosonObject(Arc<RwLock<BosonBody>>);

unsafe impl Send for BosonObject {}
unsafe impl Sync for BosonObject {}

impl Clone for BosonObject {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl BosonObject {
    pub fn new(boson_body: BosonBody) -> Self {
        Self(Arc::new(RwLock::new(boson_body)))
    }

    pub fn resolve_collisions(&self, other: &BosonObject, timestep: f32) {}

    pub fn modify_body<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&mut BosonBody) -> R,
    {
        callback(&mut self.0.write())
    }

    pub fn read_body<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&BosonBody) -> R,
    {
        callback(&self.0.read())
    }
}

pub enum BosonBody {
    PointMass(PointMass),
    RigidBody(RigidBody),
    StaticCollider(StaticCollider),
}

pub struct Boson {
    objects_count: AtomicU32,
    objects: Arc<RwLock<Vec<BosonObject>>>,
    gpu_controller: Arc<GpuController>,
    tickrate: Duration,

    // Multi Threading
    boson_thread: (Arc<RwLock<bool>>, JoinHandle<()>),
}

unsafe impl Send for Boson {}
unsafe impl Sync for Boson {}

impl Boson {
    pub fn new(gpu_controller: Arc<GpuController>) -> Self {
        info!("Initializing Boson");
        let objects: Arc<RwLock<Vec<BosonObject>>> = Arc::new(RwLock::new(Vec::new()));
        let thread_objects = objects.clone();
        let tickrate = DEFAULT_TICKRATE;
        let tr_clone = tickrate.clone();
        let boson_thread_function = std::thread::spawn(move || {
            info!("Starting Boson Thread");
            let mut last_frame_time = Instant::now();

            // let gravity = Gravity::World(Vector3::unit_y() * -9.81);
            // let gravity = Gravity::Point(Vector3::new(20.0, 6378137.0, 400.0), 5.972e24);
            let gravity = Gravity::WorldPoint(
                Vector3::unit_y() * -9.81,
                Vector3::new(0.0, 6378137.0, 0.0),
                5.972e24,
            );

            loop {
                let now = Instant::now();
                let dt = now.duration_since(last_frame_time).as_secs_f64();
                last_frame_time = now;

                let objects = thread_objects.read();
                for object in objects.iter() {
                    let mut object = object.0.write();

                    match *object {
                        BosonBody::PointMass(ref mut point_mass) => {
                            point_mass.apply_gravity(&gravity, dt);
                        }
                        _ => {}
                    }
                }

                std::thread::sleep(tr_clone);
            }
        });

        Self {
            objects_count: AtomicU32::new(0),
            objects,
            gpu_controller,
            tickrate,
            boson_thread: (Arc::new(RwLock::new(true)), boson_thread_function),
        }
    }

    pub fn add_object(&mut self, object: &BosonObject) -> u32 {
        let object_id = self
            .objects_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        // if let Ok(mut objects) = self.objects.write() {
        //     objects.push(object.clone());
        // }
        self.objects.write().push(object.clone());

        object_id
    }
}
