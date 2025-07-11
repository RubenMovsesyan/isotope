use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
    time::Instant,
};

use anyhow::{Result, anyhow};
use cgmath::{Matrix3, One, Quaternion, Vector3, Zero};
use collider::{
    Collision, CollisionPoints, cube_collider::CubeCollider, sphere_collider::SphereCollider,
};
use debug_renderer::BosonDebugRenderer;
use log::*;
use particle_system::ParticleSysytem;
use solver::Solver;
use static_collider::StaticCollider;
use wgpu::{CommandEncoder, RenderPass};

use crate::{
    Collider, Instancer, RigidBody, element::model::ModelInstance, gpu_utils::GpuController,
};

pub mod boson_math;
pub mod collider;
mod debug_renderer;
pub mod particle_system;
mod properties;
pub mod rigid_body;
pub mod solver;
pub mod static_collider;

#[derive(Debug)]
pub(crate) enum BosonDebugger {
    None,
    Inactive(BosonDebugRenderer),
    Active(BosonDebugRenderer),
}

impl Default for BosonDebugger {
    fn default() -> Self {
        Self::None
    }
}

#[allow(dead_code)]
impl BosonDebugger {
    pub(crate) fn set(self, debug_renderer: BosonDebugRenderer) -> Self {
        Self::Inactive(debug_renderer)
    }

    pub(crate) fn set_active(self, debug_renderer: BosonDebugRenderer) -> Self {
        Self::Active(debug_renderer)
    }

    pub(crate) fn activate(self) -> Self {
        match self {
            BosonDebugger::Inactive(debug_renderer) => Self::Active(debug_renderer),
            BosonDebugger::Active(_) => self,
            BosonDebugger::None => {
                warn!("Boson Debug Renderer not set, cannot activate");
                self
            }
        }
    }

    pub(crate) fn deactivate(self) -> Self {
        match self {
            BosonDebugger::Inactive(_) => self,
            BosonDebugger::Active(debug_renderer) => Self::Inactive(debug_renderer),
            BosonDebugger::None => {
                warn!("Boson Debug Renderer not set, cannot deactivate");
                self
            }
        }
    }
}

// Wrapper struct
#[derive(Debug)]
pub struct BosonObject(Arc<RwLock<BosonBody>>);

unsafe impl Send for BosonObject {}
unsafe impl Sync for BosonObject {}

impl BosonObject {
    pub fn new(object: impl Into<BosonBody>) -> Self {
        Self(Arc::new(RwLock::new(object.into())))
    }

    pub fn modify<F, R>(&mut self, callback: F) -> Result<R>
    where
        F: FnOnce(&mut BosonBody) -> R,
    {
        match self.0.write() {
            Ok(mut boson_body) => Ok(callback(&mut boson_body)),
            Err(err) => Err(anyhow!("RwLock Poisoned: {}", err)),
        }
    }

    pub fn access<F, R>(&self, callback: F) -> Result<R>
    where
        F: FnOnce(&BosonBody) -> R,
    {
        match self.0.read() {
            Ok(boson_body) => Ok(callback(&boson_body)),
            Err(err) => Err(anyhow!("RwLock Poisoned: {}", err)),
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
    ParticleSystem(ParticleSysytem),
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
            BosonBody::ParticleSystem(particle_system) => {
                callback(&mut particle_system.position);
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
            BosonBody::ParticleSystem(particle_system) => {
                callback(&mut particle_system.velocity);
            }
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
            BosonBody::StaticCollider(_) | BosonBody::ParticleSystem(_) => {}
        }
    }

    #[inline]
    pub fn get_scale_factor(&self) -> f32 {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.scale_factor,
            BosonBody::StaticCollider(_) | BosonBody::ParticleSystem(_) => 1.0,
        }
    }

