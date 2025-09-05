use std::{collections::HashMap, sync::Arc};

use gpu_controller::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, GpuController, SamplerBindingType, ShaderStages, TextureSampleType,
    TextureViewDimension,
};
use matter_vault::MatterVault;

pub struct AssetServer {
    pub(crate) asset_manager: Arc<MatterVault>,
    pub(crate) gpu_controller: Arc<GpuController>,
    pub(crate) layouts: HashMap<String, BindGroupLayout>,
}

impl AssetServer {
    pub fn new(asset_manager: Arc<MatterVault>, gpu_controller: Arc<GpuController>) -> Self {
        let mut layouts = HashMap::new();

        layouts.insert(
            "Material".to_string(),
            gpu_controller.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Material Bind Group Layout"),
                entries: &[
                    // Material Properties
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Texture
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Sampler
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            }),
        );

        Self {
            asset_manager,
            gpu_controller,
            layouts,
        }
    }
}
