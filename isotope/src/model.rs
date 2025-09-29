use std::{
    fs::File,
    io::{BufRead, BufReader},
    ops::Range,
    path::Path,
    sync::Arc,
};

use anyhow::{Result, anyhow};
use cgmath::{Matrix4, Quaternion, SquareMatrix, Vector3, Zero};
use gpu_controller::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor, BufferInitDescriptor,
    BufferUsages, GpuController, INSTANCE_BUFFER_INDEX, Instance, MaintainBase, MapMode, Mesh,
    RenderPass, Vertex,
};
use log::{debug, info};
use matter_vault::SharedMatter;
use photon::{MATERIALS_BIND_GROUP, renderer::GLOBAL_TRANSFORM_BIND_GROUP};

use crate::{
    Transform3D,
    asset_server::AssetServer,
    material::{Material, load_materials},
};

const INSTANCE_SIZE: u64 = std::mem::size_of::<Instance>() as u64;

type Position = [f32; 3];
type Normal = [f32; 3];
type UV = [f32; 2];

pub struct Model {
    gpu_controller: Arc<GpuController>,
    meshes: Vec<(Option<usize>, SharedMatter<Mesh>)>,
    materials: Vec<SharedMatter<Material>>,

    global_transform_bind_group: BindGroup,
    global_transformation_buffer: Buffer,
    instance_buffer: Buffer,
    num_instances: u32,

    instance_staging_buffer: Buffer,
}

impl Model {
    pub fn from_obj<P>(
        path: P,
        asset_server: &AssetServer,
        instances: Option<&[Instance]>,
    ) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        debug!("Full Path {:#?}", path.as_ref());

        info!("Retriving wavefrom from {:#?}", path.as_ref());
        let file = File::open(&path)?;

        // Read the obj file line by line
        let lines = BufReader::new(file).lines();

        let mut current_mesh: Option<Mesh> = None;
        let mut current_material_index: Option<usize> = None;
        let mut label_exists = false;

        // For mesh building
        let mut positions: Vec<Position> = Vec::new();
        let mut normals: Vec<Normal> = Vec::new();
        let mut uvs: Vec<UV> = Vec::new();

        // For the end return
        let mut meshes: Vec<(Option<usize>, SharedMatter<Mesh>)> = Vec::new();
        let mut materials: Vec<SharedMatter<Material>> = Vec::new();