    #[inline]
    pub fn get_mass(&self) -> f32 {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.mass,
            BosonBody::StaticCollider(_) | BosonBody::ParticleSystem(_) => 0.0,
        }
    }

    #[inline]
    pub fn get_inv_mass(&self) -> f32 {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.inv_mass,
            BosonBody::StaticCollider(_) | BosonBody::ParticleSystem(_) => 0.0,
        }
    }

    #[inline]
    pub fn get_inv_inertia(&self) -> Vector3<f32> {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.inverse_inertia,
            BosonBody::StaticCollider(_) | BosonBody::ParticleSystem(_) => Vector3::zero(),
        }
    }

    #[inline]
    pub fn get_inertia(&self) -> Matrix3<f32> {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.inertia_tensor,
            BosonBody::StaticCollider(_) | BosonBody::ParticleSystem(_) => Matrix3::one(),
        }
    }

    #[inline]
    pub fn get_pos(&self) -> Vector3<f32> {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.position,
            BosonBody::StaticCollider(static_collider) => static_collider.position,
            BosonBody::ParticleSystem(particle_system) => particle_system.position,
        }
    }

    #[inline]
    pub fn get_vel(&self) -> Vector3<f32> {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.velocity,
            BosonBody::StaticCollider(_) => Vector3::zero(),
            BosonBody::ParticleSystem(particle_system) => particle_system.velocity,
        }
    }

    #[inline]
    pub fn get_angular_vel(&self) -> Vector3<f32> {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.angular_velocity,
            BosonBody::StaticCollider(_) | BosonBody::ParticleSystem(_) => Vector3::zero(),
        }
    }

    #[inline]
    pub fn get_restitution(&self) -> f32 {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.restitution,
            BosonBody::StaticCollider(_) | BosonBody::ParticleSystem(_) => 1.0,
        }
    }

    #[inline]
    pub fn get_static_friction(&self) -> f32 {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.static_friction,
            BosonBody::StaticCollider(static_collider) => static_collider.static_friction,
            BosonBody::ParticleSystem(_) => 0.0,
        }
    }

    #[inline]
    pub fn get_dynamic_friction(&self) -> f32 {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.dynamic_friction,
            BosonBody::StaticCollider(static_collider) => static_collider.dynamic_friction,
            BosonBody::ParticleSystem(_) => 0.0,
        }
    }

    // Also build the debug renderers here
    pub(crate) fn build_collider(&mut self, scale_factor: f32, gpu_controller: Arc<GpuController>) {
        match self {
            BosonBody::RigidBody(rigid_body) => match rigid_body.collider_builder {
                crate::ColliderBuilder::Sphere => {
                    rigid_body.collider = Collider::Sphere(SphereCollider::new(
                        rigid_body.position,
                        scale_factor,
                        gpu_controller,
                    ));
                }
                crate::ColliderBuilder::Plane => {}
                crate::ColliderBuilder::Cube => {
                    rigid_body.collider = Collider::Cube(CubeCollider::new(
                        rigid_body.position,
                        scale_factor * 2.0,
                        rigid_body.orientation,
                        gpu_controller,
                    ));
                }
            },
            BosonBody::StaticCollider(_) => {}
            BosonBody::ParticleSystem(_) => {}
        }
    }

    #[inline]
    pub fn activate_debugger(&mut self, gpu_controller: Arc<GpuController>) {
        match self {
            BosonBody::RigidBody(rigid_body) => {
                rigid_body.debugger(|debugger| match debugger {
                    BosonDebugger::None => {
                        *debugger = std::mem::take(debugger)
                            .set_active(BosonDebugRenderer::new(gpu_controller));
                    }
                    _ => {
                        *debugger = std::mem::take(debugger).activate();
                    }
                });
            }
            _ => {}
        }
    }

    #[inline]
    pub(crate) fn deactivate_debugger(&mut self) {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.debugger(|debugger| {
                *debugger = std::mem::take(debugger).deactivate();
            }),
            _ => {}
        }
    }

    pub fn apply_force(&mut self, force: Vector3<f32>, delta_t: &Instant) {
        match self {
            BosonBody::RigidBody(rigid_body) => {
                rigid_body.apply_force(force, delta_t);
            }
            BosonBody::ParticleSystem(particle_system) => {
                particle_system.apply_force(force, delta_t);
            }
            BosonBody::StaticCollider(_) => {}
        }
    }

    pub fn apply_torque(&mut self, torque: Vector3<f32>, delta_t: &Instant) {
        match self {
            BosonBody::RigidBody(rigid_body) => {
                rigid_body.apply_torque(torque, delta_t);
            }
            BosonBody::ParticleSystem(_) => {}
            BosonBody::StaticCollider(_) => {}
        }
    }

    pub fn update(&mut self, delta_t: &Instant) {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.update(delta_t),
            BosonBody::ParticleSystem(particle_system) => particle_system.update(delta_t),
            BosonBody::StaticCollider(_) => {}
        }
    }

    pub fn debug_render(&self, render_pass: &mut RenderPass) {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.debug_render(render_pass),
            BosonBody::ParticleSystem(_) => {}
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
                BosonBody::ParticleSystem(_) => None,
            },
            BosonBody::StaticCollider(static_collider) => match other {
                BosonBody::RigidBody(rigid_body) => static_collider
                    .collider
                    .test_collision(&rigid_body.collider),
                BosonBody::StaticCollider(_) => None,
                BosonBody::ParticleSystem(_) => None,
            },
            BosonBody::ParticleSystem(_) => None,
        }
    }

    pub fn reset(&mut self) {
        match self {
            BosonBody::ParticleSystem(particle_system) => particle_system.reset_system(),
            _ => {}
        }
    }

    pub fn update_instances(&mut self, encoder: &mut CommandEncoder) {
        match self {
            BosonBody::ParticleSystem(particle_system) => particle_system.update_instances(encoder),
            _ => {}
        }
    }
}

impl Linkable for BosonBody {
    fn get_position(&self) -> Vector3<f32> {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.position,
            BosonBody::StaticCollider(static_collider) => static_collider.position,
            BosonBody::ParticleSystem(particle_system) => particle_system.position,
        }
    }

    fn get_orientation(&self) -> Quaternion<f32> {
        match self {
            BosonBody::RigidBody(rigid_body) => rigid_body.orientation,
            BosonBody::StaticCollider(static_collider) => static_collider.orientation,
            BosonBody::ParticleSystem(particle_system) => particle_system.orientation,
        }
    }

    fn get_instancer(&self) -> Option<Arc<Instancer<ModelInstance>>> {
        match self {
            BosonBody::ParticleSystem(particle_system) => particle_system.get_instancer(),
            _ => None,
        }
    }
}

