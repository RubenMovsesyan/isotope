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

use super::{
    asset_manager::{AssetManager, SharedAsset},
    buffered::Buffered,
    material::Material,
    model_vertex::ModelVertex,
};

pub(crate) const INDEX_FORMAT: IndexFormat = IndexFormat::Uint32;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum Mesh {
    Unbuffered {
        vertices: Vec<ModelVertex>,
        indices: Vec<u32>,
        material: String,
        label: String,
    },
    Buffered {
        label: String,
        vertex_buffer: Buffer,
        index_buffer: Buffer,
        normals_buffer: Buffer,
        normals_index_buffer: Buffer,
        num_indices: u32,
        num_normal_indices: u32,
        material: SharedAsset<Material>,
        gpu_controller: Arc<GpuController>,

        // Cpu side processing
        vertices: Vec<ModelVertex>,
        indices: Vec<u32>,

        // Delegated Rendering
        render_descriptor: PhotonRenderDescriptor,
        debug_render_descriptor: PhotonRenderDescriptor,
        normals_render_descriptor: PhotonRenderDescriptor,
    },
}

impl From<&obj_loader::mesh::Mesh> for Mesh {
    fn from(value: &obj_loader::mesh::Mesh) -> Self {
        let vertices = value
            .faces
            .iter()
            .flat_map(|face| {
                face.points
                    .iter()
                    .map(|point| ModelVertex {
                        position: value.positions[point.position_index],
                        uv_coords: value.uvs[point.uv_index],
                        normal_vec: value.normals[point.normal_index],
                    })
                    .collect::<Vec<ModelVertex>>()
            })
            .collect::<Vec<ModelVertex>>();

        let indices = (0..vertices.len() as u32).into_iter().collect::<Vec<u32>>();

        Self::Unbuffered {
            vertices,
            indices,
            material: value.material.clone(),
            label: value.label.clone(),
        }
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self::Unbuffered {
            vertices: Vec::new(),
            indices: Vec::new(),
            material: "".to_owned(),
            label: "".to_owned(),
        }
    }
}

impl Mesh {
    pub fn with_label(label: String) -> Self {
        Self::Unbuffered {
            vertices: Vec::new(),
            indices: Vec::new(),
            material: "".to_owned(),
            label,
        }
    }

    pub fn label(&self) -> String {
        match self {
            Mesh::Unbuffered { label, .. } => label.to_owned(),
            Mesh::Buffered { label, .. } => label.to_owned(),
        }
    }

    pub fn buffer(self, transform: &Buffer, asset_manager: &mut AssetManager) -> Self {
        if let Mesh::Unbuffered {
            vertices,
            indices,
            material,
            label,
        } = self
        {
            let gpu_controller = asset_manager.gpu_controller.clone();

            let vertex_buffer = gpu_controller
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    label: Some(&format!("{} Vertex Buffer", label)),
                    contents: bytemuck::cast_slice(&vertices),
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
                    contents: bytemuck::cast_slice(&indices),
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

            let material = asset_manager.get_material(material);
            info!(
                "Mesh {} using material: {}",
                label,
                material.with_read(|mat| mat.label())
            );

            let (mesh_bind_group_layout, mesh_bind_group) = bind_group_builder!(
                gpu_controller.device,
                "Mesh",
                (0, VERTEX, transform.as_entire_binding(), STORAGE_RO)
            );

            let render_descriptor = PhotonRenderDescriptorBuilder::default()
                .add_render_chain(material.with_read(|material| match material {
                    Material::Buffered {
                        render_descriptor, ..
                    } => render_descriptor.clone(),
                    _ => {
                        unimplemented!()
                    }
                }))
                .with_vertex_shader(include_str!("shaders/model_vert.wgsl"))
                .with_fragment_shader(include_str!("shaders/model_frag.wgsl"))
                .with_polygon_mode(PolygonMode::Fill)
                .with_label("Mesh")
                .with_vertex_buffer_layouts(&[ModelVertex::desc(), ModelInstance::desc()])
                .add_bind_group_with_layout((
                    mesh_bind_group_layout.clone(),
                    mesh_bind_group.clone(),
                ))
                .build(gpu_controller.clone());

            let debug_render_descriptor = PhotonRenderDescriptorBuilder::default()
                .with_vertex_shader(include_str!("shaders/model_vert_debug.wgsl"))
                .with_fragment_shader(include_str!("shaders/model_frag_debug.wgsl"))
                .with_polygon_mode(PolygonMode::Line)
                .with_label("Mesh Debug")
                .with_vertex_buffer_layouts(&[ModelVertex::desc(), ModelInstance::desc()])
                .add_bind_group_with_layout((
                    mesh_bind_group_layout.clone(),
                    mesh_bind_group.clone(),
                ))
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

            Self::Buffered {
                label,
                vertex_buffer,
                index_buffer,
                normals_buffer,
                normals_index_buffer,
                num_indices: indices.len() as u32,
                num_normal_indices: normal_indices.len() as u32,
                vertices,
                indices,
                material,
                gpu_controller,
                render_descriptor,
                debug_render_descriptor,
                normals_render_descriptor,
            }
        } else {
            self
        }
    }

