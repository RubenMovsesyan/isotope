use std::{mem, path::Path};

use cgmath::{InnerSpace, MetricSpace, Vector3};
use log::*;
use model_fragment::ModelFragment;
use wgpu::{
    BufferAddress, CommandEncoder, RenderPass, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexStepMode,
};

use crate::{
    BosonBody, BosonObject, Transform,
    boson::Linkable,
    photon::{instancer::Instance, renderer::camera::PhotonCamera},
};

use super::{asset_manager::AssetManager, buffered::Buffered, mesh::Mesh};

mod model_fragment;

type DistanceDescriptor = fn(f32) -> usize;

#[repr(C)]
#[derive(Debug, bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub struct ModelInstance {
    position: [f32; 3],
    _padding: f32,
    orientation: [f32; 4],
}

unsafe impl Instance for ModelInstance {}

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
                    offset: mem::size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 4,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl Default for ModelInstance {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            _padding: 0.0,
            orientation: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelTransform {
    position: [f32; 3],
    _padding: f32, // IMPORTANT: MAKE SURE TO HAVE THE PADDING IN THE RIGHT PLACE
    orientation: [f32; 4],
}

#[derive(Debug)]
pub enum Model {
    Single(ModelFragment),
    Lod(DistanceDescriptor, Vec<ModelFragment>),
}

#[allow(dead_code)]
impl Model {
    pub fn from_obj<P>(path: P, asset_manager: &mut AssetManager) -> Self
    where
        P: AsRef<Path>,
    {
        Self::Single(ModelFragment::from_obj(path, asset_manager))
    }

    pub fn from_objs<P>(
        paths: &[P],
        asset_manager: &mut AssetManager,
        distance_descriptor: DistanceDescriptor,
    ) -> Self
    where
        P: AsRef<Path>,
    {
        Self::Lod(
            distance_descriptor,
            paths
                .iter()
                .map(|path| ModelFragment::from_obj(path, asset_manager))
                .collect::<Vec<_>>(),
        )
    }

    pub(crate) fn link_transform(&mut self, transform: &Transform) {
        let model_transform = ModelTransform {
            position: transform.position.into(),
            orientation: transform.orientation.into(),
            _padding: 0.0,
        };

        match self {
            Model::Single(model_fragment) => {
                model_fragment.culling_position = transform.position.into();
                model_fragment.write_transform(model_transform);
            }
            Model::Lod(_, model_fragments) => {
                model_fragments.iter_mut().for_each(|model_fragment| {
                    model_fragment.culling_position = transform.position.into();
                    model_fragment.write_transform(model_transform);
                });
            }
        }
    }

    pub(crate) fn link_boson(&mut self, boson_object: &BosonObject) {
        match boson_object.access(|object| match object {
            BosonBody::ParticleSystem(particle_system) => {
                if let Some(particle_instancer) = particle_system.get_instancer() {
                    match self {
                        Model::Single(model_fragment) => {
                            model_fragment.instancer = particle_instancer;
                        }
                        Model::Lod(_, model_fragments) => {
                            model_fragments.iter_mut().for_each(|model_fragment| {
                                model_fragment.instancer = particle_instancer.clone();
                            });
                        }
                    }
                }
            }
            _ => {}
        }) {
            Ok(_) => {}
            Err(err) => {
                error!("Unable to link boson object due to: {}", err);
            }
        }
    }

    pub(crate) fn compute_instances(&self, encoder: &mut CommandEncoder) {
        match self {
            Model::Single(model_fragment) => {
                model_fragment.instancer.compute_instances(|_| {}, encoder);
            }
            Model::Lod(_, model_fragments) => {
                // Only need to use the first instancer because
                // they all share the same instancer
                model_fragments[0]
                    .instancer
                    .compute_instances(|_| {}, encoder);
            }
        }
    }

    pub(crate) fn boson_meshes(&self) -> &Vec<Mesh> {
        match self {
            Model::Single(model_fragment) => &model_fragment.meshes,
            Model::Lod(_, model_fragments) => &model_fragments[0].meshes,
        }
    }

    pub(crate) fn boson_meshes_mut(&mut self) -> &mut Vec<Mesh> {
        match self {
            Model::Single(model_fragment) => &mut model_fragment.meshes,
            Model::Lod(_, model_fragments) => &mut model_fragments[0].meshes,
        }
    }

    pub fn render(&self, render_pass: &mut RenderPass, camera: &PhotonCamera) {
        match self {
            Model::Single(model_fragment) => {
                render_pass
                    .set_vertex_buffer(1, model_fragment.instancer.instance_buffer.slice(..));

                for mesh in model_fragment.meshes.iter() {
                    mesh.render(
                        render_pass,
                        model_fragment.instancer.instance_count as u32,
                        &model_fragment.culling_position,
                        camera,
                    );
                }
            }
            Model::Lod(distance_function, model_fragments) => {
                render_pass
                    .set_vertex_buffer(1, model_fragments[0].instancer.instance_buffer.slice(..));

                // The culling position is the same for all models
                let position: Vector3<f32> = model_fragments[0].culling_position.into();
                let distance = camera.eye.to_homogeneous().truncate().distance(position);
                let model_index = distance_function(distance);

                for mesh in model_fragments[model_index].meshes.iter() {
                    mesh.render(
                        render_pass,
                        model_fragments[model_index].instancer.instance_count as u32,
                        &model_fragments[model_index].culling_position,
                        camera,
                    );
                }
            }
        }
    }

    pub unsafe fn debug_render(&self, render_pass: &mut RenderPass, camera: &PhotonCamera) {
        match self {
            Model::Single(model_fragment) => {
                render_pass
                    .set_vertex_buffer(1, model_fragment.instancer.instance_buffer.slice(..));

                for mesh in model_fragment.meshes.iter() {
                    mesh.debug_render(
                        render_pass,
                        model_fragment.instancer.instance_count as u32,
                        &model_fragment.culling_position,
                        camera,
                    );
                }
            }
            Model::Lod(distance_function, model_fragments) => {
                render_pass
                    .set_vertex_buffer(1, model_fragments[0].instancer.instance_buffer.slice(..));

                // The culling position is the same for all models
                let position: Vector3<f32> = model_fragments[0].culling_position.into();
                let model_index = distance_function(position.magnitude());

                for mesh in model_fragments[model_index].meshes.iter() {
                    mesh.debug_render(
                        render_pass,
                        model_fragments[model_index].instancer.instance_count as u32,
                        &model_fragments[model_index].culling_position,
                        camera,
                    );
                }
            }
        }
    }
}
