use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Device,
    SamplerBindingType, ShaderStages, TextureSampleType, TextureViewDimension,
};

pub const BIND_GROUP_LAYOUT_DESCRIPTOR: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("Photon Texture Bind Group Layout"),
    entries: &[
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                multisampled: false,
                view_dimension: TextureViewDimension::D2,
                sample_type: TextureSampleType::Float { filterable: true },
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        },
    ],
};

pub fn create_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BIND_GROUP_LAYOUT_DESCRIPTOR)
}
