use parking_lot::RwLock;
use std::{
    sync::{Arc, atomic::AtomicU32},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use gpu_controller::GpuController;
use log::info;
use properties::gravity::Gravitational;
pub use properties::gravity::Gravity;
pub use rigid_body::{RigidBody, RigidBodyBuilder};
pub use static_collider::StaticCollider;

mod collider;
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

    pub fn resolve_collisions(&self, other: &BosonObject, timestep: f64) {}

    #[inline]
    pub fn modify_body<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&mut BosonBody) -> R,
    {
        callback(&mut self.0.write())
    }

    #[inline]
    pub fn read_body<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&BosonBody) -> R,
    {
        callback(&self.0.read())
    }

    #[inline]
    pub fn update(&self, timestep: f64) {
        self.modify_body(|body| match body {
            BosonBody::RigidBody(rigid_body) => {
                rigid_body.update(timestep);
            }
            _ => {}
        })
    }
}

pub enum BosonBody {
    RigidBody(RigidBody),
    StaticCollider(StaticCollider),
}

#[derive(Default)]
pub struct BosonBuilder {
    gravity: Option<Gravity>,
}

impl BosonBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn gravity(mut self, gravity: Gravity) -> Self {
        self.gravity = Some(gravity);
        self
    }

    pub fn build(self, gpu_controller: Arc<GpuController>) -> Boson {
        Boson::new(self.gravity, gpu_controller)
    }
}

pub struct Boson {
    objects_count: AtomicU32,
    objects: Arc<RwLock<Vec<BosonObject>>>,
    gpu_controller: Arc<GpuController>,
    tickrate: Duration,

    world_gravity: Arc<RwLock<Option<Gravity>>>,

    // Multi Threading
    boson_thread: (Arc<RwLock<bool>>, JoinHandle<()>),
}

unsafe impl Send for Boson {}
unsafe impl Sync for Boson {}

impl Boson {
    pub fn new(gravity: Option<Gravity>, gpu_controller: Arc<GpuController>) -> Self {
        info!("Initializing Boson");
        let objects: Arc<RwLock<Vec<BosonObject>>> = Arc::new(RwLock::new(Vec::new()));
        // let world_gravity = Arc::new(RwLock::new(None));
        // let world_gravity = Arc::new(RwLock::new(Some(Gravity::WorldPoint {
        //     gravitational_acceleration: Vector3::unit_y() * -9.81,
        //     location: Vector3::new(0.0, 6378137.0, 0.0),
        //     mass: 5.972e24,
        // })));
        let world_gravity = Arc::new(RwLock::new(gravity));
        let thread_objects = objects.clone();
        let tickrate = DEFAULT_TICKRATE;
        let tr_clone = tickrate.clone();
        let thread_world_gravity = world_gravity.clone();
        let boson_thread_function = std::thread::spawn(move || {
            info!("Starting Boson Thread");
            let mut last_frame_time = Instant::now();

            loop {
                let now = Instant::now();
                let dt = now.duration_since(last_frame_time).as_secs_f64();
                last_frame_time = now;

                // Apply World Gravity to all objects if available
                {
                    if let Some(world_gravity) = &*thread_world_gravity.read() {
                        for object in thread_objects.read().iter() {
                            object.0.write().apply_gravity(world_gravity, dt);
                        }
                    }
                }

                // Update all objects after forces are applied
                {
                    for object in thread_objects.read().iter() {
                        object.update(dt);
                    }
                }

                std::thread::sleep(tr_clone);
            }
        });

        Self {
            objects_count: AtomicU32::new(0),
            objects,
            gpu_controller,
            world_gravity,
            tickrate,
            boson_thread: (Arc::new(RwLock::new(true)), boson_thread_function),
        }
    }

    pub fn add_object(&mut self, object: &BosonObject) -> u32 {
        let object_id = self
            .objects_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        self.objects.write().push(object.clone());

        object_id
    }
}
