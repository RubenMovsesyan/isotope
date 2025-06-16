use std::sync::Arc;

use cgmath::{
    Deg, EuclideanSpace, InnerSpace, Matrix4, Point3, Quaternion, Rad, Rotation, Rotation3,
    SquareMatrix, Vector3, perspective,
};
use frustum::Frustum;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferUsages,
    util::{BufferInitDescriptor, DeviceExt},
};

mod frustum;

use crate::{
    GpuController, Transform,
    photon::window::{DEFAULT_HEIGHT, DEFAULT_WIDTH},
};

pub(crate) type Vector4 = [f32; 4];
pub(crate) type Matrix4x4 = [[f32; 4]; 4];

pub const OPENGL_TO_WGPU_MATIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
);

// Clamping constants
const FOVY_CLAMP: (f32, f32) = (0.1, 179.9);

const MAX_UP_DOT: f32 = 0.99;

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
            aspect: DEFAULT_WIDTH as f32 / DEFAULT_HEIGHT as f32,
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
        let up_norm = self.up.normalize();
        let right = forward_norm.cross(up_norm);

        // Pitch slerp
        let pitch_rotation = Quaternion::from_axis_angle(right, Rad(delta.1));

        // Yaw slerp
        let yaw_rotation = Quaternion::from_axis_angle(up_norm, Rad(delta.0));

        // Change Pitch and Yaw
        self.target = yaw_rotation.rotate_vector(self.target);

        // Make sure to clamp the pitch rotation so it doesn't go over
        let new_target = pitch_rotation.rotate_vector(self.target);
        if new_target.normalize().dot(up_norm).abs() < MAX_UP_DOT {
            self.target = new_target;
        }

        self.target = self.target.normalize();
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

    // For frustum culling
    pub(crate) frustum: Frustum,

    camera_uniform: CameraUniform,
    buffer: Buffer,
    gpu_controller: Arc<GpuController>,
    pub(crate) bind_group: BindGroup,
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

        let frustum = Frustum::default();

        Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
            frustum,
            camera_uniform,
            buffer,
            bind_group,
            gpu_controller,
        }
    }

    pub(crate) fn link_transform(&mut self, transform: &Transform) {
        self.eye = Point3::from_vec(transform.position);
        self.target = transform.orientation.rotate_vector(self.target).normalize();

        self.frustum.update(
            self.eye,
            self.target,
            self.up,
            self.aspect,
            self.fovy,
            self.znear,
            self.zfar,
        );

        // let view = Matrix4::look_at_rh(self.eye, self.eye + self.target, self.up);
        let view = Matrix4::look_to_rh(self.eye, self.target, self.up);
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
        self.eye = cam_cont.eye;
        self.target = cam_cont.target.normalize();
        self.up = cam_cont.up.normalize();
        self.aspect = cam_cont.aspect;
        self.fovy = cam_cont.fovy;
        self.znear = cam_cont.znear;
        self.zfar = cam_cont.zfar;

        self.frustum.update(
            self.eye,
            self.target,
            self.up,
            self.aspect,
            self.fovy,
            self.znear,
            self.zfar,
        );

        // let view = Matrix4::look_at_rh(self.eye, self.eye + self.target, self.up);
        let view = Matrix4::look_to_rh(self.eye, self.target, self.up);
        let proj = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);

        let view_proj = OPENGL_TO_WGPU_MATIX * proj * view;

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

    // Used internally for changing the screen size
    pub(crate) fn set_aspect(&mut self, new_aspect: f32) {
        // let view = Matrix4::look_at_rh(self.eye, self.eye + self.target, self.up);
        let view = Matrix4::look_to_rh(self.eye, self.target, self.up);
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
