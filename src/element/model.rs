use std::{
    mem,
    path::Path,
    sync::{Arc, RwLock},
    time::Instant,
};

use anyhow::{Result, anyhow};
use cgmath::{One, Quaternion, Vector3, Zero};
use log::*;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferAddress, BufferDescriptor,
    BufferUsages, CommandEncoder, RenderPass, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    GpuController, ParallelInstancerBuilder, Transform, bind_group_builder,
    boson::Linkable,
    element::{
        material::load_materials,
        model_vertex::{ModelVertex, VertexNormalVec, VertexPosition, VertexUvCoord},
    },
    photon::{
        instancer::{Instance, InstanceBufferDescriptor, Instancer},
        render_descriptor::STORAGE_RO,
    },
    utils::file_io::read_lines,
};

use super::{asset_manager::AssetManager, buffered::Buffered, material::Material, mesh::Mesh};

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
    materials: Vec<Arc<Material>>,

    // Global position and rotation
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
    transform_dirty: bool,

    // GPU
    transform_buffer: Arc<Buffer>,
    transform_bind_group: BindGroup,
    gpu_controller: Arc<GpuController>,

    // Physics Linking
    boson_link: Option<Arc<RwLock<dyn Linkable>>>,

    // For gpu instancing
    instancer: Arc<Instancer<ModelInstance>>,

    // Temp
    time_buffer: Buffer,
    time: Instant,
}

impl Model {
    pub fn from_obj<P>(path: P, asset_manager: Arc<RwLock<AssetManager>>) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let gpu_controller = if let Ok(asset_manager) = asset_manager.read() {
            asset_manager.gpu_controller.clone()
        } else {
            return Err(anyhow!("Asset Manager Poisoned"));
        };

        info!(
            "Loading Object: {:#?}",
            path.as_ref()
                .to_str()
                .ok_or(anyhow!("Object Path Not Valid"))?
        );

        // Obj file reading variables
        let mut mesh_name: Option<String> = None;
        let mut vertices: Vec<VertexPosition> = Vec::new();
        let mut uv_coords: Vec<VertexUvCoord> = Vec::new();
        let mut normals: Vec<VertexNormalVec> = Vec::new();

        let mut model_vertices: Vec<ModelVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let mut meshes: Vec<Mesh> = Vec::new();
        let mut materials: Vec<Arc<Material>> = Vec::new();

        let mut material_index: Option<usize> = None;