    // Shifts all the vertices of the model with a given callback
    pub(crate) fn shift_vertices<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut ModelVertex),
    {
        let vertices = match self {
            Mesh::Unbuffered { vertices, .. } => vertices,
            Mesh::Buffered { vertices, .. } => vertices,
        };

        vertices.iter_mut().for_each(|vertex| {
            callback(vertex);
        });

        // After modifying the vertices write it to the gpu
        match self {
            Mesh::Buffered {
                gpu_controller,
                vertex_buffer,
                vertices,
                ..
            } => {
                gpu_controller.queue.write_buffer(
                    &vertex_buffer,
                    0,
                    bytemuck::cast_slice(&vertices),
                );
            }
            _ => {}
        }
    }

    pub(crate) fn render(&self, render_pass: &mut RenderPass, instance_count: u32) {
        match self {
            Mesh::Buffered {
                vertex_buffer,
                index_buffer,
                render_descriptor,
                num_indices,
                ..
            } => {
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), INDEX_FORMAT);

                render_descriptor.setup_render(render_pass);

                render_pass.draw_indexed(0..*num_indices, 0, 0..instance_count);
            }
            _ => {}
        }
    }

    pub(crate) fn debug_render(&self, render_pass: &mut RenderPass, instance_count: u32) {
        match self {
            Mesh::Buffered {
                vertex_buffer,
                index_buffer,
                debug_render_descriptor,
                num_indices,
                normals_buffer,
                normals_index_buffer,
                normals_render_descriptor,
                num_normal_indices,
                ..
            } => {
                // Draw the mesh lines
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), INDEX_FORMAT);

                debug_render_descriptor.setup_render(render_pass);

                render_pass.draw_indexed(0..*num_indices, 0, 0..instance_count);

                // Draw the normals
                render_pass.set_vertex_buffer(0, normals_buffer.slice(..));
                render_pass.set_index_buffer(normals_index_buffer.slice(..), INDEX_FORMAT);

                normals_render_descriptor.setup_render(render_pass);

                render_pass.draw_indexed(0..*num_normal_indices, 0, 0..instance_count);
            }
            _ => {}
        }
    }

    // pub(crate) fn set_shaders(&mut self, vertex_shader: &str, fragment_shader: &str) {
    //     self.render_descriptor = PhotonRenderDescriptorBuilder::default()
    //         .add_render_chain(self.material.render_descriptor.clone())
    //         .with_vertex_shader(vertex_shader)
    //         .with_fragment_shader(fragment_shader)
    //         .with_polygon_mode(PolygonMode::Fill)
    //         .with_label("Mesh")
    //         .with_vertex_buffer_layouts(&[ModelVertex::desc(), ModelInstance::desc()])
    //         .add_bind_group_with_layout(bind_group_builder!(
    //             self.gpu_controller.device,
    //             "Mesh",
    //             (0, VERTEX, self.transform.as_entire_binding(), STORAGE_RO)
    //         ))
    //         .build(self.gpu_controller.clone())
    // }

    // #[allow(dead_code)]
    // pub(crate) fn set_debug_shaders(&mut self, vertex_shader: &str, fragment_shader: &str) {
    //     self.debug_render_descriptor = PhotonRenderDescriptorBuilder::default()
    //         .add_render_chain(self.material.render_descriptor.clone())
    //         .with_vertex_shader(vertex_shader)
    //         .with_fragment_shader(fragment_shader)
    //         .with_polygon_mode(PolygonMode::Line)
    //         .with_label("Mesh Debug")
    //         .with_vertex_buffer_layouts(&[ModelVertex::desc(), ModelInstance::desc()])
    //         .add_bind_group_with_layout(bind_group_builder!(
    //             self.gpu_controller.device,
    //             "Mesh",
    //             (0, VERTEX, self.transform.as_entire_binding(), STORAGE_RO)
    //         ))
    //         .build(self.gpu_controller.clone())
    // }
}
