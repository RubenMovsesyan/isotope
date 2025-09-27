use std::{collections::HashMap, sync::Arc};

use gpu_controller::{
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, GpuController,
    SamplerBindingType, ShaderStages, TextureSampleType, TextureViewDimension,
};
use matter_vault::MatterVault;
use photon::renderer::defered_renderer::{
    ALBEDO_BINDING, MATERIAL_BINDING, NORMAL_BINDING, POSITION_BINDING, SAMPLER_BINDING,
};

unsafe impl Send for AssetServer {}
unsafe impl Sync for AssetServer {}

pub struct AssetServer {
    pub(crate) asset_manager: Arc<MatterVault>,
    pub(crate) gpu_controller: Arc<GpuController>,
}

impl AssetServer {
    pub fn new(asset_manager: Arc<MatterVault>, gpu_controller: Arc<GpuController>) -> Self {
        // let mut layouts = HashMap::new();
        gpu_controller.write_layouts(|layouts| {
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

            layouts.insert(
                "Global Transform".to_string(),
                gpu_controller.create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Global Transform Bind Group Layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                }),
            );

            layouts.insert(
                "G-Buffer".to_string(),
                gpu_controller.create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("G-Buffer Bind Group Layout"),
                    entries: &[
                        // Albedo
                        BindGroupLayoutEntry {
                            binding: ALBEDO_BINDING,
                            count: None,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                multisampled: false,
                                sample_type: TextureSampleType::Float { filterable: true },
                                view_dimension: TextureViewDimension::D2,
                            },
                        },
                        BindGroupLayoutEntry {
                            binding: POSITION_BINDING,
                            count: None,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                multisampled: false,
                                sample_type: TextureSampleType::Float { filterable: true },
                                view_dimension: TextureViewDimension::D2,
                            },
                        },
                        // Normal
                        BindGroupLayoutEntry {
                            binding: NORMAL_BINDING,
                            count: None,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                multisampled: false,
                                sample_type: TextureSampleType::Float { filterable: true },
                                view_dimension: TextureViewDimension::D2,
                            },
                        },
                        // Material
                        BindGroupLayoutEntry {
                            binding: MATERIAL_BINDING,
                            count: None,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                multisampled: false,
                                sample_type: TextureSampleType::Float { filterable: true },
                                view_dimension: TextureViewDimension::D2,
                            },
                        },
                        // Sampler
                        BindGroupLayoutEntry {
                            binding: SAMPLER_BINDING,
                            count: None,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        },
                    ],
                }),
            );

            layouts.insert(
                "Lights".to_string(),
                gpu_controller.create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Lights Bind Group Layout"),
                    entries: &[
                        // Array of Lights
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // Number of Lights
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }),
            );

            layouts.insert(
                "Camera".to_string(),
                gpu_controller.create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Camera Bind Group Layout Descriptor"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                }),
            );
        });

        Self {
            asset_manager,
            gpu_controller,
        }
    }
}
