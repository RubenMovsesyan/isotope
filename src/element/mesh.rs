use std::sync::Arc;

use log::*;
use wgpu::{
    Buffer, BufferUsages, IndexFormat, PolygonMode, PrimitiveTopology, RenderPass,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    GpuController, bind_group_builder,
    element::{model::ModelInstance, model_vertex::ModelNormalVertex},
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
    pub normals_buffer: Buffer,
    pub normals_index_buffer: Buffer,
    pub num_indices: u32,
    pub num_normal_indices: u32,
    pub transform: Arc<Buffer>,
    pub material: Arc<Material>,
    pub gpu_controller: Arc<GpuController>,

    // Cpu side processing
    pub(crate) vertices: Vec<ModelVertex>,
    pub(crate) indices: Vec<u32>,

    // Delegated Rendering
    render_descriptor: PhotonRenderDescriptor,
    debug_render_descriptor: PhotonRenderDescriptor,
    normals_render_descriptor: PhotonRenderDescriptor,
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

        // Create the vertices which are the base of the normal and the point that it leads to
        let normal_vertices = {
            indices
                .chunks(3)
                .map(|triangle_indices| {
                    let vertex_1 = triangle_indices[0];
                    let vertex_2 = triangle_indices[1];
                    let vertex_3 = triangle_indices[2];

                    let position_1 = vertices[vertex_1 as usize].position;
                    let position_2 = vertices[vertex_2 as usize].position;
                    let position_3 = vertices[vertex_3 as usize].position;

                    let normal_1 = vertices[vertex_1 as usize].normal_vec;
                    let normal_2 = vertices[vertex_2 as usize].normal_vec;
                    let normal_3 = vertices[vertex_3 as usize].normal_vec;

                    let position_avg = [
                        (position_1[0] + position_2[0] + position_3[0]) / 3.0,
                        (position_1[1] + position_2[1] + position_3[1]) / 3.0,
                        (position_1[2] + position_2[2] + position_3[2]) / 3.0,
                    ];

                    let normal_point = [
                        ((normal_1[0] + normal_2[0] + normal_3[0]) / 3.0) + position_avg[0],
                        ((normal_1[1] + normal_2[1] + normal_3[1]) / 3.0) + position_avg[1],
                        ((normal_1[2] + normal_2[2] + normal_3[2]) / 3.0) + position_avg[2],
                    ];

                    Vec::from([position_avg, normal_point])
                })
                .flat_map(|vertex| vertex)
                .collect::<Vec<[f32; 3]>>()
        };

        let normals_buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Mesh Normals Buffer"),
                contents: bytemuck::cast_slice(&normal_vertices),
                usage: BufferUsages::VERTEX,
            });

        let index_buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some(&format!("{} Index Buffer", label)),
                contents: bytemuck::cast_slice(indices),
                usage: BufferUsages::INDEX,
            });

        // Create the list of indices that connect the normal vectors
        let normal_indices = (0..normal_vertices.len())
            .into_iter()
            .map(|index| index as u32)
            .collect::<Vec<u32>>();

        let normals_index_buffer =
            gpu_controller
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    label: Some("Mesh Normals Index Buffer"),
                    contents: bytemuck::cast_slice(&normal_indices),
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
            .add_bind_group_with_layout((mesh_bind_group_layout.clone(), mesh_bind_group.clone()))
            .build(gpu_controller.clone());

        let normals_render_descriptor = PhotonRenderDescriptorBuilder::default()
            .with_vertex_shader(include_str!("shaders/model_normals_vert_debug.wgsl"))
            .with_fragment_shader(include_str!("shaders/model_normals_frag_debug.wgsl"))
            .with_polygon_mode(PolygonMode::Line)
            .with_primitive_topology(PrimitiveTopology::LineList)
            .with_label("Mesh Normals")
            .with_vertex_buffer_layouts(&[ModelNormalVertex::desc(), ModelInstance::desc()])
            .add_bind_group_with_layout((mesh_bind_group_layout, mesh_bind_group))
            .build(gpu_controller.clone());

        Self {
            label,
            vertex_buffer,
            index_buffer,
            normals_buffer,
            normals_index_buffer,
            num_normal_indices: normal_indices.len() as u32,
            num_indices: indices.len() as u32,
            transform,
            material,
            gpu_controller,
            vertices: Vec::from(vertices),
            indices: Vec::from(indices),
            render_descriptor,
            debug_render_descriptor,
            normals_render_descriptor,
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
        // Draw the mesh lines
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), INDEX_FORMAT);

        self.debug_render_descriptor.setup_render(render_pass);

        render_pass.draw_indexed(0..self.num_indices, 0, 0..instance_count);

        // Draw the normals
        render_pass.set_vertex_buffer(0, self.normals_buffer.slice(..));
        render_pass.set_index_buffer(self.normals_index_buffer.slice(..), INDEX_FORMAT);

        self.normals_render_descriptor.setup_render(render_pass);

        render_pass.draw_indexed(0..self.num_normal_indices, 0, 0..instance_count);
    }
}
