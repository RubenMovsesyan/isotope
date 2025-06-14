use std::sync::Arc;

use cgmath::{
    Deg, EuclideanSpace, InnerSpace, Matrix4, Point3, Quaternion, Rotation, SquareMatrix, Vector3,
    perspective,
};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferUsages,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{GpuController, Transform};

pub(crate) type Vector4 = [f32; 4];
pub(crate) type Matrix4x4 = [[f32; 4]; 4];

pub const OPENGL_TO_WGPU_MATIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0, 1.0,
);

// Clamping constants
const FOVY_CLAMP: (f32, f32) = (0.1, 179.9);

#[derive(Debug)]
pub struct CameraController {
    eye: Point3<f32>,
    target: Vector3<f32>,
    up: Vector3<f32>,

    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            eye: Point3::new(10.0, 10.0, 10.0),
            target: Vector3::new(-5.0, -5.0, -5.0),
            up: Vector3::unit_y(),
            aspect: 90.0,
            fovy: 90.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}

#[allow(dead_code)]
impl CameraController {
    pub(crate) fn set_aspect(&mut self, new_aspect: f32) {
        self.aspect = new_aspect;
    }

    pub fn forward(&mut self, amount: f32) {
        self.eye += self.target.normalize() * amount;
    }

    pub fn backward(&mut self, amount: f32) {
        self.eye -= self.target.normalize() * amount;
    }

    pub fn strafe_left(&mut self, amount: f32) {
        self.eye += self.up.cross(self.target).normalize() * amount;
    }

    pub fn strafe_right(&mut self, amount: f32) {
        self.eye -= self.up.cross(self.target).normalize() * amount;
    }

    pub fn up(&mut self, amount: f32) {
        self.eye += self.up.normalize() * amount;
    }

    pub fn down(&mut self, amount: f32) {
        self.eye -= self.up.normalize() * amount;
    }

    pub fn zoom_in(&mut self, amount: f32) {
        self.fovy = (self.fovy - amount).clamp(FOVY_CLAMP.0, FOVY_CLAMP.1);
    }

    pub fn zoom_out(&mut self, amount: f32) {
        self.fovy = (self.fovy + amount).clamp(FOVY_CLAMP.0, FOVY_CLAMP.1);
    }

    pub fn look(&mut self, delta: (f32, f32)) {
        let forward_norm = self.target.normalize();
        let right = forward_norm.cross(self.up);

        // Change Pitch
        let rotation = Quaternion {
            v: right * f32::sin(delta.1),
            s: f32::cos(delta.0),
        }
        .normalize();

        self.target = rotation.rotate_vector(self.target);

        let up_norm = self.up.normalize();
        // Change Yaw
        let rotation = Quaternion {
            v: up_norm * f32::sin(delta.0),
            s: f32::cos(delta.1),
        }
        .normalize();

        self.target = rotation.rotate_vector(self.target);
    }
}

#[derive(Debug)]
pub struct Camera3D;

#[derive(Debug)]
pub struct PhotonCamera {
    // Position and direction
    eye: Point3<f32>,
    target: Vector3<f32>,
    up: Vector3<f32>,

    // Camera view values
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,

    camera_uniform: CameraUniform,
    buffer: Buffer,
    gpu_controller: Arc<GpuController>,
    pub(crate) bind_group: BindGroup,
    uniform_dirty: bool,
}

impl PhotonCamera {
    pub(crate) fn create_new_camera_3d(
        gpu_controller: Arc<GpuController>,
        eye: Point3<f32>,
        target: Vector3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let view = Matrix4::look_at_rh(eye, eye + target, up);
        let proj = perspective(Deg(fovy), aspect, znear, zfar);

        let view_proj = OPENGL_TO_WGPU_MATIX * proj * view;

        let camera_uniform = CameraUniform {
            view_position: eye.to_homogeneous().into(),
            view_projection: view_proj.into(),
        };

        let buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Camera Uniform Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

        let bind_group = gpu_controller
            .device
            .create_bind_group(&BindGroupDescriptor {
                label: Some("Photon Camera 3D Bind Group"),
                layout: &gpu_controller.layouts.camera_layout,
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
            bind_group,
            gpu_controller,
            uniform_dirty: false,
        }
    }

    pub(crate) fn write_buffer(&mut self) {
        if self.uniform_dirty {
            self.gpu_controller.queue.write_buffer(
                &self.buffer,
                0,
                bytemuck::cast_slice(&[self.camera_uniform]),
            );

            self.uniform_dirty = false;
        }
    }

    pub(crate) fn link_transform(&mut self, transform: &Transform) {
        self.eye = Point3::from_vec(transform.position);
        self.target = transform.orientation.rotate_vector(self.target);

        let view = Matrix4::look_at_rh(self.eye, self.eye + self.target, self.up);

        let proj = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);

        let view_proj = OPENGL_TO_WGPU_MATIX * proj * view;
        self.camera_uniform = CameraUniform {
            view_position: transform.position.extend(1.0).into(),
            view_projection: view_proj.into(),
        };

        self.gpu_controller.queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub(crate) fn link_cam_controller(&mut self, cam_cont: &CameraController) {
        let view = Matrix4::look_at_rh(cam_cont.eye, cam_cont.eye + cam_cont.target, cam_cont.up);
        let proj = perspective(
            Deg(cam_cont.fovy),
            cam_cont.aspect,
            cam_cont.znear,
            cam_cont.zfar,
        );

        let view_proj = OPENGL_TO_WGPU_MATIX * proj * view;

        self.camera_uniform = CameraUniform {
            view_position: cam_cont.eye.to_homogeneous().into(),
            view_projection: view_proj.into(),
        };

        self.gpu_controller.queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    // Used internally for changing the screen size
    pub(crate) fn set_aspect(&mut self, new_aspect: f32) {
        let view = Matrix4::look_at_rh(self.eye, self.eye + self.target, self.up);
        let proj = perspective(Deg(self.fovy), new_aspect, self.znear, self.zfar);

        let view_proj = OPENGL_TO_WGPU_MATIX * proj * view;

        self.aspect = new_aspect;
        self.camera_uniform = CameraUniform {
            view_position: self.eye.to_homogeneous().into(),
            view_projection: view_proj.into(),
        };

        self.gpu_controller.queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_position: Vector4,
    pub view_projection: Matrix4x4,
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_position: [0.0; 4],
            view_projection: Matrix4::identity().into(),
        }
    }
}
