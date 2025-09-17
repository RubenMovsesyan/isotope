pub use camera_3d::PerspectiveCamera3D;
use cgmath::Matrix4;
use gpu_controller::{
    BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType,
    ShaderStages,
};

mod camera_3d;

pub const OPENGL_TO_WGPU_MATIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
);

pub const CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR: BindGroupLayoutDescriptor =
    BindGroupLayoutDescriptor {
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
    };

pub trait PhotonCamera {
    fn bind_group(&self) -> &BindGroup;
}
