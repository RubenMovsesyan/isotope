use bytemuck::Zeroable;
use light::Light;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferUsages,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::GpuController;

use super::photon_layouts::PhotonLayoutsManager;

pub mod light;

#[derive(Debug)]
pub(crate) struct Lights {
    pub buffer: Buffer,
    pub num_lights_buffer: Buffer,
    pub bind_group: BindGroup,
    pub num_lights: usize,
}

impl Lights {
    // Create the Lights given a list of lights
    pub fn new_with_lights(
        gpu_controller: &GpuController,
        photon_layouts: &PhotonLayoutsManager,
        lights: &[Light],
    ) -> Self {
        // Create a buffer with room for at least 1 element in it
        // let buffer = match lights.len() {
        //     0 => gpu_controller.device.create_buffer(&BufferDescriptor {
        //         label: Some("Lights Buffer"),
        //         mapped_at_creation: false,
        //         size: std::mem::size_of::<Light>() as u64,
        //         usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        //     }),
        //     _ => gpu_controller
        //         .device
        //         .create_buffer_init(&BufferInitDescriptor {
        //             label: Some("Lights Buffer"),
        //             contents: bytemuck::cast_slice(lights),
        //             usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        //         }),
        // };

        let lights_buf = (lights.len()..256)
            .into_iter()
            .map(|_| Light::zeroed())
            .collect::<Vec<Light>>();

        let buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Lights Buffer"),
                contents: bytemuck::cast_slice(&[lights, &lights_buf].concat()),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            });

        let num_lights_buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Lights Len Buffer"),
                contents: bytemuck::cast_slice(&[lights.len() as u32]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

        let bind_group = gpu_controller
            .device
            .create_bind_group(&BindGroupDescriptor {
                label: Some("Lights Bind Group"),
                layout: &photon_layouts.lights_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: num_lights_buffer.as_entire_binding(),
                    },
                ],
            });

        Self {
            buffer,
            bind_group,
            num_lights_buffer,
            num_lights: lights.len(),
        }
    }

    // TODO: add index of light update
    pub fn update(
        &mut self,
        gpu_controller: &GpuController,
        photon_layouts: &PhotonLayoutsManager,
        lights: &[Light],
    ) {
        // If the number of lights is different then we need to create a new buffer
        if lights.len() != self.num_lights {
            let lights_buf = (lights.len()..256)
                .into_iter()
                .map(|_| Light::zeroed())
                .collect::<Vec<Light>>();

            let buffer = gpu_controller
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    label: Some("Lights Buffer"),
                    contents: bytemuck::cast_slice(&[lights, &lights_buf].concat()),
                    usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                });

            let num_lights_buffer =
                gpu_controller
                    .device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: Some("Lights Len Buffer"),
                        contents: bytemuck::cast_slice(&[lights.len() as u32]),
                        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                    });

            let bind_group = gpu_controller
                .device
                .create_bind_group(&BindGroupDescriptor {
                    label: Some("Lights Bind Group"),
                    layout: &photon_layouts.lights_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: buffer.as_entire_binding(),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: num_lights_buffer.as_entire_binding(),
                        },
                    ],
                });

            self.buffer = buffer;
            self.bind_group = bind_group;
            self.num_lights_buffer = num_lights_buffer;
            self.num_lights = lights.len();
        } else {
            // If the lights are the same length then just update the values
            gpu_controller
                .queue
                .write_buffer(&self.buffer, 0, bytemuck::cast_slice(lights));
        }
    }
}
