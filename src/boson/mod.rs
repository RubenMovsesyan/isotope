use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
    time::Instant,
};

use cgmath::{Matrix3, One, Quaternion, Vector3, Zero};
use collider::{Collision, CollisionPoints, sphere_collider::SphereCollider};
use log::*;
use solver::Solver;
use static_collider::StaticCollider;
use wgpu::RenderPass;

use crate::{
    Collider, RigidBody, gpu_utils::GpuController,
    photon::renderer::photon_layouts::PhotonLayoutsManager,
};

pub mod boson_math;
pub mod collider;
mod properties;
pub mod rigid_body;
pub mod solver;
pub mod static_collider;

// Wrapper struct
#[derive(Debug)]
pub struct BosonObject(Arc<RwLock<BosonBody>>);

impl BosonObject {
    pub fn new(object: impl Into<BosonBody>) -> Self {
        Self(Arc::new(RwLock::new(object.into())))
    }

    pub fn modify<F>(&mut self, callback: F)
    where
        F: Fn(&mut BosonBody),
    {
        if let Ok(mut boson_body) = self.0.write() {
            callback(&mut boson_body);
        }
    }

    pub fn access<F>(&self, callback: F)
    where
        F: FnOnce(&BosonBody),
    {
        if let Ok(boson_body) = self.0.read() {
            callback(&boson_body);
        }
    }
}

impl Clone for BosonObject {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Into<Arc<RwLock<dyn Linkable>>> for BosonObject {
    fn into(self) -> Arc<RwLock<dyn Linkable>> {
        self.0.clone()
    }
}

#[derive(Debug)]
pub enum BosonBody {
    RigidBody(RigidBody),
    StaticCollider(StaticCollider),
}

impl BosonBody {
    pub fn new(object: impl Into<BosonBody>) -> Self {
        object.into()
    }

    pub fn pos<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut Vector3<f32>),
    {
        match self {
            BosonBody::RigidBody(rigid_body) => {
                callback(&mut rigid_body.position);
                rigid_body.collider.link_pos(&rigid_body.position);
            }
            BosonBody::StaticCollider(static_collider) => {
                callback(&mut static_collider.position);
                static_collider.collider.link_pos(&static_collider.position);
            }
        }
    }

    pub fn vel<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut Vector3<f32>),
    {
        match self {
            BosonBody::RigidBody(rigid_body) => {
                callback(&mut rigid_body.velocity);
            }
            BosonBody::StaticCollider(_) => {}
        }
    }

    pub fn angular_vel<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut Vector3<f32>),
    {
        match self {
            BosonBody::RigidBody(rigid_body) => {
                callback(&mut rigid_body.angular_velocity);
            }
            BosonBody::StaticCollider(_) => {}
        }
    }

    #[inline]
    pub fn get_mass(&self) -> f32 {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.mass,
            BosonBody::StaticCollider(_) => 0.0,
        }
    }

    #[inline]
    pub fn get_inv_mass(&self) -> f32 {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.inv_mass,
            BosonBody::StaticCollider(_) => 0.0,
        }
    }

    #[inline]
    pub fn get_inv_inertia(&self) -> Vector3<f32> {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.inverse_inertia,
            BosonBody::StaticCollider(_) => Vector3::zero(),
        }
    }

    #[inline]
    pub fn get_inertia(&self) -> Matrix3<f32> {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.inertia_tensor,
            BosonBody::StaticCollider(_) => Matrix3::one(),
        }
    }

    #[inline]
    pub fn get_pos(&self) -> Vector3<f32> {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.position,
            BosonBody::StaticCollider(static_collider) => static_collider.position,
        }
    }

    #[inline]
    pub fn get_vel(&self) -> Vector3<f32> {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.velocity,
            BosonBody::StaticCollider(_) => Vector3::zero(),
        }
    }

    #[inline]
    pub fn get_angular_vel(&self) -> Vector3<f32> {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.angular_velocity,
            BosonBody::StaticCollider(_) => Vector3::zero(),
        }
    }

    #[inline]
    pub fn get_restitution(&self) -> f32 {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.restitution,
            BosonBody::StaticCollider(_) => 1.0,
        }
    }

    #[inline]
    pub fn get_static_friction(&self) -> f32 {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.static_friction,
            BosonBody::StaticCollider(static_collider) => static_collider.static_friction,
        }
    }

    #[inline]
    pub fn get_dynamic_friction(&self) -> f32 {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.dynamic_friction,
            BosonBody::StaticCollider(static_collider) => static_collider.dynamic_friction,
        }
    }

    pub(crate) fn build_collider(
        &mut self,
        gpu_controller: Arc<GpuController>,
        photon_layouts_manager: &PhotonLayoutsManager,
    ) {
        match self {
            BosonBody::RigidBody(rigid_body) => match rigid_body.collider_builder {
                crate::ColliderBuilder::Sphere => {
                    rigid_body.collider = Collider::Sphere(SphereCollider::new(
                        rigid_body.position,
                        1.0,
                        gpu_controller,
                        photon_layouts_manager,
                    ));
                }
                crate::ColliderBuilder::Plane => {}
                crate::ColliderBuilder::Cube => {}
            },
            BosonBody::StaticCollider(_) => {}
        }
    }

    pub fn apply_force(&mut self, force: Vector3<f32>, delta_t: &Instant) {
        match self {
            BosonBody::RigidBody(rigid_body) => {
                rigid_body.apply_force(force, delta_t);
            }
            BosonBody::StaticCollider(_) => {}
        }
    }

    pub fn apply_torque(&mut self, torque: Vector3<f32>, delta_t: &Instant) {
        match self {
            BosonBody::RigidBody(rigid_body) => {
                rigid_body.apply_torque(torque, delta_t);
            }
            BosonBody::StaticCollider(_) => {}
        }
    }

    pub fn update(&mut self, delta_t: &Instant) {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.update(delta_t),
            BosonBody::StaticCollider(_) => {}
        }
    }

    pub fn debug_render(&self, render_pass: &mut RenderPass) {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.debug_render(render_pass),
            BosonBody::StaticCollider(_) => {}
        }
    }

    pub fn test_collision(&self, other: &BosonBody) -> Option<CollisionPoints> {
        match self {
            BosonBody::RigidBody(rigid_body) => match other {
                BosonBody::RigidBody(other_rigid) => {
                    rigid_body.test_collision(&other_rigid.collider)
                }
                BosonBody::StaticCollider(static_collider) => {
                    rigid_body.test_collision(&static_collider.collider)
                }
            },
            BosonBody::StaticCollider(static_collider) => match other {
                BosonBody::RigidBody(rigid_body) => static_collider
                    .collider
                    .test_collision(&rigid_body.collider),
                BosonBody::StaticCollider(_) => None,
            },
        }
    }
}

