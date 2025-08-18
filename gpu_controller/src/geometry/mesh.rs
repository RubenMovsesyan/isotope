use std::{f32, mem, sync::Arc};

use log::debug;
use wgpu::{
    BindGroupLayoutDescriptor, Buffer, BufferBindingType, BufferDescriptor, BufferUsages,
    IndexFormat, RenderPass, util::BufferInitDescriptor,
};

use crate::GpuController;

use super::vertex::Vertex;

// CPU side mesh
pub struct MeshDescriptor {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct Mesh {
    gpu_controller: Arc<GpuController>,

    // GPU side
    label: String,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,

    // CPU side
    mesh_descriptor: MeshDescriptor,
}

// logic used in this crate
trait Distance {
    fn dist(&self) -> f32;
}

impl Distance for [f32; 3] {
    fn dist(&self) -> f32 {
        f32::sqrt(self.iter().map(|vertex_pos| vertex_pos.powi(2)).sum())
    }
}

// For loading in custom obj files
impl From<&obj_loader::mesh::Mesh> for MeshDescriptor {
    fn from(value: &obj_loader::mesh::Mesh) -> Self {
        let vertices = value
            .faces
            .iter()
            .flat_map(|face| {
                face.points
                    .iter()
                    .map(|point| {
                        Vertex::new(
                            value.positions[point.position_index],
                            value.uvs[point.uv_index],
                            value.normals[point.normal_index],
                        )
                    })
                    .collect::<Vec<Vertex>>()
            })
            .collect::<Vec<Vertex>>();

        let indices = (0..vertices.len() as u32).into_iter().collect::<Vec<u32>>();

        // TODO impelment in the future
        let _cullable_radius = vertices
            .iter()
            .map(|vertex| vertex.position.dist())
            .fold(f32::NEG_INFINITY, |a, b| a.max(b));

        debug!("Model Cullable radius: {}", _cullable_radius);

        Self { vertices, indices }
    }
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
            mesh_descriptor: MeshDescriptor { vertices, indices },
        }
    }

    pub fn from_mesh_descriptor(
        gpu_controller: Arc<GpuController>,
        label: String,
        mesh_descriptor: MeshDescriptor,
    ) -> Self {
        let vertex_buffer = gpu_controller.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("{} Vertex Buffer", label)),
            contents: bytemuck::cast_slice(&mesh_descriptor.vertices),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let index_buffer = gpu_controller.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("{} Index Buffer", label)),
            contents: bytemuck::cast_slice(&mesh_descriptor.indices),
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        });

        Self {
            gpu_controller,
            label,
            vertex_buffer,
            index_buffer,
            num_indices: mesh_descriptor.indices.len() as u32,
            mesh_descriptor,
        }
    }

    pub fn render(&self, render_pass: &mut RenderPass) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

        // render_pass.draw(0..8, 0..1);
    }
}
