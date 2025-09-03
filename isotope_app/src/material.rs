use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    sync::Arc,
};

use anyhow::{Result, anyhow};
use gpu_controller::GpuController;
use log::{debug, error, info};
use matter_vault::SharedMatter;

use crate::{asset_server::AssetServer, texture::IsotopeTexture};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MaterialProperties {
    _padding: [u32; 2],
    ambient_color: [f32; 3],
    diffuse_color: [f32; 3],
    specular_color: [f32; 3],
    specular_focus: f32,
    optical_density: f32,
    dissolve: f32,
    illum: u32,
}

// ERROR color as default
impl Default for MaterialProperties {
    fn default() -> Self {
        Self {
            _padding: [0; 2],
            ambient_color: [1.0, 0.0, 1.0],
            diffuse_color: [1.0, 0.0, 1.0],
            specular_color: [1.0, 0.0, 1.0],
            specular_focus: 100.0,
            optical_density: 0.0,
            dissolve: 0.0,
            illum: 0,
        }
    }
}

pub struct Material {
    properties: MaterialProperties,
    label: String,

    gpu_controller: Arc<GpuController>,
    texture: Option<SharedMatter<IsotopeTexture>>,
}

pub fn load_materials<P>(path: P, asset_server: &AssetServer) -> Result<Vec<SharedMatter<Material>>>
where
    P: AsRef<Path>,
{
    info!("Loading Materials From Path: {:#?}", path.as_ref());

    let file = File::open(path.as_ref())?;

    let mut materials: Vec<SharedMatter<Material>> = Vec::new();
    let mut current_material: Option<Material> = None;
    let mut label_exists = false;

    for line in BufReader::new(file).lines().map_while(Result::ok) {
        let tokens = line.split_whitespace().collect::<Vec<_>>();

        if tokens.is_empty() {
            continue;
        }

        if !label_exists {
            match tokens[0] {
                "newmtl" => {
                    // If there is a material already in the queue then push it to the list of materials
                    if let Some(material) = current_material.take() {
                        let material_label = material.label.clone();
                        materials.push(asset_server.asset_manager.add(material_label, material)?);
                    }

                    let label = tokens[1].to_string();

                    // If the material is already shared, add it to the list of materals
                    if let Ok(material) = asset_server.asset_manager.share(&label) {
                        debug!("Material already exists: {}", label);
                        materials.push(material);
                        label_exists = true;
                    } else {
                        debug!("Creating new material: {}", label);
                        current_material = Some(Material {
                            gpu_controller: asset_server.gpu_controller.clone(),
                            label: label.clone(),
                            properties: MaterialProperties::default(),
                            texture: None,
                        })
                    }
                }
                // Specular Focus
                "Ns" => {
                    current_material
                        .as_mut()
                        .ok_or_else(|| anyhow!("No Current Material"))?
                        .properties
                        .specular_focus = tokens[1].parse::<f32>()?;
                }
                // Ambient Color
                "Ka" => {
                    current_material
                        .as_mut()
                        .ok_or_else(|| anyhow!("No Current Material"))?
                        .properties
                        .ambient_color = [
                        tokens[1].parse::<f32>()?,
                        tokens[2].parse::<f32>()?,
                        tokens[3].parse::<f32>()?,
                    ];
                }
                // Diffuse Color
                "Kd" => {
                    current_material
                        .as_mut()
                        .ok_or_else(|| anyhow!("No Current Material"))?
                        .properties
                        .diffuse_color = [
                        tokens[1].parse::<f32>()?,
                        tokens[2].parse::<f32>()?,
                        tokens[3].parse::<f32>()?,
                    ];
                }
                // Specular Color
                "Ks" => {
                    current_material
                        .as_mut()
                        .ok_or_else(|| anyhow!("No Current Material"))?
                        .properties
                        .specular_color = [
                        tokens[1].parse::<f32>()?,
                        tokens[2].parse::<f32>()?,
                        tokens[3].parse::<f32>()?,
                    ];
                }
                // Optical Density
                "Ni" => {
                    current_material
                        .as_mut()
                        .ok_or_else(|| anyhow!("No Current Material"))?
                        .properties
                        .optical_density = tokens[1].parse::<f32>()?;
                }
                // Dissolve
                "d" => {
                    current_material
                        .as_mut()
                        .ok_or_else(|| anyhow!("No Current Material"))?
                        .properties
                        .dissolve = tokens[1].parse::<f32>()?;
                }
                // Illumination
                "illum" => {
                    current_material
                        .as_mut()
                        .ok_or_else(|| anyhow!("No Current Material"))?
                        .properties
                        .illum = tokens[1].parse::<u32>()?;
                }
                // Texture
                "map_Kd" => {
                    let material_path = path
                        .as_ref()
                        .parent()
                        .ok_or_else(|| {
                            error!("Failed to get parent path");
                            anyhow!("Failed to get parent path")
                        })?
                        .join(tokens[1]);

                    debug!("Searching in path: {:#?}", material_path);

                    current_material
                        .as_mut()
                        .ok_or_else(|| anyhow!("No Current Material"))?
                        .texture = Some(
                        asset_server
                            .asset_manager
                            .share(tokens[1].to_string())
                            .or_else(|_err| {
                                asset_server.asset_manager.add(
                                    tokens[1].to_string(),
                                    IsotopeTexture::new_from_path(material_path, asset_server)
                                        .map_err(|err| {
                                            error!("Failed to Load Texture: {:#?}", err);
                                            err
                                        })?,
                                )
                            })?,
                    )
                }
                _ => {}
            }
        } else {
            match tokens[0] {
                "newmtl" => {
                    label_exists = false;

                    let label = tokens[1].to_string();

                    // If the material is already shared, add it to the list of materals
                    if let Ok(material) = asset_server.asset_manager.share(&label) {
                        debug!("Material already exists: {}", label);
                        materials.push(material);
                        label_exists = true;
                    } else {
                        debug!("Creating new material: {}", label);
                        current_material = Some(Material {
                            gpu_controller: asset_server.gpu_controller.clone(),
                            label: label.clone(),
                            properties: MaterialProperties::default(),
                            texture: None,
                        })
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(material) = current_material.take() {
        let material_label = material.label.clone();
        materials.push(asset_server.asset_manager.add(material_label, material)?);
    }

    Ok(materials)
}
