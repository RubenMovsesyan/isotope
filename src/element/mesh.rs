use std::sync::Arc;

use log::*;
use wgpu::{
    Buffer, BufferUsages, IndexFormat, PolygonMode, RenderPass,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    GpuController, bind_group_builder,
    element::model::ModelInstance,
    photon::render_descriptor::{
        PhotonRenderDescriptor, PhotonRenderDescriptorBuilder, STORAGE_RO,
    },
};

use super::{buffered::Buffered, material::Material, model_vertex::ModelVertex};

pub(crate) const INDEX_FORMAT: IndexFormat = IndexFormat::Uint32;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Mesh {
    pub(crate) label: String,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_indices: u32,
    pub transform: Arc<Buffer>,
    pub material: Arc<Material>,
    pub gpu_controller: Arc<GpuController>,

    // Cpu side processing
    pub(crate) vertices: Vec<ModelVertex>,
    pub(crate) indices: Vec<u32>,

    // Delegated Rendering
    render_descriptor: PhotonRenderDescriptor,
    debug_render_descriptor: PhotonRenderDescriptor,
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

        info!("Created New Mesh: {}", label);

        let material = if let Some(material) = material {
            material
        } else {
            Arc::new(Material::new_default(gpu_controller.clone()))
        };

        let (mesh_bind_group_layout, mesh_bind_group) = bind_group_builder!(
            gpu_controller.device,
            "Mesh",
            (0, VERTEX, transform.as_entire_binding(), STORAGE_RO)
        );

        let render_descriptor = PhotonRenderDescriptorBuilder::default()
            .add_render_chain(material.render_descriptor.clone())
            .with_vertex_shader(include_str!("shaders/model_vert.wgsl"))
            .with_fragment_shader(include_str!("shaders/model_frag.wgsl"))
            .with_polygon_mode(PolygonMode::Fill)
            .with_label("Mesh")
            .with_vertex_buffer_layouts(&[ModelVertex::desc(), ModelInstance::desc()])
            .add_bind_group_with_layout((mesh_bind_group_layout.clone(), mesh_bind_group.clone()))
            .build(gpu_controller.clone());

        let debug_render_descriptor = PhotonRenderDescriptorBuilder::default()
            .with_vertex_shader(include_str!("shaders/model_vert_debug.wgsl"))
            .with_fragment_shader(include_str!("shaders/model_frag_debug.wgsl"))
            .with_polygon_mode(PolygonMode::Line)
            .with_label("Mesh Debug")
            .with_vertex_buffer_layouts(&[ModelVertex::desc(), ModelInstance::desc()])
            .add_bind_group_with_layout((mesh_bind_group_layout, mesh_bind_group))
            .build(gpu_controller.clone());

        Self {
            label,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            transform,
            material,
            gpu_controller,
            vertices: Vec::from(vertices),
            indices: Vec::from(indices),
            render_descriptor,
            debug_render_descriptor,
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

    pub(crate) fn set_debug_shaders(&mut self, vertex_shader: &str, fragment_shader: &str) {
        self.debug_render_descriptor = PhotonRenderDescriptorBuilder::default()
            .add_render_chain(self.material.render_descriptor.clone())
            .with_vertex_shader(vertex_shader)
            .with_fragment_shader(fragment_shader)
            .with_polygon_mode(PolygonMode::Line)
            .with_label("Mesh Debug")
            .with_vertex_buffer_layouts(&[ModelVertex::desc(), ModelInstance::desc()])
            .add_bind_group_with_layout(bind_group_builder!(
                self.gpu_controller.device,
                "Mesh",
                (0, VERTEX, self.transform.as_entire_binding(), STORAGE_RO)
            ))
            .build(self.gpu_controller.clone())
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

    pub(crate) fn render(&self, render_pass: &mut RenderPass, instance_count: u32) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), INDEX_FORMAT);

        self.render_descriptor.setup_render(render_pass);

        render_pass.draw_indexed(0..self.num_indices, 0, 0..instance_count);
    }

    pub(crate) fn debug_render(&self, render_pass: &mut RenderPass, instance_count: u32) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), INDEX_FORMAT);

        self.debug_render_descriptor.setup_render(render_pass);

        render_pass.draw_indexed(0..self.num_indices, 0, 0..instance_count);
    }
}
