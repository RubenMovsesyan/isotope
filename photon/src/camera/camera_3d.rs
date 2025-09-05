use std::sync::Arc;

use cgmath::{Deg, Matrix4, Point3, Vector3, perspective};
use gpu_controller::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferInitDescriptor,
    BufferUsages, GpuController, ShaderStages,
};

use super::{CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR, OPENGL_TO_WGPU_MATIX};

#[repr(C)]
#[derive(Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PerspectiveCam3DUniform {
    view_position: [f32; 4],
    view_projection: [[f32; 4]; 4],
}

pub struct PerspectiveCamera3D {
    eye: Point3<f32>,
    target: Vector3<f32>,
    up: Vector3<f32>,

    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,

    camera_uniform: PerspectiveCam3DUniform,
    buffer: Buffer,
    gpu_controller: Arc<GpuController>,
    pub(crate) bind_group: BindGroup,
}

impl PerspectiveCamera3D {
    pub fn new(
        gpu_controller: Arc<GpuController>,
        eye: Point3<f32>,
        target: Vector3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        // let view = Matrix4::look_at_rh(eye, eye + target, up);
        let view = Matrix4::look_to_rh(eye, target, up);
        let proj = perspective(Deg(fovy), aspect, znear, zfar);

        let view_proj = OPENGL_TO_WGPU_MATIX * proj * view;

        let camera_uniform = PerspectiveCam3DUniform {
            view_position: eye.to_homogeneous().into(),
            view_projection: view_proj.into(),
        };

        let buffer = gpu_controller.create_buffer_init(&BufferInitDescriptor {
            label: Some("Perspective 3D Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group_layout =
            gpu_controller.create_bind_group_layout(&CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR);

        let bind_group = gpu_controller.create_bind_group(&BindGroupDescriptor {
            label: Some("Perspective 3D Camera Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
            camera_uniform,
            buffer,
            gpu_controller,
            bind_group,
        }
    }
}
