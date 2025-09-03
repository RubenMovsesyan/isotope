use std::path::Path;

use anyhow::{Result, anyhow};
use gpu_controller::{
    AddressMode, Extent3d, FilterMode, Sampler, SamplerDescriptor, TexelCopyBufferLayout, Texture,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};
use image::ImageReader;
use log::{debug, error, info};

use crate::asset_server::AssetServer;

const ROW_SIZE: u32 = std::mem::size_of::<f32>() as u32;

pub(crate) struct IsotopeTexture {
    texture: Texture,
    view: TextureView,
    sampler: Sampler,
}

impl IsotopeTexture {
    pub fn new_empty(asset_server: &AssetServer) -> Self {
        info!("Creating Empty Texture");

        let size = Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };

        let texture = asset_server
            .gpu_controller
            .create_texture(&TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

        let view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = asset_server
            .gpu_controller
            .create_sampler(&SamplerDescriptor {
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                mipmap_filter: FilterMode::Linear,
                ..Default::default()
            });

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn new_from_path<P>(path: P, asset_server: &AssetServer) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        info!("Loading Texture: {:#?}", path.as_ref());

        let img_reader = ImageReader::open(path.as_ref())?;
        let dimensions = img_reader.into_dimensions()?;

        let size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        debug!("Texture Size: {:#?}", size);

        // Create the wgpu texture
        let texture = asset_server
            .gpu_controller
            .create_texture(&TextureDescriptor {
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
        let sampler = asset_server
            .gpu_controller
            .create_sampler(&SamplerDescriptor {
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                mipmap_filter: FilterMode::Linear,
                ..Default::default()
            });

        if let Some(path) = path.as_ref().to_str() {
            let texture_clone = texture.clone();
            let path_clone = path.to_string();
            let gpu_controller_clone = asset_server.gpu_controller.clone();
            std::thread::spawn(move || {
                ImageReader::open(&path_clone)
                    .and_then(|img| {
                        img.decode()
                            .and_then(|img| {
                                let rgba = img.to_rgba8();

                                info!("Texture Loaded Writing to buffer");
                                gpu_controller_clone.write_texture(
                                    &texture_clone,
                                    &rgba,
                                    TexelCopyBufferLayout {
                                        offset: 0,
                                        bytes_per_row: Some(ROW_SIZE * dimensions.0),
                                        rows_per_image: Some(dimensions.1),
                                    },
                                    size,
                                );

                                Ok(())
                            })
                            .map_err(|err| {
                                return anyhow!(err);
                            });

                        Ok(())
                    })
                    .map_err(|err| {
                        error!("Error loading texture: {}", err);
                        anyhow!(err)
                    });
            });
        }

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}
