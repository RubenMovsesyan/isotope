use std::sync::Arc;

use camera_3d::PerspectiveCamera3D;
use cgmath::{Matrix4, Point3, Vector3};
use gpu_controller::{
    BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType,
    GpuController, ShaderStages,
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

pub enum Camera {
    PerspectiveCamera3D(PerspectiveCamera3D),
}

impl Camera {
    pub fn new_perspective_3d<V: Into<[f32; 3]>>(
        gpu_controller: Arc<GpuController>,
        eye: V,
        target: V,
        up: V,
        aspect: f32,
        fovy: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self::PerspectiveCamera3D(PerspectiveCamera3D::new(
            gpu_controller,
            Point3::from(eye.into()),
            Vector3::from(target.into()),
            Vector3::from(up.into()),
            aspect,
            fovy,
            near,
            far,
        ))
    }

    pub fn bind_group(&self) -> &BindGroup {
        match self {
            Camera::PerspectiveCamera3D(camera) => &camera.bind_group,
        }
    }
}
