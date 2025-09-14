use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{Result, anyhow};
use gpu_controller::{Mesh, RenderPass, Vertex};
use log::{debug, info};
use matter_vault::SharedMatter;
use photon::MATERIALS_BIND_GROUP;

use crate::{
    asset_server::AssetServer,
    material::{Material, load_materials},
};

type Position = [f32; 3];
type Normal = [f32; 3];
type UV = [f32; 2];

pub struct Model {
    meshes: Vec<(Option<usize>, SharedMatter<Mesh>)>,
    materials: Vec<SharedMatter<Material>>,
}

impl Model {
    pub fn from_obj<P>(path: P, asset_server: &AssetServer) -> Result<Self>
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

        Ok(Self { meshes, materials })
    }

    pub fn render(&self, render_pass: &mut RenderPass) {
        for (material_index, mesh) in self.meshes.iter() {
            mesh.read(|mesh| {
                if let Some(material_index) = material_index.as_ref() {
                    self.materials[*material_index].read(|material| {
                        render_pass.set_bind_group(MATERIALS_BIND_GROUP, &material.bind_group, &[]);
                    });
                }

                mesh.render(render_pass);
            });
        }
    }
}
