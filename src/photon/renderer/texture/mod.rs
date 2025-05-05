use std::path::Path;

use anyhow::{Result, anyhow};
use image::{GenericImageView, ImageReader};
use log::*;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, CompareFunction,
    Extent3d, FilterMode, Sampler, SamplerDescriptor, SurfaceConfiguration, TexelCopyBufferLayout,
    Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

use crate::GpuController;

use super::photon_layouts::PhotonLayoutsManager;

const ROW_SIZE: u32 = std::mem::size_of::<f32>() as u32;

pub(crate) const PHOTON_TEXTURE_DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

// Common functionality between all textures
pub(crate) trait View {
    fn view(&self) -> &TextureView;
}

#[derive(Debug)]
pub(crate) struct PhotonTexture {
    texture: Texture,
    view: TextureView,
    sampler: Sampler,
    pub bind_group: BindGroup,
}

impl PhotonTexture {
    pub fn new_from_path<P>(
        gpu_controller: &GpuController,
        path: P,
        photon_layouts: &PhotonLayoutsManager,
    ) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        info!("Loading Texture: {:#?}", path.as_ref());

        // Load the image from path
        let img = ImageReader::open(path.as_ref())?.decode()?;
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        debug!("Texture Size: {:#?}", size);

        // Create the wgpu texture
        let texture = gpu_controller.device.create_texture(&TextureDescriptor {
            label: Some(&format!(
                "Photon Texture: {}",
                path.as_ref()
                    .to_str()
                    .ok_or(anyhow!("Texture Path not available"))?
            )),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        // TODO: Add Support for changing the texture filter modes
        let sampler = gpu_controller.device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = gpu_controller
            .device
            .create_bind_group(&BindGroupDescriptor {
                label: Some(&format!(
                    "Photon Texture Bind Group: {}",
                    path.as_ref()
                        .to_str()
                        .ok_or(anyhow!("Texture Path not available"))?,
                )),
                layout: &photon_layouts.texture_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&sampler),
                    },
                ],
            });

        // Send the texture to the gpu
        gpu_controller.queue.write_texture(
            texture.as_image_copy(),
            &rgba,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(ROW_SIZE * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        Ok(Self {
            texture,
            view,
            sampler,
            bind_group,
        })
    }
}

impl View for PhotonTexture {
    fn view(&self) -> &TextureView {
        &self.view
    }
}

#[derive(Debug)]
pub struct PhotonDepthTexture {
    texture: Texture,
    view: TextureView,
    sampler: Sampler,
}

impl PhotonDepthTexture {
    pub fn new_depth_texture(
        gpu_controller: &GpuController,
        surface_configuration: &SurfaceConfiguration,
    ) -> Self {
        let size = Extent3d {
            width: surface_configuration.width.max(1),
            height: surface_configuration.height.max(1),
            depth_or_array_layers: 1,
        };

        let texture = gpu_controller.device.create_texture(&TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: PHOTON_TEXTURE_DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = gpu_controller.device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            compare: Some(CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}

impl View for PhotonDepthTexture {
    fn view(&self) -> &TextureView {
        &self.view
    }
}
