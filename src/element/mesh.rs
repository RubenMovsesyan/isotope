use std::sync::Arc;

use log::*;
use wgpu::{
    Buffer, BufferUsages, IndexFormat,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::GpuController;

use super::{material::Material, model_vertex::ModelVertex};

pub(crate) const INDEX_FORMAT: IndexFormat = IndexFormat::Uint32;

#[derive(Debug)]
pub struct Mesh {
    label: String,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_indices: u32,
    pub instance_buffer: Option<Buffer>,
    pub material: Option<Arc<Material>>,
}

impl Mesh {
    pub fn new(
        label: String,
        vertices: &[ModelVertex],
        indices: &[u32],
        gpu_controller: &GpuController,
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

        info!("Created New Mesh: {}", label);

        Self {
            label,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            instance_buffer: None,
            material: None,
        }
    }
}
