use std::{f32, sync::Arc};

use log::{debug, info, warn};
use wgpu::{Buffer, BufferUsages, IndexFormat, RenderPass, util::BufferInitDescriptor};

use crate::GpuController;

use super::vertex::Vertex;

pub enum Mesh {
    Cpu {
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
    },
    Gpu {
        gpu_controller: Arc<GpuController>,

        label: String,
        vertex_buffer: Buffer,
        index_buffer: Buffer,
        num_indices: u32,
    },
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
impl From<&obj_loader::mesh::Mesh> for Mesh {
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

        Self::Cpu { vertices, indices }
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

        Self::Gpu {
            gpu_controller,
            label,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }

    pub fn buffer(&mut self, label: String, gpu_controller: Arc<GpuController>) {
        match self {
            Self::Cpu { vertices, indices } => {
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

                *self = Self::Gpu {
                    gpu_controller,
                    label,
                    vertex_buffer,
                    index_buffer,
                    num_indices: indices.len() as u32,
                };
            }
            Self::Gpu { .. } => {}
        }

        info!("New mesh has been buffered");
    }

    pub fn render(&self, render_pass: &mut RenderPass) {
        match self {
            Self::Cpu { .. } => {
                warn!("Mesh in unbuffered CPU state, not rendered");
            }
            Self::Gpu {
                vertex_buffer,
                index_buffer,
                num_indices,
                ..
            } => {
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
                render_pass.draw_indexed(0..*num_indices, 0, 0..1);
            }
        }
    }
}
