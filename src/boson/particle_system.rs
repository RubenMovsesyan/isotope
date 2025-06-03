use std::{sync::Arc, time::Instant};

use cgmath::{One, Quaternion, Vector3, Zero};
use log::{debug, warn};
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    Instancer, Isotope, ParallelInstancerBuilder, bind_group_builder,
    element::model::ModelInstance,
    photon::render_descriptor::{STORAGE_RO, STORAGE_RW},
};

use super::{BosonBody, Linkable};

#[derive(Debug)]
pub struct InitialState {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
}

impl Into<InitialCondition> for InitialState {
    fn into(self) -> InitialCondition {
        InitialCondition {
            position: self.position.into(),
            _padding0: 0.0,
            velocity: self.velocity.into(),
            _padding1: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InitialCondition {
    pub position: [f32; 3],
    _padding0: f32,
    pub velocity: [f32; 3],
    _padding1: f32,
}

#[derive(Debug)]
pub struct ParticleSysytem {
    pub(crate) position: Vector3<f32>,
    pub(crate) velocity: Vector3<f32>,
    pub(crate) orientation: Quaternion<f32>,

    particle_count: u64,

    instancer: Arc<Instancer<ModelInstance>>,

    // delta_t accumulator to avoid particle sytem physics slowdown
    delta_t_accum: f32,

    // Buffers
    delta_time_buffer: Buffer,
    initial_condition_buffer: Buffer,
    velocity_buffer: Buffer,
    reset_buffer: Buffer,
}

impl ParticleSysytem {
    pub fn new(particle_count: u64, isotope: &Isotope) -> Self {
        let position = Vector3::zero();
        let velocity = Vector3::zero();
        let orientation = Quaternion::one();

        let gpu_controller = isotope.gpu_controller.clone();

        // To write delta_t to every iteration
        let delta_time_buffer = gpu_controller.device.create_buffer(&BufferDescriptor {
            label: Some("Particle System Time Buffer"),
            mapped_at_creation: false,
            size: std::mem::size_of::<f32>() as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        // Initial conditions of the particle system
        let initial_condition_buffer = gpu_controller.device.create_buffer(&BufferDescriptor {
            label: Some("Particle System Initial Condition Buffer"),
            mapped_at_creation: false,
            size: std::mem::size_of::<InitialCondition>() as u64 * particle_count,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        // To write a reset signal to
        let reset_buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Particle System Reset Buffer"),
                contents: bytemuck::cast_slice(&[1u32]),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            });

        // To Keep track of the particles velocities
        let velocity_buffer = gpu_controller.device.create_buffer(&BufferDescriptor {
            label: Some("Particle System Velocity Buffer"),
            mapped_at_creation: false,
            size: std::mem::size_of::<[f32; 3]>() as u64 * particle_count,
            usage: BufferUsages::STORAGE,
        });

        let instancer = Arc::new(
            ParallelInstancerBuilder::default()
                .add_bind_group_with_layout(bind_group_builder!(
                    gpu_controller.device,
                    "Particle System",
                    (
                        0,
                        COMPUTE,
                        delta_time_buffer.as_entire_binding(),
                        STORAGE_RO
                    ),
                    (
                        1,
                        COMPUTE,
                        initial_condition_buffer.as_entire_binding(),
                        STORAGE_RO
                    ),
                    (2, COMPUTE, reset_buffer.as_entire_binding(), STORAGE_RW),
                    (3, COMPUTE, velocity_buffer.as_entire_binding(), STORAGE_RW)
                ))
                .with_instance_count(particle_count)
                .with_label("Particle System")
                .with_compute_shader(include_str!("shaders/ic_particle_system.wgsl"))
                .build(gpu_controller)
                .expect("Failed to create Particle System Instancer"),
        );

        Self {
            position,
            velocity,
            orientation,
            particle_count,

            delta_time_buffer,
            delta_t_accum: 0.0,
            initial_condition_buffer,
            reset_buffer,
            velocity_buffer,
            instancer,
        }
    }

    pub fn set_initial_conditions(&self, initial_conditions: Vec<InitialState>) {
        if initial_conditions.len() != self.particle_count as usize {
            warn!("Initial Conditions Len Not Equal to the particle size");
        }

        self.instancer.write_to_buffer(
            &self.initial_condition_buffer,
            bytemuck::cast_slice(
                &initial_conditions
                    .into_iter()
                    .enumerate()
                    .filter_map(|(index, initial_state)| {
                        // Make sure we only take the instances that are bounded by the number of particles
                        if index < self.particle_count as usize {
                            Some(initial_state.into())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<InitialCondition>>(),
            ),
        );

        // Reset with the new initial conditions
        self.reset_system();
    }

    pub fn reset_system(&self) {
        debug!("Resetting Particle System");
        self.instancer
            .write_to_buffer(&self.reset_buffer, bytemuck::cast_slice(&[1u32]));
    }

    pub(crate) fn update(&mut self, delta_t: &Instant) {
        let dt = delta_t.elapsed().as_secs_f32();

        self.delta_t_accum += dt;

        // // Write the delta time to the buffer
        // self.instancer
        //     .write_to_buffer(&self.delta_time_buffer, bytemuck::cast_slice(&[dt]));

        // // Step through the physics step
        // self.instancer.compute_instances(|_| {});
    }

    pub(crate) fn update_on_render(&mut self) {
        // Write the delta time to the buffer
        self.instancer.write_to_buffer(
            &self.delta_time_buffer,
            bytemuck::cast_slice(&[self.delta_t_accum]),
        );

        self.delta_t_accum = 0.0;

        // Step through the physics step
        self.instancer.compute_instances(|_| {});
    }

    pub(crate) fn apply_force(&mut self, force: Vector3<f32>, delta_t: &Instant) {
        // For simplicity
        let dt = delta_t.elapsed().as_secs_f32();

        // v = v_0 + F/m * t
        self.velocity += force * dt; // For now the mass is 1.0 but should be changed later
        // x = x_0 + v * t
        self.position += self.velocity * dt;
    }
}

impl Linkable for ParticleSysytem {
    fn get_position(&self) -> Vector3<f32> {
        self.position
    }

    fn get_orientation(&self) -> Quaternion<f32> {
        self.orientation
    }

    fn get_instancer(&self) -> Option<Arc<Instancer<ModelInstance>>> {
        Some(self.instancer.clone())
    }
}

impl Into<BosonBody> for ParticleSysytem {
    fn into(self) -> BosonBody {
        BosonBody::ParticleSystem(self)
    }
}
