use std::sync::Arc;

use cgmath::{Deg, InnerSpace, Matrix4, Point3, SquareMatrix, Vector3, perspective};
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

    // Call after changing anything
    fn update(&mut self) {
        let view = Matrix4::look_at_rh(self.eye, self.eye + self.target, self.up);
        let proj = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);

        let view_proj = OPENGL_TO_WGPU_MATIX * proj * view;

        self.camera_uniform = CameraUniform {
            view_position: self.eye.to_homogeneous().into(),
            view_projection: view_proj.into(),
        };

        self.uniform_dirty = true;
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

    pub fn modify<F>(&mut self, callback: F)
    where
        F: Fn(
            &mut Point3<f32>,
            &mut Vector3<f32>,
            &mut Vector3<f32>,
            &mut f32,
            &mut f32,
            &mut f32,
            &mut f32,
        ),
    {
        callback(
            &mut self.eye,
            &mut self.target,
            &mut self.up,
            &mut self.aspect,
            &mut self.fovy,
            &mut self.znear,
            &mut self.zfar,
        );

        // Value clamping to prevent crashing
        self.fovy = self.fovy.clamp(FOVY_CLAMP.0, FOVY_CLAMP.1);
        // Make sure to normalized the target
        self.target = self.target.normalize();

        self.update();
    }

    pub fn modify_eye<F>(&mut self, callback: F)
    where
        F: Fn(&mut Point3<f32>),
    {
        callback(&mut self.eye);

        self.update();
    }

    pub fn modify_target<F>(&mut self, callback: F)
    where
        F: Fn(&mut Vector3<f32>),
    {
        callback(&mut self.target);

        // Make sure to normalized the target
        self.target = self.target.normalize();

        self.update();
    }

    pub fn modify_up<F>(&mut self, callback: F)
    where
        F: Fn(&mut Vector3<f32>),
    {
        callback(&mut self.up);

        self.update();
    }

    pub fn modify_aspect<F>(&mut self, callback: F)
    where
        F: Fn(&mut f32),
    {
        callback(&mut self.aspect);

        self.update();
    }

    pub fn modify_fovy<F>(&mut self, callback: F)
    where
        F: Fn(&mut f32),
    {
        callback(&mut self.fovy);

        // Value clamping to prevent crashing
        self.fovy = self.fovy.clamp(FOVY_CLAMP.0, FOVY_CLAMP.1);

        self.update();
    }

    pub fn modify_znear<F>(&mut self, callback: F)
    where
        F: Fn(&mut f32),
    {
        callback(&mut self.znear);

        self.update();
    }

    pub fn modify_zfar<F>(&mut self, callback: F)
    where
        F: Fn(&mut f32),
    {
        callback(&mut self.zfar);

        self.update();
    }

    pub(crate) fn link_transform(&mut self, transform: &Transform) {
        let view = Matrix4::look_at_rh(
            Point3::from_homogeneous(transform.position.extend(1.0)),
            Point3::from_homogeneous((transform.position + self.target).extend(1.0)),
            self.up,
        );

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
