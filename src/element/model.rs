use std::{
    mem,
    path::Path,
    sync::{Arc, RwLock},
    time::Instant,
};

use anyhow::{Result, anyhow};
use cgmath::{One, Quaternion, Vector3, Zero};
use log::*;
use obj_loader::Obj;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferAddress, BufferDescriptor,
    BufferUsages, CommandEncoder, RenderPass, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    GpuController, ParallelInstancerBuilder, Transform, bind_group_builder,
    boson::Linkable,
    element::model_vertex::{ModelVertex, VertexNormalVec, VertexPosition, VertexUvCoord},
    photon::{
        instancer::{Instance, InstanceBufferDescriptor, Instancer},
        render_descriptor::STORAGE_RO,
    },
    utils::file_io::read_lines,
};

use super::{
    asset_manager::{AssetManager, SharedAsset},
    buffered::Buffered,
    material::Material,
    mesh::Mesh,
};

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

#[allow(dead_code)]
#[derive(Debug)]
pub struct Model {
    pub(crate) meshes: Vec<Mesh>,
    materials: Vec<SharedAsset<Material>>,

    // GPU
    transform_buffer: Arc<Buffer>,
    // transform_bind_group: BindGroup,
    gpu_controller: Arc<GpuController>,

    // Physics Linking
    // boson_link: Option<Arc<RwLock<dyn Linkable>>>,

    // For gpu instancing
    instancer: Arc<Instancer<ModelInstance>>,
    // Temp
    // time_buffer: Buffer,
    // time: Instant,
}

impl Model {
    pub fn from_obj<P>(path: P, asset_manager: Arc<RwLock<AssetManager>>) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        debug!("Full Path: {:#?}", path.as_ref());
        if let Ok(mut asset_manager) = asset_manager.write() {
            let gpu_controller = asset_manager.gpu_controller.clone();

            let model_obj = if let Ok(obj) = Obj::new(&path) {
                obj
            } else {
                todo!()
            };

            // Create the transform buffer as it is needed for the meshes to reference
            let transform_buffer = gpu_controller.device.create_buffer(&BufferDescriptor {
                label: Some("Model Transform"),
                mapped_at_creation: false,
                size: std::mem::size_of::<ModelTransform>() as u64,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            });

            let path_parent = if let Some(parent) = path.as_ref().parent() {
                parent
            } else {
                Path::new("")
            };

            // Load all the materials first in case we need some
            let materials = model_obj
                .materials
                .iter()
                .map(|(_, material)| {
                    // Search for the material in the asset manager and add it if not in there
                    let new_material: Material = material.into();

                    if let Some(material) = asset_manager.search_material(new_material.label()) {
                        material
                    } else {
                        info!(
                            "Adding New Material to Asset Manager: {}",
                            new_material.label()
                        );
                        let buffered = new_material.buffer(path_parent, &mut asset_manager);
                        asset_manager.add_material(buffered)
                    }
                })
                .collect::<Vec<SharedAsset<Material>>>();

            let meshes = model_obj
                .meshes
                .iter()
                .map(|mesh| {
                    let new_mesh: Mesh = mesh.into();
                    new_mesh.buffer(&transform_buffer, &mut asset_manager)
                })
                .collect::<Vec<Mesh>>();

            let instancer = Arc::new(Instancer::new_series(
                gpu_controller.clone(),
                InstanceBufferDescriptor::Size(1),
                "Mesh",
            ));

            Ok(Self {
                gpu_controller,
                meshes,
                materials,
                transform_buffer: Arc::new(transform_buffer),
                instancer,
            })
        } else {
            Err(anyhow!("Asset Manager Poisoned"))
        }
    }

    pub(crate) fn link_transform(&self, tranform: &Transform) {
        let model_transform = ModelTransform {
            position: tranform.position.into(),
            orientation: tranform.orientation.into(),
            _padding: 0.0,
        };

        self.gpu_controller.queue.write_buffer(
            &self.transform_buffer,
            0,
            bytemuck::cast_slice(&[model_transform]),
        );
    }

    pub fn compute_instances(&self, encoder: &mut CommandEncoder) {
        self.instancer.compute_instances(|_| {}, encoder);
    }

    pub fn render(&mut self, render_pass: &mut RenderPass) {
        render_pass.set_vertex_buffer(1, self.instancer.instance_buffer.slice(..));

        for mesh in self.meshes.iter() {
            mesh.render(render_pass, self.instancer.instance_count as u32);
        }
    }

    ///! Always call after main render
    pub unsafe fn debug_render(&self, render_pass: &mut RenderPass) {
        render_pass.set_vertex_buffer(1, self.instancer.instance_buffer.slice(..));

        for mesh in self.meshes.iter() {
            mesh.debug_render(render_pass, self.instancer.instance_count as u32)
        }
    }

    // pub fn with_custom_shaders(
    //     mut self,
    //     vertex_shader: &str,
    //     fragment_shader: &str,
    // ) -> Result<Self> {
    //     self.meshes.iter_mut().for_each(|mesh| {
    //         mesh.with_write(|mesh| mesh.set_shaders(vertex_shader, fragment_shader));
    //     });

    //     Ok(self)
    // }

    // pub fn with_custom_time_instancer(mut self, compute_shader: &str, instances: u64) -> Self {
    //     let instancer: Instancer<ModelInstance> = ParallelInstancerBuilder::default()
    //         .add_bind_group_with_layout(bind_group_builder!(
    //             self.gpu_controller.device,
    //             "Time Instancer",
    //             (0, COMPUTE, self.time_buffer.as_entire_binding(), STORAGE_RO)
    //         ))
    //         .with_instance_count(instances)
    //         .with_label("Time Instancer")
    //         .with_compute_shader(compute_shader)
    //         .build(self.gpu_controller.clone())
    //         .expect("Failed to build model instancer");

    //     self.instancer = Arc::new(instancer);

    //     self
    // }

    // // Modifying the position
    // pub fn pos<F>(&mut self, callback: F)
    // where
    //     F: FnOnce(&mut Vector3<f32>),
    // {
    //     callback(&mut self.position);
    //     self.transform_dirty = true;
    // }

    // // Modifying the rotaition
    // pub fn rot<F>(&mut self, callback: F)
    // where
    //     F: FnOnce(&mut Quaternion<f32>),
    // {
    //     callback(&mut self.rotation);
    //     self.transform_dirty = true;
    // }
}
