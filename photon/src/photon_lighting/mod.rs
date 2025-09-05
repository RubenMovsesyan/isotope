use std::{ops::Range, sync::Arc};

use anyhow::Result;
use gpu_controller::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferInitDescriptor,
    BufferUsages, GpuController, ShaderStages,
};
pub use light::Light;
use log::{debug, warn};

// TODO: Change the size of later
const MAX_LIGHTS: usize = 1024;

pub mod light;

pub struct LightsManager {
    gpu_controller: Arc<GpuController>,
    lights_buffer: Buffer,
    num_lights_buffer: Buffer,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,

    lights: [Light; MAX_LIGHTS],
    num_lights: u32,
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
            lights: [Light::default(); MAX_LIGHTS],
            num_lights: 0,
        })
    }

    // Copys the light to the buffer
    // pub fn add_light(&mut self, light: &Light) {
    //     if self.num_lights < MAX_LIGHTS as u32 {
    //         self.lights[self.num_lights as usize] = *light;
    //         self.num_lights += 1;
    //         self.update_buffer();
    //     }
    // }

    // pub fn update_lights(&mut self, lights: &[Light], range: Range<u32>) {
    //     self.lights[range.start as usize..range.end as usize].copy_from_slice(lights);
    //     self.update_buffer_range(range);
    // }

    // Updates the buffer with the current lights
    pub fn update_lights(&mut self, lights: &[Light]) {
        if lights.len() > MAX_LIGHTS {
            warn!("Too many lights");
        } else {
            for (i, light) in lights.iter().enumerate() {
                self.lights[i] = *light;
            }
            self.num_lights = lights.len() as u32;
        }

        self.gpu_controller.write_buffer(
            &self.lights_buffer,
            0,
            bytemuck::cast_slice(&self.lights),
        );

        self.gpu_controller.write_buffer(
            &self.num_lights_buffer,
            0,
            bytemuck::cast_slice(&[self.num_lights]),
        );
    }

    // fn update_buffer_range(&self, range: Range<u32>) {
    //     self.gpu_controller.write_buffer(
    //         &self.lights_buffer,
    //         range.start as u64 * std::mem::size_of::<Light>() as u64,
    //         bytemuck::cast_slice(&self.lights[range.start as usize..range.end as usize]),
    //     );
    // }
}
