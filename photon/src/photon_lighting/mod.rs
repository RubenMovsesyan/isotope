use std::sync::Arc;

use anyhow::Result;
use gpu_controller::GpuController;
pub use light::Light;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, ShaderStages,
    util::BufferInitDescriptor,
};

// TODO: Change the size of later
const MAX_LIGHTS: usize = 1024;

pub mod light;

pub struct LightsManager {
    gpu_controller: Arc<GpuController>,
    lights_buffer: Buffer,
    num_lights_buffer: Buffer,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl LightsManager {
    pub fn new(gpu_controller: Arc<GpuController>) -> Result<Self> {
        // Create the bind group layout for the lights
        let bind_group_layout =
            gpu_controller.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Lights Bind Group Layout"),
                entries: &[
                    // Array of Lights
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Number of Lights
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // Create buffers
        let lights_buffer = gpu_controller.create_buffer_init(&BufferInitDescriptor {
            label: Some("Lights Buffer"),
            contents: bytemuck::cast_slice(&vec![Light::default(); MAX_LIGHTS]),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        let num_lights_buffer = gpu_controller.create_buffer_init(&BufferInitDescriptor {
            label: Some("Num Lights Buffer"),
            contents: bytemuck::cast_slice(&[0u32]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Create the bind group
        let bind_group = gpu_controller.create_bind_group(&BindGroupDescriptor {
            label: Some("Lighting Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                // Lights Buffer
                BindGroupEntry {
                    binding: 0,
                    resource: lights_buffer.as_entire_binding(),
                },
                // Number of Lights
                BindGroupEntry {
                    binding: 1,
                    resource: num_lights_buffer.as_entire_binding(),
                },
            ],
        });

        Ok(Self {
            gpu_controller,
            lights_buffer,
            num_lights_buffer,
            bind_group_layout,
            bind_group,
        })
    }
}
