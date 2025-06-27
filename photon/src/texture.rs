use anyhow::Result;
use gpu_controller::GpuController;
use wgpu::{
    AddressMode, CompareFunction, Extent3d, FilterMode, Sampler, SamplerDescriptor, Texture,
    TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
    wgt::TextureDescriptor,
};

const PHOTON_DEPTH_TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth32Float;

#[derive(Debug)]
pub struct PhotonTexture {
    pub texture: Texture,
    pub view: TextureView,
    pub sampler: Sampler,
}

impl PhotonTexture {
    pub fn new_depth_texture(gpu_controller: &GpuController) -> Result<Self> {
        let size = gpu_controller.with_surface_config(|surface_config| Extent3d {
            width: surface_config.width.max(1),
            height: surface_config.height.max(1),
            depth_or_array_layers: 1,
        })?;

        let texture = gpu_controller.create_texture(&TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: PHOTON_DEPTH_TEXTURE_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = gpu_controller.create_sampler(&SamplerDescriptor {
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

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}
