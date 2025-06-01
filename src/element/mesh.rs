use std::{mem, sync::Arc};

use log::*;
use wgpu::{
    Buffer, BufferAddress, BufferUsages, IndexFormat, PolygonMode, RenderPass, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    GpuController, bind_group_builder,
    photon::render_descriptor::{
        PhotonRenderDescriptor, PhotonRenderDescriptorBuilder, STORAGE_RO,
    },
};

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
    pub(crate) label: String,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_indices: u32,
    pub instance_buffer: Buffer,
    pub instance_buffer_len: u32,
    pub transform: Arc<Buffer>,
    pub material: Arc<Material>,
    pub gpu_controller: Arc<GpuController>,

    // Cpu side processing
    pub(crate) vertices: Vec<ModelVertex>,
    pub(crate) indices: Vec<u32>,

    // Delegated Rendering
    render_descriptor: PhotonRenderDescriptor,
}

impl Mesh {
    pub fn new(
        label: String,
        vertices: &[ModelVertex],
        indices: &[u32],
        transform: Arc<Buffer>,
        gpu_controller: Arc<GpuController>,
        material: Option<Arc<Material>>,
    ) -> Self {
        let vertex_buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some(&format!("{} Vertex Buffer", label)),
                contents: bytemuck::cast_slice(vertices),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
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

        let material = if let Some(material) = material {
            material
        } else {
            Arc::new(Material::new_default(gpu_controller.clone()))
        };

        let render_descriptor = PhotonRenderDescriptorBuilder::default()
            .add_render_chain(material.render_descriptor.clone())
            .with_vertex_shader(include_str!("shaders/model_vert.wgsl"))
            .with_fragment_shader(include_str!("shaders/model_frag.wgsl"))
            .with_polygon_mode(PolygonMode::Fill)
            .with_label("Mesh")
            .with_vertex_buffer_layouts(&[ModelVertex::desc(), ModelInstance::desc()])
            .add_bind_group_with_layout(bind_group_builder!(
                gpu_controller.device,
                "Mesh",
                (0, VERTEX, transform.as_entire_binding(), STORAGE_RO)
            ))
            .build(gpu_controller.clone());

        Self {
            label,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            instance_buffer,
            instance_buffer_len: 1,
            transform,
            material,
            gpu_controller,
            vertices: Vec::from(vertices),
            indices: Vec::from(indices),
            render_descriptor,
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

    pub(crate) fn set_shaders(&mut self, vertex_shader: &str, fragment_shader: &str) {
        self.render_descriptor = PhotonRenderDescriptorBuilder::default()
            .add_render_chain(self.material.render_descriptor.clone())
            .with_vertex_shader(vertex_shader)
            .with_fragment_shader(fragment_shader)
            .with_polygon_mode(PolygonMode::Fill)
            .with_label("Mesh")
            .with_vertex_buffer_layouts(&[ModelVertex::desc(), ModelInstance::desc()])
            .add_bind_group_with_layout(bind_group_builder!(
                self.gpu_controller.device,
                "Mesh",
                (0, VERTEX, self.transform.as_entire_binding(), STORAGE_RO)
            ))
            .build(self.gpu_controller.clone())
    }

    // Only write to the instance buffer instead of mutating
    pub fn change_instance_buffer(&self, instances: &[ModelInstance]) {
        self.gpu_controller.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(instances),
        );
    }

    pub(crate) fn shift_vertices<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut ModelVertex),
    {
        self.vertices.iter_mut().for_each(|vertex| {
            callback(vertex);
        });

        // After modifying the vertices write it to the gpu
        self.gpu_controller.queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&self.vertices),
        );
    }

    pub(crate) fn render(&self, render_pass: &mut RenderPass) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), INDEX_FORMAT);
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        self.render_descriptor.setup_render(render_pass);

        render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instance_buffer_len);
    }
}
