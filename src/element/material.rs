use log::*;
use std::{path::Path, sync::Arc};

use anyhow::{Result, anyhow};

use crate::{
    GpuController,
    photon::renderer::{photon_layouts::PhotonLayoutsManager, texture::PhotonTexture},
    utils::file_io::read_lines,
};

#[derive(Debug)]
pub struct Material {
    pub name: String,
    pub diffuse_texture: Option<PhotonTexture>,
}

impl Material {}

pub fn load_materials<P>(
    gpu_controller: &GpuController,
    photon_layouts_manager: &PhotonLayoutsManager,
    path: P,
) -> Result<Vec<Arc<Material>>>
where
    P: AsRef<Path>,
{
    info!(
        "Loading Material: {}",
        path.as_ref()
            .to_str()
            .ok_or(anyhow!("Material Path not available"))?
    );

    let lines = read_lines(path.as_ref())?;

    let mut materials: Vec<Arc<Material>> = Vec::new();
    let mut current_material: Option<Material> = None;

    for line in lines.map_while(Result::ok) {
        let line_split = line.split_whitespace().collect::<Vec<_>>();

        if line_split.is_empty() {
            continue;
        }

        match line_split[0] {
            "newmtl" => {
                // If there is a material currently, add it to the materials list
                if current_material.is_some() {
                    unsafe {
                        materials.push(Arc::new(current_material.take().unwrap_unchecked()));
                    }
                }

                current_material = Some(Material {
                    name: line_split[1].to_string(),
                    diffuse_texture: None,
                });
            }
            "map_Kd" => {
                if current_material.is_some() {
                    let diffuse_texture_path = path
                        .as_ref()
                        .parent()
                        .ok_or(anyhow!("Diffuse Texture Path Invalid"))?
                        .join(line_split[1]);

                    debug!("Texture Path: {:#?}", diffuse_texture_path);

                    let diffuse_texture = PhotonTexture::new_from_path(
                        gpu_controller,
                        diffuse_texture_path,
                        photon_layouts_manager,
                    )?;

                    unsafe {
                        current_material.as_mut().unwrap_unchecked().diffuse_texture =
                            Some(diffuse_texture);
                    }
                } else {
                    // If there is not material at this point that means that the
                    // file is corrupt
                    return Err(anyhow!("Material File Corrupt"));
                }
            }
            _ => {}
        }
    }

    Ok(materials)
}