        for line in lines.map_while(Result::ok) {
            let tokens = line.split_whitespace().collect::<Vec<_>>();

            if tokens.is_empty() {
                continue;
            }

            if !label_exists {
                match tokens[0] {
                    "o" => {
                        if let Some(mut mesh) = current_mesh.take() {
                            mesh.buffer(asset_server.gpu_controller.clone());
                            let mesh_label = mesh.label().clone();
                            meshes.push((
                                current_material_index,
                                asset_server.asset_manager.add(mesh_label, mesh)?,
                            ));
                        }

                        let label = tokens[1].to_string();

                        // If the mesh is already shared, add it to the list of meshes
                        if let Ok(mesh) = asset_server.asset_manager.share(&label) {
                            debug!("Mesh already exists: {}", label);
                            meshes.push((current_material_index, mesh));
                            label_exists = true;
                        } else {
                            debug!("Creating new mesh: {}", label);
                            current_mesh = Some(Mesh::Cpu {
                                label: label.clone(),
                                vertices: Vec::new(),
                                indices: Vec::new(),
                            });
                        }
                    }
                    // Vertex Position
                    "v" => {
                        positions.push([
                            tokens[1].parse()?,
                            tokens[2].parse()?,
                            tokens[3].parse()?,
                        ]);
                    }
                    // Vertex UV
                    "vt" => {
                        uvs.push([tokens[1].parse()?, tokens[2].parse()?]);
                    }
                    // Vertex Normal
                    "vn" => {
                        normals.push([tokens[1].parse()?, tokens[2].parse()?, tokens[3].parse()?]);
                    }
                    // Connecting Face
                    "f" => {
                        let mut vertices: Vec<Vertex> = Vec::new();

                        for vertex_info in tokens[1..].iter() {
                            let vertex_tokens = vertex_info.split('/').collect::<Vec<_>>();

                            vertices.push(Vertex {
                                position: positions[vertex_tokens[0].parse::<usize>()? - 1],
                                uv_coord: uvs[vertex_tokens[1].parse::<usize>()? - 1],
                                normal_vec: normals[vertex_tokens[2].parse::<usize>()? - 1],
                            });
                        }

                        match current_mesh.as_mut() {
                            Some(mesh) => {
                                mesh.vertices_indices(|mesh_vertices, mesh_indices| {
                                    mesh_indices.append(
                                        &mut (0..vertices.len())
                                            .into_iter()
                                            .map(|i| (i + mesh_vertices.len()) as u32)
                                            .collect::<Vec<u32>>(),
                                    );

                                    mesh_vertices.append(&mut vertices);
                                })?;
                            }
                            None => {}
                        }
                    }
                    "mtllib" => {
                        let path_to_material = path
                            .as_ref()
                            .parent()
                            .ok_or(anyhow!("Obj Path is invalid"))?
                            .join(tokens[1]);

                        materials = load_materials(&path_to_material, asset_server)?;
                    }
                    "usemtl" => {
                        let material_name = tokens[1].to_string();
                        current_material_index =
                            materials.iter().enumerate().find_map(|(index, material)| {
                                if material.read(|m| m.label == material_name) {
                                    Some(index)
                                } else {
                                    None
                                }
                            });
                    }
                    _ => {}
                }
            } else {
                match tokens[0] {
                    "o" => {
                        label_exists = false;

                        let label = tokens[1].to_string();

                        // If the mesh is already shared, add it to the list of meshes
                        if let Ok(mesh) = asset_server.asset_manager.share(&label) {
                            debug!("Mesh already exists: {}", label);
                            meshes.push((current_material_index, mesh));
                            label_exists = true;
                        } else {
                            debug!("Creating new mesh: {}", label);
                            current_mesh = Some(Mesh::Cpu {
                                label: label.clone(),
                                vertices: Vec::new(),
                                indices: Vec::new(),
                            });
                        }
                    }
                    "mtllib" => {
                        let path_to_material = path
                            .as_ref()
                            .parent()
                            .ok_or(anyhow!("Obj Path is invalid"))?
                            .join(tokens[1]);

                        materials = load_materials(&path_to_material, asset_server)?;
                    }
                    "usemtl" => {
                        let material_name = tokens[1].to_string();
                        current_material_index =
                            materials.iter().enumerate().find_map(|(index, material)| {
                                if material.read(|m| m.label == material_name) {
                                    Some(index)
                                } else {
                                    None
                                }
                            });
                    }
                    _ => {}
                }
            }
        }

        if let Some(mut mesh) = current_mesh.take() {
            mesh.buffer(asset_server.gpu_controller.clone());
            let mesh_label = mesh.label().clone();
            meshes.push((
                current_material_index,
                asset_server.asset_manager.add(mesh_label, mesh)?,
            ));
        }

        // Create the instance buffer for the model
        let (instance_buffer, num_instances) = if let Some(instances) = instances {
            (
                asset_server
                    .gpu_controller
                    .create_buffer_init(&BufferInitDescriptor {
                        label: Some("Model Instance Buffer"),
                        usage: BufferUsages::VERTEX
                            | BufferUsages::COPY_DST
                            | BufferUsages::COPY_SRC,
                        contents: bytemuck::cast_slice(&instances),
                    }),
                instances.len() as u32,
            )
        } else {
            (
                asset_server
                    .gpu_controller
                    .create_buffer_init(&BufferInitDescriptor {
                        label: Some("Model Instance Buffer"),
                        usage: BufferUsages::VERTEX
                            | BufferUsages::COPY_DST
                            | BufferUsages::COPY_SRC,
                        contents: bytemuck::cast_slice(&[Instance::new(
                            Vector3::zero(),
                            Quaternion::new(0.0, 0.0, 0.0, 1.0),
                            Matrix4::identity(),
                        )]),
                    }),
                1,
            )
        };

