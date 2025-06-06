use log::*;
use std::{path::Path, sync::Arc};
use wgpu::{
    Buffer, BufferUsages,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    bind_group_builder,
    element::asset_manager::SharedAsset,
    gpu_utils::GpuController,
    photon::{
        render_descriptor::{PhotonRenderDescriptor, PhotonRenderDescriptorBuilder, STORAGE_RO},
        renderer::texture::PhotonTexture,
    },
};

use super::asset_manager::AssetManager;

// For sending color data to gpu
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
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

// Error color as a default
impl Default for MaterialColor {
    fn default() -> Self {
        Self {
            _padding: [0, 0],
            ambient_color: [1.0, 0.0, 1.0],
            diffuse_color: [1.0, 0.0, 1.0],
            specular_color: [1.0, 0.0, 1.0],
            specular_focus: 100.0,
            optical_density: 0.0,
            dissolve: 0.0,
            illum: 0,
            optional_texture: 0,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Material {
    Unbuffered {
        material_color: MaterialColor,
        diffuse_texture_label: String,
        label: String,
    },
    Buffered {
        label: String,
        diffuse_texture: SharedAsset<PhotonTexture>,

        // For the gpu
        color_buffer: Buffer,
        render_descriptor: Arc<PhotonRenderDescriptor>,
        gpu_controller: Arc<GpuController>,
    },
}

impl From<&obj_loader::material::Material> for Material {
    fn from(value: &obj_loader::material::Material) -> Self {
        let (optional_texture, texture_label) = if let Some(texture) = value.texture.as_ref() {
            (1, texture.to_owned())
        } else {
            (0, "".to_string())
        };

        Self::Unbuffered {
            label: value.label.to_owned(),
            diffuse_texture_label: texture_label,
            material_color: MaterialColor {
                _padding: [0, 0],
                ambient_color: value.ambient_color,
                diffuse_color: value.diffuse_color,
                specular_color: value.specular_color,
                specular_focus: value.specular_focus,
                optical_density: value.optical_density,
                dissolve: value.dissovle,
                illum: value.illum,
                optional_texture,
            },
        }
    }
}

impl From<obj_loader::material::Material> for Material {
    fn from(value: obj_loader::material::Material) -> Self {
        let (optional_texture, texture_label) = if let Some(texture) = value.texture.as_ref() {
            (1, texture.to_owned())
        } else {
            (0, "".to_string())
        };

        Self::Unbuffered {
            label: value.label.to_owned(),
            diffuse_texture_label: texture_label,
            material_color: MaterialColor {
                _padding: [0, 0],
                ambient_color: value.ambient_color,
                diffuse_color: value.diffuse_color,
                specular_color: value.specular_color,
                specular_focus: value.specular_focus,
                optical_density: value.optical_density,
                dissolve: value.dissovle,
                illum: value.illum,
                optional_texture,
            },
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::Unbuffered {
            label: "".to_owned(),
            diffuse_texture_label: "".to_owned(),
            material_color: MaterialColor::default(),
        }
    }
}

impl Material {
    pub fn with_label(label: String) -> Self {
        Self::Unbuffered {
            material_color: MaterialColor::default(),
            diffuse_texture_label: "".to_owned(),
            label,
        }
    }

    pub fn label(&self) -> String {
        match self {
            Material::Unbuffered { label, .. } => label.to_owned(),
            Material::Buffered { label, .. } => label.to_owned(),
        }
    }

    /// Consumes self and buffers all the data onto the gpu
    pub fn buffer<P>(self, directory: P, asset_manager: &mut AssetManager) -> Self
    where
        P: AsRef<Path>,
    {
        if let Material::Unbuffered {
            material_color,
            diffuse_texture_label,
            label,
        } = self
        {
            // For convenience
            let gpu_controller = asset_manager.gpu_controller.clone();

            let pathed_diffuse_texture_label = if let Some(directory) = directory.as_ref().to_str()
            {
                String::from(directory) + "/" + diffuse_texture_label.as_str()
            } else {
                "".to_string()
            };

            info!("Searching for texture: {}", pathed_diffuse_texture_label);

            let diffuse_texture = asset_manager.get_texture(&pathed_diffuse_texture_label);

            let color_buffer = gpu_controller
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    label: Some("Material Color Buffer"),
                    contents: bytemuck::cast_slice(&[material_color]),
                    usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                });

            let render_descriptor = PhotonRenderDescriptorBuilder::default()
                .add_render_chain(
                    diffuse_texture.with_read(|texture| texture.render_descriptor.clone()),
                )
                .with_label(&format!("Material: {}", label))
                .add_bind_group_with_layout(bind_group_builder!(
                    gpu_controller.device,
                    "Material",
                    (0, FRAGMENT, color_buffer.as_entire_binding(), STORAGE_RO)
                ))
                .build_module(gpu_controller.clone());

            Self::Buffered {
                label,
                color_buffer,
                diffuse_texture,
                render_descriptor: Arc::new(render_descriptor),
                gpu_controller,
            }
        } else {
            self
        }
    }

    // pub fn new_default(asset_manager: Arc<RwLock<AssetManager>>) -> Self {
    //     let gpu_controller = if let Ok(asset_manager) = asset_manager.read() {
    //         asset_manager.gpu_controller.clone()
    //     } else {
    //         unimplemented!();
    //     };

    //     let texture = PhotonTexture::new_empty(gpu_controller.clone());

    //     let color_buffer = gpu_controller.device.create_buffer(&BufferDescriptor {
    //         label: Some("Material Color Buffer"),
    //         mapped_at_creation: false,
    //         size: std::mem::size_of::<MaterialColor>() as u64,
    //         usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    //     });

    //     let render_descriptor = PhotonRenderDescriptorBuilder::default()
    //         .add_render_chain(texture.render_descriptor.clone())
    //         .with_label("Material")
    //         .add_bind_group_with_layout(bind_group_builder!(
    //             gpu_controller.device,
    //             "Material",
    //             (0, FRAGMENT, color_buffer.as_entire_binding(), STORAGE_RO)
    //         ))
    //         .build_module(gpu_controller);

    //     Self {
    //         name: String::from("Material Default"),
    //         diffuse_texture: SharedAsset::new(texture),
    //         ambient_color: ERROR_COLOR,
    //         diffuse_color: ERROR_COLOR,
    //         specular_color: ERROR_COLOR,
    //         dissolve: 0.0,
    //         illum: 0,
    //         optical_density: 0.0,
    //         specular_focus: 100.0,
    //         color_buffer,
    //         render_descriptor: Arc::new(render_descriptor),
    //     }
    // }

    // fn get_color(&self, texture: bool) -> MaterialColor {
    //     MaterialColor {
    //         ambient_color: [
    //             self.ambient_color.r as f32,
    //             self.ambient_color.g as f32,
    //             self.ambient_color.b as f32,
    //         ],
    //         diffuse_color: [
    //             self.diffuse_color.r as f32,
    //             self.diffuse_color.g as f32,
    //             self.diffuse_color.b as f32,
    //         ],
    //         specular_color: [
    //             self.specular_color.r as f32,
    //             self.specular_color.g as f32,
    //             self.specular_color.b as f32,
    //         ],
    //         specular_focus: self.specular_focus as f32,
    //         optical_density: self.optical_density as f32,
    //         dissolve: self.dissolve as f32,
    //         illum: self.illum as u32,
    //         optional_texture: if texture { 1 } else { 0 },
    //         _padding: [0, 0],
    //     }
    // }
}
