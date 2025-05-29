use log::*;
use std::{path::Path, sync::Arc};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor, BufferUsages, Color,
};

use anyhow::{Result, anyhow};

use crate::{
    GpuController,
    photon::renderer::{photon_layouts::PhotonLayoutsManager, texture::PhotonTexture},
    utils::file_io::read_lines,
};

// Default color to check for errors
const ERROR_COLOR: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};

// For sending color data to gpu
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialColor {
    _padding: [u32; 2], // IMPORTANT! make sure padding is at the beginning because of alignment
    pub ambient_color: [f32; 3],
    pub diffuse_color: [f32; 3],
    pub specular_color: [f32; 3],
    pub specular_focus: f32,
    pub optical_density: f32,
    pub dissolve: f32,
    pub illum: u32,
    pub optional_texture: u32, // 0 for false, 1 for true
}

#[derive(Debug)]
pub struct Material {
    pub name: String,
    pub diffuse_texture: PhotonTexture,
    pub ambient_color: Color,
    pub diffuse_color: Color,
    pub specular_color: Color,
    pub specular_focus: f64,
    pub optical_density: f64,
    pub dissolve: f64,
    pub illum: u64,

    // For the gpu
    pub color_buffer: Buffer,
    pub bind_group: BindGroup,
}

impl Material {
    fn get_color(&self, texture: bool) -> MaterialColor {
        MaterialColor {
            ambient_color: [
                self.ambient_color.r as f32,
                self.ambient_color.g as f32,
                self.ambient_color.b as f32,
            ],
            diffuse_color: [
                self.diffuse_color.r as f32,
                self.diffuse_color.g as f32,
                self.diffuse_color.b as f32,
            ],
            specular_color: [
                self.specular_color.r as f32,
                self.specular_color.g as f32,
                self.specular_color.b as f32,
            ],
            specular_focus: self.specular_focus as f32,
            optical_density: self.optical_density as f32,
            dissolve: self.dissolve as f32,
            illum: self.illum as u32,
            optional_texture: if texture { 1 } else { 0 },
            _padding: [0, 0],
        }
    }
}

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
    let mut texture = false;

    for line in lines.map_while(Result::ok) {
        let line_split = line.split_whitespace().collect::<Vec<_>>();

        if line_split.is_empty() {
            continue;
        }

        match line_split[0] {
            "newmtl" => {
                // If there is a material currently, add it to the materials list
                if let Some(material) = current_material.take() {
                    // Write the color information to the material buffer first
                    // debug!("Material Data: {:#?}", material.get_color(texture));
                    gpu_controller.queue.write_buffer(
                        &material.color_buffer,
                        0,
                        bytemuck::cast_slice(&[material.get_color(texture)]),
                    );
                    materials.push(Arc::new(material));
                    texture = false;
                }

                let color_buffer = gpu_controller.device.create_buffer(&BufferDescriptor {
                    label: Some("Material Color Buffer"),
                    mapped_at_creation: false,
                    size: std::mem::size_of::<MaterialColor>() as u64,
                    usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                });

                current_material = Some(Material {
                    name: line_split[1].to_string(),
                    diffuse_texture: PhotonTexture::new_empty(
                        gpu_controller,
                        photon_layouts_manager,
                    )?,
                    ambient_color: ERROR_COLOR,
                    diffuse_color: ERROR_COLOR,
                    specular_color: ERROR_COLOR,
                    specular_focus: 100.0,
                    optical_density: 1.0,
                    dissolve: 1.0,
                    illum: 0,
                    bind_group: gpu_controller
                        .device
                        .create_bind_group(&BindGroupDescriptor {
                            label: Some("Photon Material Bind Group"),
                            layout: &photon_layouts_manager.material_layout,
                            entries: &[BindGroupEntry {
                                binding: 0,
                                resource: color_buffer.as_entire_binding(),
                            }],
                        }),
                    color_buffer,
                });

                debug!("Found New Material: {}", line_split[1]);
            }
            "Ka" => {
                let (r, g, b) = (
                    line_split[1].to_string().parse::<f64>()?,
                    line_split[2].to_string().parse::<f64>()?,
                    line_split[3].to_string().parse::<f64>()?,
                );

                if let Some(material) = current_material.as_mut() {
                    material.ambient_color = Color { r, g, b, a: 1.0 };
                }

                debug!("Found Ambient Color: {}, {}, {}", r, g, b);
            }
            "Kd" => {
                let (r, g, b) = (
                    line_split[1].to_string().parse::<f64>()?,
                    line_split[2].to_string().parse::<f64>()?,
                    line_split[3].to_string().parse::<f64>()?,
                );

                if let Some(material) = current_material.as_mut() {
                    material.diffuse_color = Color { r, g, b, a: 1.0 };
                }

                debug!("Found Diffuse Color: {}, {}, {}", r, g, b);
            }
            "Ks" => {
                let (r, g, b) = (
                    line_split[1].to_string().parse::<f64>()?,
                    line_split[2].to_string().parse::<f64>()?,
                    line_split[3].to_string().parse::<f64>()?,
                );

                if let Some(material) = current_material.as_mut() {
                    material.specular_color = Color { r, g, b, a: 1.0 };
                }

                debug!("Found Specular Color: {}, {}, {}", r, g, b);
            }
            "Ns" => {
                let specular_focus = line_split[1].to_string().parse::<f64>()?;

                if let Some(material) = current_material.as_mut() {
                    material.specular_focus = specular_focus;
                }

                debug!("Found Specular Focus: {}", specular_focus);
            }
            "Ni" => {
                let optical_density = line_split[1].to_string().parse::<f64>()?;

                if let Some(material) = current_material.as_mut() {
                    material.optical_density = optical_density;
                }

                debug!("Found Optical Density: {}", optical_density);
            }
            "d" => {
                let dissolve = line_split[1].to_string().parse::<f64>()?;

                if let Some(material) = current_material.as_mut() {
                    material.dissolve = dissolve;
                }

                debug!("Found dissolve: {}", dissolve);
            }
            "illum" => {
                let illum = line_split[1].to_string().parse::<u64>()?;

                if let Some(material) = current_material.as_mut() {
                    material.illum = illum;
                }

                debug!("Found Illumination: {}", illum);
            }
            "map_Kd" => {
                texture = true;
                if let Some(material) = current_material.as_mut() {
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

                    material.diffuse_texture = diffuse_texture;
                } else {
                    return Err(anyhow!("Material File Corrupt"));
                }
            }
            _ => {}
        }
    }

    if let Some(material) = current_material.take() {
        // Write the color information to the material buffer first
        debug!("Material Data: {:#?}", material.get_color(texture));
        gpu_controller.queue.write_buffer(
            &material.color_buffer,
            0,
            bytemuck::cast_slice(&[material.get_color(texture)]),
        );
        materials.push(Arc::new(material));
    }

    Ok(materials)
}
