use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, Device, ShaderStages,
};

const BIND_GROUP_LAYOUT_DESCRIPTOR: BindGroupLayoutDescriptor = BindGroupLayoutDescriptor {
    label: Some("Transform Bind Group Layout"),
    entries: &[BindGroupLayoutEntry {
        // Transform Buffer
        binding: 0,
        visibility: ShaderStages::VERTEX,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }],
};

pub fn create_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BIND_GROUP_LAYOUT_DESCRIPTOR)
}
