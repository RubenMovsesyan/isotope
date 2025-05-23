use std::{path::Path, sync::Arc};

use anyhow::{Result, anyhow};
use log::*;
use wgpu::RenderPass;

use crate::{
    Isotope,
    element::{
        material::load_materials,
        model_vertex::{ModelVertex, VertexNormalVec, VertexPosition, VertexUvCoord},
    },
    utils::file_io::read_lines,
};

use super::{
    material::Material,
    mesh::{INDEX_FORMAT, Mesh},
};

pub use super::mesh::ModelInstance;

// Set to 5 to allow for future expansion of bind groups for the shader
pub(crate) const MODEL_BIND_GROUP: u32 = 2;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Model {
    meshes: Vec<Mesh>,
    materials: Vec<Arc<Material>>,
    instances: Vec<ModelInstance>,
}

impl Model {
    pub fn from_obj<P>(path: P, isotope: &Isotope) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        info!(
            "Loading Object: {:#?}",
            path.as_ref()
                .to_str()
                .ok_or(anyhow!("Object Path Not Valid"))?
        );

        let gpu_controller = isotope.gpu_controller.clone();
        let photon_layouts_manager = if let Some(photon) = isotope.photon.as_ref() {
            &photon.renderer.layouts
        } else {
            return Err(anyhow!("Photon not initialzed"));
        };

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
                        let mut new_mesh =
                            Mesh::new(name, &model_vertices, &indices, gpu_controller.clone());

                        if let Some(mat_ind) = material_index.take() {
                            new_mesh.material = Some(materials[mat_ind].clone());
                        }

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
                        &gpu_controller,
                        photon_layouts_manager,
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
            let mut new_mesh = Mesh::new(name, &model_vertices, &indices, gpu_controller.clone());

            if let Some(mat_ind) = material_index.take() {
                new_mesh.material = Some(materials[mat_ind].clone());
            }

            meshes.push(new_mesh);
        }

        Ok(Self {
            meshes,
            materials,
            instances: Vec::new(),
        })
    }

    // Functino to modify the instance rather than set new ones
    pub fn modify_instances<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut [ModelInstance]),
    {
        callback(&mut self.instances);

        // Set the instance buffer for all the meshes
        for mesh in self.meshes.iter_mut() {
            mesh.set_instance_buffer(&self.instances);
        }
    }

    pub fn set_instances(&mut self, instances: &[ModelInstance]) {
        // If the instance buffer is a different size update the instances to match the size
        // if the new buffer is lonver then clear the instances and start anew
        if instances.len() > self.instances.len() {
            if self.instances.capacity() < instances.len() {
                self.instances = Vec::with_capacity(instances.len());
            }

            for (index, instance) in instances.iter().enumerate() {
                if index < self.instances.len() {
                    self.instances[index] = *instance;
                } else {
                    self.instances.push(*instance);
                }
            }
        // if it is shorter then pop from the current buffer and push until finished
        } else if instances.len() <= self.instances.len() {
            while self.instances.len() > instances.len() {
                _ = self.instances.pop();
            }

            for (index, instance) in instances.iter().enumerate() {
                self.instances[index] = *instance;
            }
        }

        warn!("Self Instances: {:#?}", self.instances);

        // Set the instance buffer for all the meshes
        for mesh in self.meshes.iter_mut() {
            mesh.set_instance_buffer(&self.instances);
        }
    }

    pub fn render(&self, render_pass: &mut RenderPass) {
        for mesh in self.meshes.iter() {
            // Vertices
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            // Vertex Indices
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), INDEX_FORMAT);
            // Instances
            render_pass.set_vertex_buffer(1, mesh.instance_buffer.slice(..));

            // Set the bind group for the material of the mesh
            // TODO: add optional texture support
            render_pass.set_bind_group(
                MODEL_BIND_GROUP,
                &mesh
                    .material
                    .as_ref()
                    .unwrap()
                    .diffuse_texture
                    .as_ref()
                    .unwrap()
                    .bind_group,
                &[],
            );

            // TODO: Add Gpu Instancing to the Mesh
            render_pass.draw_indexed(0..mesh.num_indices, 0, 0..mesh.instance_buffer_len);
        }
    }
}
