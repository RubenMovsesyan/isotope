use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::Result;
use gpu_controller::{Mesh, Vertex};
use log::{debug, info};
use matter_vault::SharedMatter;
use wgpu::RenderPass;

use crate::asset_server::AssetServer;

type Position = [f32; 3];
type Normal = [f32; 3];
type UV = [f32; 2];

pub struct Model {
    meshes: Vec<SharedMatter<Mesh>>,
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
        let mut label_exists = false;

        // For mesh building
        let mut positions: Vec<Position> = Vec::new();
        let mut normals: Vec<Normal> = Vec::new();
        let mut uvs: Vec<UV> = Vec::new();

        // For the end return
        let mut meshes: Vec<SharedMatter<Mesh>> = Vec::new();

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
                            meshes.push(asset_server.asset_manager.add(mesh_label, mesh)?);
                        }

                        let label = tokens[1].to_string();

                        // If the mesh is already shared, add it to the list of meshes
                        if let Ok(mesh) = asset_server.asset_manager.share(&label) {
                            debug!("Mesh already exists: {}", label);
                            meshes.push(mesh);
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
                    // TODO: Add materials
                    _ => {}
                }
            } else {
                match tokens[0] {
                    "o" => {
                        label_exists = false;
                    }
                    _ => {}
                }
            }
        }

        if let Some(mut mesh) = current_mesh.take() {
            mesh.buffer(asset_server.gpu_controller.clone());
            let mesh_label = mesh.label().clone();
            meshes.push(asset_server.asset_manager.add(mesh_label, mesh)?);
        }

        Ok(Self { meshes })
    }

    pub fn render(&self, render_pass: &mut RenderPass) {
        for mesh in self.meshes.iter() {
            mesh.read(|mesh| {
                mesh.render(render_pass);
            });
        }
    }
}
