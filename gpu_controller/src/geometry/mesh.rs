use std::{f32, sync::Arc};

use anyhow::{Result, anyhow};
use log::{debug, info, warn};
use wgpu::{Buffer, BufferUsages, IndexFormat, RenderPass, util::BufferInitDescriptor};

use crate::GpuController;

use super::vertex::Vertex;

pub enum Mesh {
    Cpu {
        label: String,

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

    pub fn label(&self) -> &String {
        match self {
            Mesh::Cpu { label, .. } => &label,
            Mesh::Gpu { label, .. } => &label,
        }
    }

    pub fn buffer(&mut self, gpu_controller: Arc<GpuController>) {
        match self {
            Self::Cpu {
                label,
                vertices,
                indices,
            } => {
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
                    label: label.clone(), // TODO: find a better way to do this
                    vertex_buffer,
                    index_buffer,
                    num_indices: indices.len() as u32,
                };
            }
            Self::Gpu { .. } => {}
        }

        info!("New mesh has been buffered");
    }

    pub fn vertices<F, R>(&mut self, vertices_callback: F) -> Result<R>
    where
        F: FnOnce(&mut Vec<Vertex>) -> R,
    {
        match self {
            Self::Cpu { vertices, .. } => Ok(vertices_callback(vertices)),
            Self::Gpu { .. } => Err(anyhow!("Mesh is in GPU state, cannot access vertices")),
        }
    }

    pub fn indices<F, R>(&mut self, indices_callback: F) -> Result<R>
    where
        F: FnOnce(&mut Vec<u32>) -> R,
    {
        match self {
            Self::Cpu { indices, .. } => Ok(indices_callback(indices)),
            Self::Gpu { .. } => Err(anyhow!("Mesh is in GPU state, cannot access indices")),
        }
    }

    pub fn vertices_indices<F, R>(&mut self, vertices_indices_callback: F) -> Result<R>
    where
        F: FnOnce(&mut Vec<Vertex>, &mut Vec<u32>) -> R,
    {
        match self {
            Self::Cpu {
                vertices, indices, ..
            } => Ok(vertices_indices_callback(vertices, indices)),
            Self::Gpu { .. } => Err(anyhow!(
                "Mesh is in GPU state, cannot access vertices and indices"
            )),
        }
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