        // Create the buffer for global tranformations
        let transform_buffer = Arc::new(gpu_controller.device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Model Transform Buffer"),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&[ModelTransform {
                    position: [0.0, 0.0, 0.0],
                    orientation: [0.0, 0.0, 0.0, 1.0],
                    _padding: 0.0,
                }]),
            },
        ));

        let lines = read_lines(&path)?;

        for line in lines.map_while(Result::ok) {
            let line_split = line.split_whitespace().collect::<Vec<_>>();

            if line_split.is_empty() {
                continue;
            }

            match line_split[0] {
                // Object definition
                "o" => {
                    // If there is a mesh currently in the buffer then add it to the model
                    if let Some(name) = mesh_name.take() {
                        let new_mesh = Mesh::new(
                            name,
                            &model_vertices,
                            &indices,
                            transform_buffer.clone(),
                            asset_manager.clone(),
                            if let Some(material_index) = material_index.take() {
                                Some(materials[material_index].clone())
                            } else {
                                None
                            },
                        );

                        meshes.push(new_mesh);

                        model_vertices.clear();
                        indices.clear();
                    }

                    mesh_name = Some(line_split[1].to_string());
                }
                // vertex
                "v" => {
                    vertices.push([
                        line_split[1].parse::<f32>()?,
                        line_split[2].parse::<f32>()?,
                        line_split[3].parse::<f32>()?,
                    ]);
                }
                // uv coordinate
                "vt" => {
                    uv_coords.push([
                        1.0 - line_split[1].parse::<f32>()?,
                        1.0 - line_split[2].parse::<f32>()?,
                    ]);
                }
                // vertex normal
                "vn" => {
                    normals.push([
                        line_split[1].parse::<f32>()?,
                        line_split[2].parse::<f32>()?,
                        line_split[3].parse::<f32>()?,
                    ]);
                }
                // face
                "f" => {
                    for vertex_info in line_split[1..=3].iter() {
                        let vertex_info_split = vertex_info.split('/').collect::<Vec<_>>();

                        // Get the indices of each vertex, uv, and normal for the face
                        let (vertex_index, uv_index, normal_index) = (
                            vertex_info_split[0].parse::<usize>()? - 1,
                            vertex_info_split[1].parse::<usize>()? - 1,
                            vertex_info_split[2].parse::<usize>()? - 1,
                        );

                        model_vertices.push(ModelVertex::new(
                            vertices[vertex_index],
                            uv_coords[uv_index],
                            normals[normal_index],
                        ));

                        indices.push(model_vertices.len() as u32 - 1);
                    }
                }
                // material
                "mtllib" => {
                    let path_to_material = path
                        .as_ref()
                        .parent()
                        .ok_or(anyhow!("Obj Path is invalid"))?
                        .join(line_split[1]);

                    // Add all the found materials to the materials
                    materials.append(&mut load_materials(
                        asset_manager.clone(),
                        path_to_material,
                    )?);
                }
                // object using the material
                "usemtl" => {
                    debug!("Searching for material: {}", line_split[1]);
                    debug!("Num Materials: {}", materials.len());
                    material_index = Some(
                        *materials
                            .iter()
                            .enumerate()
                            .filter_map(|(ind, m)| {
                                info!("Searching: {}", m.name);
                                if &m.name == line_split[1] {
                                    info!("Found: {}", line_split[1]);
                                    Some(ind)
                                } else {
                                    warn!("Skipping: {}", line_split[1]);
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                            .first()
                            .ok_or(anyhow!("Material Invalid"))?,
                    );
                }
                _ => {}
            }
        }

        // Add the remaining object to the list
        if let Some(name) = mesh_name.take() {
            let new_mesh = Mesh::new(
                name,
                &model_vertices,
                &indices,
                transform_buffer.clone(),
                asset_manager.clone(),
                if let Some(material_index) = material_index.take() {
                    Some(materials[material_index].clone())
                } else {
                    None
                },
            );
            meshes.push(new_mesh);
        }

        // Create the bind group from the layout
        let transform_bind_group = gpu_controller
            .device
            .create_bind_group(&BindGroupDescriptor {
                label: Some("Model Transform Bind Group"),
                // Safetey: Photon should alread exist at this point
                layout: &gpu_controller.layouts.transform_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: transform_buffer.as_entire_binding(),
                }],
            });

        let instancer = Instancer::new_series(
            gpu_controller.clone(),
            InstanceBufferDescriptor::Size(1),
            "Default Model Instance",
        );

        let time_buffer = gpu_controller.device.create_buffer(&BufferDescriptor {
            label: Some("Time Buffer"),
            mapped_at_creation: false,
            size: std::mem::size_of::<f32>() as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        Ok(Self {
            meshes,
            materials,
            position: Vector3::zero(),
            rotation: Quaternion::one(),
            transform_dirty: false,
            transform_buffer,
            transform_bind_group,
            gpu_controller,
            boson_link: None,
            instancer: Arc::new(instancer),
            time_buffer,
            time: Instant::now(),
        })
    }

    pub fn with_custom_shaders(
        mut self,
        vertex_shader: &str,
        fragment_shader: &str,
    ) -> Result<Self> {
        self.meshes.iter_mut().for_each(|mesh| {
            mesh.set_shaders(vertex_shader, fragment_shader);
        });

        Ok(self)
    }

    pub fn with_custom_time_instancer(mut self, compute_shader: &str, instances: u64) -> Self {
        let instancer: Instancer<ModelInstance> = ParallelInstancerBuilder::default()
            .add_bind_group_with_layout(bind_group_builder!(
                self.gpu_controller.device,
                "Time Instancer",
                (0, COMPUTE, self.time_buffer.as_entire_binding(), STORAGE_RO)
            ))
            .with_instance_count(instances)
            .with_label("Time Instancer")
            .with_compute_shader(compute_shader)
            .build(self.gpu_controller.clone())
            .expect("Failed to build model instancer");

        self.instancer = Arc::new(instancer);

        self
    }

    // Modifying the position
    pub fn pos<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut Vector3<f32>),
    {
        callback(&mut self.position);
        self.transform_dirty = true;
    }

    // Modifying the rotaition
    pub fn rot<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut Quaternion<f32>),
    {
        callback(&mut self.rotation);
        self.transform_dirty = true;
    }

    // // Linking a boson object to the model for physics
    // pub fn link(&mut self, linkable: impl Into<Arc<RwLock<dyn Linkable>>>) {
    //     let l: Arc<RwLock<dyn Linkable>> = linkable.into();
    //     if let Ok(l) = l.write() {
    //         self.position = l.get_position();
    //         self.rotation = l.get_orientation();

    //         if let Some(instancer) = l.get_instancer() {
    //             self.instancer = instancer;
    //         }
    //     }

    //     self.boson_link = Some(l);

    //     // adujst the vertices of the model so that the center of mass is the origin
    //     let center_of_mass = calculate_center_of_mass(self);
    //     info!(
    //         "Center of mass: {:#?}\nMoving origin to center",
    //         center_of_mass
    //     );

    //     for mesh in self.meshes.iter_mut() {
    //         mesh.shift_vertices(|model_vertex| {
    //             model_vertex.position =
    //                 (Vector3::from(model_vertex.position) - center_of_mass).into();
    //         });
    //     }
    // }
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
        // // If the model is linked then update the position
        // if let Some(boson_link) = self.boson_link.as_ref() {
        //     if let Ok(boson_link) = boson_link.read() {
        //         // Set the models position based off the boson object
        //         self.position = boson_link.get_position();
        //         self.rotation = boson_link.get_orientation();
        //         self.transform_dirty = true;
        //     }
        // }

        // let time_elapsed = self.time.elapsed().as_secs_f32();

        // // Write the time to the buffer
        // self.gpu_controller.queue.write_buffer(
        //     &self.time_buffer,
        //     0,
        //     bytemuck::cast_slice(&[time_elapsed]),
        // );

        // if self.transform_dirty {
        //     // Update the position buffer if changed
        //     let model_transform = ModelTransform {
        //         position: self.position.into(),
        //         orientation: self.rotation.into(),
        //         _padding: 0.0,
        //     };

        //     self.gpu_controller.queue.write_buffer(
        //         &self.transform_buffer,
        //         0,
        //         bytemuck::cast_slice(&[model_transform]),
        //     );

        //     self.transform_dirty = false;
        // }

        render_pass.set_vertex_buffer(1, self.instancer.instance_buffer.slice(..));

        for mesh in self.meshes.iter() {
            mesh.render(render_pass, self.instancer.instance_count as u32);
        }
    }

    ///! Always call after main render
    pub unsafe fn debug_render(&self, render_pass: &mut RenderPass) {
        render_pass.set_vertex_buffer(1, self.instancer.instance_buffer.slice(..));

        for mesh in self.meshes.iter() {
            mesh.debug_render(render_pass, self.instancer.instance_count as u32);
        }
    }
}
