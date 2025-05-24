use std::{mem, sync::Arc};

use log::*;
use wgpu::{
    Buffer, BufferAddress, BufferUsages, IndexFormat, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::GpuController;

use super::{buffered::Buffered, material::Material, model_vertex::ModelVertex};

pub(crate) const INDEX_FORMAT: IndexFormat = IndexFormat::Uint32;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelInstance {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
}

impl Default for ModelInstance {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

impl Buffered for ModelInstance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: mem::size_of::<ModelInstance>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                // Position
                VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: VertexFormat::Float32x3,
                },
                // Rotation
                VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 4,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Mesh {
    label: String,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_indices: u32,
    pub instance_buffer: Buffer,
    pub instance_buffer_len: u32,
    pub material: Option<Arc<Material>>,
    pub gpu_controller: Arc<GpuController>,
}

impl Mesh {
    pub fn new(
        label: String,
        vertices: &[ModelVertex],
        indices: &[u32],
        gpu_controller: Arc<GpuController>,
    ) -> Self {
        let vertex_buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some(&format!("{} Vertex Buffer", label)),
                contents: bytemuck::cast_slice(vertices),
                usage: BufferUsages::VERTEX,
            });

        let index_buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some(&format!("{} Index Buffer", label)),
                contents: bytemuck::cast_slice(indices),
                usage: BufferUsages::INDEX,
            });

        let instance_buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Model Instance Buffer"),
                contents: bytemuck::cast_slice(&[ModelInstance::default()]),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });

        info!("Created New Mesh: {}", label);

        Self {
            label,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            instance_buffer,
            instance_buffer_len: 1,
            material: None,
            gpu_controller,
        }
    }

    // Write the instance buffer to the gpu or create a new one if the size has changed
    pub fn set_instance_buffer(&mut self, instances: &[ModelInstance]) {
        let length = instances.len() as u32;

        if length != self.instance_buffer_len {
            self.instance_buffer =
                self.gpu_controller
                    .device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: Some("Model Instance Buffer"),
                        contents: bytemuck::cast_slice(instances),
                        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                    });

            self.instance_buffer_len = length;
            info!("Regenerated Instance Buffer");
        } else {
            self.gpu_controller.queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(instances),
            );
        }
    }

    // Only write to the instance buffer instead of mutating
    pub fn change_instance_buffer(&self, instances: &[ModelInstance]) {
        self.gpu_controller.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(instances),
        );
    }
}