        let instance_staging_buffer =
            asset_server
                .gpu_controller
                .create_buffer(&BufferDescriptor {
                    label: Some("Model Instance Staging Buffer"),
                    size: num_instances as u64 * INSTANCE_SIZE,
                    usage: BufferUsages::MAP_READ
                        | BufferUsages::MAP_WRITE
                        | BufferUsages::COPY_SRC
                        | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

        let global_transformation_buffer =
            asset_server
                .gpu_controller
                .create_buffer_init(&BufferInitDescriptor {
                    label: Some("Global Transformation Buffer"),
                    usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                    contents: bytemuck::cast_slice(&[Transform3D::default()]),
                });

        let global_transform_bind_group = asset_server.gpu_controller.read_layouts(|layouts| {
            asset_server
                .gpu_controller
                .create_bind_group(&BindGroupDescriptor {
                    label: Some("Global Transform Bind Group"),
                    layout: &layouts["Global Transform"],
                    entries: &[BindGroupEntry {
                        binding: 0,
                        resource: global_transformation_buffer.as_entire_binding(),
                    }],
                })
        })?;

        Ok(Self {
            gpu_controller: asset_server.gpu_controller.clone(),
            meshes,
            materials,
            instance_buffer,
            global_transform_bind_group,
            global_transformation_buffer,
            num_instances,
            instance_staging_buffer,
        })
    }

    pub fn render(&self, render_pass: &mut RenderPass) {
        for (material_index, mesh) in self.meshes.iter() {
            mesh.read(|mesh| {
                if let Some(material_index) = material_index.as_ref() {
                    self.materials[*material_index].read(|material| {
                        render_pass.set_bind_group(MATERIALS_BIND_GROUP, &material.bind_group, &[]);
                    });
                }

                render_pass.set_bind_group(
                    GLOBAL_TRANSFORM_BIND_GROUP,
                    &self.global_transform_bind_group,
                    &[],
                );

                render_pass
                    .set_vertex_buffer(INSTANCE_BUFFER_INDEX, self.instance_buffer.slice(..));

                mesh.render(render_pass, self.num_instances);
            });
        }
    }

    pub fn modify_instances<F>(&self, range: Option<Range<u64>>, callback: F) -> Result<()>
    where
        F: FnOnce(&mut [Instance]),
    {
        let range = range.unwrap_or(0..self.num_instances as u64);
        let byte_range = (range.start * INSTANCE_SIZE)..(range.end * INSTANCE_SIZE);

        // Copy the instance buffer to mappable buffer
        let mut encoder = self
            .gpu_controller
            .create_command_encoder("Instance Update Copy To");
        encoder.copy_buffer_to_buffer(
            &self.instance_buffer,
            byte_range.start,
            &self.instance_staging_buffer,
            byte_range.start,
            byte_range.end - byte_range.start,
        );
        self.gpu_controller.submit(encoder);
        self.gpu_controller.poll(MaintainBase::Wait)?;

        // Map the buffer for the CPU to read
        let buffer_slice = self.instance_staging_buffer.slice(byte_range.clone());
        buffer_slice.map_async(MapMode::Write, |_| {});
        self.gpu_controller.poll(MaintainBase::Wait)?;

        {
            let mut mapped_data = buffer_slice.get_mapped_range_mut();
            let instances = bytemuck::cast_slice_mut::<u8, Instance>(&mut *mapped_data);
            callback(instances);
        }

        self.instance_staging_buffer.unmap();

        // Copy the data back
        let mut encoder = self
            .gpu_controller
            .create_command_encoder("Instance Update Copy Back");
        encoder.copy_buffer_to_buffer(
            &self.instance_staging_buffer,
            byte_range.start,
            &self.instance_buffer,
            byte_range.start,
            byte_range.end - byte_range.start,
        );
        self.gpu_controller.submit(encoder);

        Ok(())
    }
}