impl Linkable for BosonBody {
    fn get_position(&self) -> Vector3<f32> {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.position,
            BosonBody::StaticCollider(static_collider) => static_collider.position,
        }
    }

    fn get_rotation(&self) -> Quaternion<f32> {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.orientation,
            BosonBody::StaticCollider(static_collider) => static_collider.orientation,
        }
    }
}

// Trait that ensures objects are linkable in isotope
pub trait Linkable: Debug + Send + Sync {
    fn get_position(&self) -> Vector3<f32>;
    fn get_rotation(&self) -> Quaternion<f32>;
}

#[derive(Debug)]
pub struct Boson {
    objects: Vec<BosonObject>,
    solvers: Vec<Arc<dyn Solver>>,
    gravity: Vector3<f32>,
    gpu_controller: Arc<GpuController>,
    photon_layouts_manager: Arc<PhotonLayoutsManager>,
}

impl Boson {
    pub(crate) fn new(
        gpu_controller: Arc<GpuController>,
        photon_layouts_manager: Arc<PhotonLayoutsManager>,
    ) -> Self {
        Self {
            objects: Vec::new(),
            solvers: Vec::new(),
            gravity: Vector3 {
                x: 0.0,
                y: -9.81, // default for now
                z: 0.0,
            },
            gpu_controller,
            photon_layouts_manager,
        }
    }

    pub fn add_dynamic_object(&mut self, mut object: BosonObject) {
        object.modify(|object| {
            object.build_collider(self.gpu_controller.clone(), &self.photon_layouts_manager);
        });

        self.objects.push(object);
        info!("Added Object to Boson");
    }

    pub fn add_solver(&mut self, solver: impl Solver) {
        self.solvers.push(Arc::new(solver));
    }

    fn resolve_collisions(&mut self, delta_t: &Instant) {
        // Get all the collisions
        let mut collisions: Vec<Collision> = Vec::new();

        for (a_index, object_a) in self.objects.iter().enumerate() {
            let mut check_collision = true;

            // check if the collision needs to be checked
            object_a.access(|object_a| match object_a {
                BosonBody::StaticCollider(_) => check_collision = false,
                _ => {}
            });

            if !check_collision {
                continue;
            }

            for (b_index, object_b) in self.objects.iter().enumerate() {
                if a_index == b_index {
                    continue;
                }

                let mut collision = None;

                object_a.access(|object_a| {
                    object_b.access(|object_b| {
                        collision = object_a.test_collision(object_b);
                    });
                });

                if let Some(points) = collision {
                    collisions.push(Collision {
                        object_a: object_a.clone(),
                        object_b: object_b.clone(),
                        points,
                    })
                }
            }
        }

        // Run all the solvers on the collisions
        for solver in self.solvers.iter() {
            solver.solve(&mut collisions, delta_t);
        }
    }

    pub fn debug_render(&self, render_pass: &mut RenderPass) {
        for object in self.objects.iter() {
            object.access(|object| {
                object.debug_render(render_pass);
            });
        }
    }

    pub(crate) fn step(&mut self, delta_t: &Instant) {
        self.resolve_collisions(delta_t);

        for object in self.objects.iter_mut() {
            object.modify(|boson_body| {
                boson_body.update(delta_t);
            });
        }
    }
}
