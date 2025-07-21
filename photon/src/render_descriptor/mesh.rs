use std::{mem, sync::Arc};

use gpu_controller::GpuController;
use wgpu::{
    BindGroupLayoutDescriptor, Buffer, BufferBindingType, BufferDescriptor, BufferUsages,
    util::BufferInitDescriptor,
};

use super::vertex::Vertex;

pub struct Mesh {
    gpu_controller: Arc<GpuController>,

    // GPU side
    label: String,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,

    // CPU side
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Mesh {
    pub fn new(
        gpu_controller: Arc<GpuController>,
        label: String,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Self {
        let vertices = Vec::from(vertices);
        let indices = Vec::from(indices);

        let vertex_buffer = gpu_controller.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("{} Vertex Buffer", label)),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let index_buffer = gpu_controller.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("{} Index Buffer", label)),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        });

        Self {
            gpu_controller,
            label,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            vertices,
            indices,
        }
    }
}
