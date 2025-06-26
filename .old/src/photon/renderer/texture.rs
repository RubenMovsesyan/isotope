use std::{path::Path, sync::Arc};

use anyhow::{Result, anyhow};
use image::ImageReader;
use log::*;
use wgpu::{
    AddressMode, BindingResource, CompareFunction, Extent3d, FilterMode, Sampler,
    SamplerDescriptor, TexelCopyBufferLayout, Texture, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};
use winit::dpi::PhysicalSize;

use crate::{
    GpuController, bind_group_builder,
    photon::render_descriptor::{
        PhotonRenderDescriptor, PhotonRenderDescriptorBuilder, SAMPLER, TEXTURE,
    },
};

const ROW_SIZE: u32 = std::mem::size_of::<f32>() as u32;

pub(crate) const PHOTON_TEXTURE_DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

// Common functionality between all textures
pub(crate) trait View {
    fn view(&self) -> &TextureView;
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct PhotonTexture {
    pub(crate) texture: Texture,
    pub(crate) view: TextureView,
    pub(crate) sampler: Sampler,

    pub(crate) updated: bool,

    // For rendering
    pub(crate) render_descriptor: Arc<PhotonRenderDescriptor>,
}

impl PhotonTexture {
    pub fn new_empty(gpu_controller: Arc<GpuController>) -> Self {
        info!("Creating Empty Texture");

        let size = Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };

        let texture = gpu_controller.device.create_texture(&TextureDescriptor {
            label: Some("Photon Texture Empty"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = gpu_controller.device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..Default::default()
        });

        let render_descriptor = PhotonRenderDescriptorBuilder::default()
            .with_label("Texture")
            .add_bind_group_with_layout(bind_group_builder!(
                gpu_controller.device,
                "Texture",
                (0, FRAGMENT, BindingResource::TextureView(&view), TEXTURE),
                (1, FRAGMENT, BindingResource::Sampler(&sampler), SAMPLER)
            ))
            .build_module(gpu_controller);

        Self {
            texture,
            view,
            sampler,
            updated: false,
            render_descriptor: Arc::new(render_descriptor), // bind_group,
        }
    }

    pub fn new_from_path<P>(gpu_controller: Arc<GpuController>, path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        info!("Loading Texture: {:#?}", path.as_ref());

        // Load the image from path
        // let img = ImageReader::open(path.as_ref())?.decode()?;
        // let rgba = img.to_rgba8();
        let img_reader = ImageReader::open(path.as_ref())?;
        let dimensions = img_reader.into_dimensions()?;

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
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..Default::default()
        });

        let render_descriptor = PhotonRenderDescriptorBuilder::default()
            .with_label("Texture")
            .add_bind_group_with_layout(bind_group_builder!(
                gpu_controller.device,
                "Texture",
                (0, FRAGMENT, BindingResource::TextureView(&view), TEXTURE),
                (1, FRAGMENT, BindingResource::Sampler(&sampler), SAMPLER)
            ))
            .build_module(gpu_controller.clone());

        // Create a thread to read the image and then write it to the texture when it is done
        if let Some(path) = path.as_ref().to_str() {
            let texture_clone = texture.clone();
            let path_clone = path.to_string();
            std::thread::spawn(move || {
                if let Ok(img) = ImageReader::open(&path_clone) {
                    if let Ok(img) = img.decode() {
                        let rgba = img.to_rgba8();

                        info!("Texture Loaded Writing to buffer");
                        gpu_controller.queue.write_texture(
                            texture_clone.as_image_copy(),
                            &rgba,
                            TexelCopyBufferLayout {
                                offset: 0,
                                bytes_per_row: Some(ROW_SIZE * dimensions.0),
                                rows_per_image: Some(dimensions.1),
                            },
                            size,
                        );
                    } else {
                        error!("Error Reading Image");
                        return;
                    }
                }
            });
        }

        Ok(Self {
            texture,
            view,
            sampler,
            updated: false,
            render_descriptor: Arc::new(render_descriptor),
        })
    }
}

impl View for PhotonTexture {
    fn view(&self) -> &TextureView {
        &self.view
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct PhotonDepthTexture {
    texture: Texture,
    view: TextureView,
    sampler: Sampler,
}

impl PhotonDepthTexture {
    pub fn new_depth_texture(gpu_controller: &GpuController) -> Self {
        let surface_configuration = gpu_controller.surface_configuration();
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
            mipmap_filter: FilterMode::Linear,
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

    pub fn new_depth_texture_from_size(
        gpu_controller: &GpuController,
        size: PhysicalSize<u32>,
    ) -> Self {
        let size = Extent3d {
            width: size.width.max(1),
            height: size.height.max(1),
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
            mipmap_filter: FilterMode::Linear,
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