// Trait that ensures objects are linkable in isotope
pub trait Linkable: Debug + Send + Sync {
    fn get_position(&self) -> Vector3<f32>;
    fn get_orientation(&self) -> Quaternion<f32>;
    fn get_instancer(&self) -> Option<Arc<Instancer<ModelInstance>>>;
}

#[derive(Debug)]
pub struct Boson {
    objects: Vec<BosonObject>,
    solvers: Vec<Arc<dyn Solver>>,
    gpu_controller: Arc<GpuController>,
    debugging: bool,
}

unsafe impl Send for Boson {}
unsafe impl Sync for Boson {}

impl Boson {
    pub(crate) fn new(gpu_controller: Arc<GpuController>) -> Self {
        Self {
            objects: Vec::new(),
            solvers: Vec::new(),
            gpu_controller,
            debugging: false,
        }
    }

    pub fn add_dynamic_object(&mut self, mut object: BosonObject) {
        match object.modify(|object| {
            object.build_collider(object.get_scale_factor(), self.gpu_controller.clone());

            if self.debugging {
                object.activate_debugger(self.gpu_controller.clone());
            }
        }) {
            Ok(_) => {
                self.objects.push(object);
                info!("Added Object to Boson");
            }
            Err(err) => {
                error!("Failed to add Boson Object due to: {}", err);
            }
        }
    }

    pub(crate) fn add_solver(&mut self, solver: impl Solver) {
        self.solvers.push(Arc::new(solver));
    }

    fn resolve_collisions(&mut self, delta_t: &Instant) {
        // Get all the collisions
        let mut collisions: Vec<Collision> = Vec::new();

        for (a_index, object_a) in self.objects.iter().enumerate() {
            let mut check_collision = true;

            // check if the collision needs to be checked
            match object_a.access(|object_a| match object_a {
                BosonBody::StaticCollider(_) | BosonBody::ParticleSystem(_) => {
                    check_collision = false
                }
                _ => {}
            }) {
                Err(err) => {
                    warn!(
                        "Failed to check if collision needs to be checked due to: {}",
                        err
                    );
                }
                _ => {}
            }

            if !check_collision {
                continue;
            }

            for (b_index, object_b) in self.objects.iter().enumerate() {
                if a_index == b_index {
                    continue;
                }

                let mut collision = None;

                match object_a.access(|object_a| {
                    object_b.access(|object_b| {
                        collision = object_a.test_collision(object_b);
                    })
                }) {
                    Err(err) => {
                        error!(
                            "Failed to access Boson objects to check for collision due to: {}",
                            err
                        );
                    }
                    _ => {}
                }

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

    #[allow(dead_code)]
    pub(crate) fn set_debugger<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut bool),
    {
        let old_debugging = self.debugging;
        callback(&mut self.debugging);

        if old_debugging != self.debugging {
            if self.debugging {
                for object in self.objects.iter_mut() {
                    match object.modify(|object| {
                        object.activate_debugger(self.gpu_controller.clone());
                    }) {
                        Err(err) => {
                            error!("Failed to set debugger on boson object due to: {}", err);
                        }
                        _ => {}
                    }
                }
            } else {
                for object in self.objects.iter_mut() {
                    match object.modify(|object| {
                        object.deactivate_debugger();
                    }) {
                        Err(err) => {
                            error!("Failed to set debugger on boson object due to: {}", err);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub(crate) fn debug_render(&self, render_pass: &mut RenderPass) {
        if self.debugging {
            for object in self.objects.iter() {
                match object.access(|object| {
                    object.debug_render(render_pass);
                }) {
                    Err(err) => {
                        error!("Failed to debug render boson object due to: {}", err);
                    }
                    _ => {}
                }
            }
        }
    }

    // Some GPU based systems should only be updated during render to avoid slowdown from writing to the gpu
    pub(crate) fn update_instances(&mut self, encoder: &mut CommandEncoder) {
        for object in self.objects.iter_mut() {
            match object.modify(|boson_body| boson_body.update_instances(encoder)) {
                Err(err) => {
                    error!("Failed to update instances of boson object due to: {}", err);
                }
                _ => {}
            }
        }
    }

    pub(crate) fn step(&mut self, delta_t: &Instant) {
        self.resolve_collisions(delta_t);

        for object in self.objects.iter_mut() {
            match object.modify(|boson_body| {
                boson_body.update(delta_t);
            }) {
                Err(err) => {
                    error!("Failed to step boson object due to: {}", err);
                }
                _ => {}
            }
        }
    }
}

// Internal struct for keeping track of which obejects have been added to the boson engine
#[derive(Debug)]
pub(crate) struct BosonAdded;
