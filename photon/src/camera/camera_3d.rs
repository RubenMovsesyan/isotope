use std::sync::Arc;

use cgmath::{Deg, InnerSpace, Matrix4, Point3, Vector3, perspective};
use gpu_controller::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferInitDescriptor, BufferUsages,
    GpuController,
};

use super::{CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR, OPENGL_TO_WGPU_MATIX};

// Clamping constants
const FOVY_CLAMP: (f32, f32) = (0.1, 179.9);
const MAX_UP_DOT: f32 = 0.99;

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

    fn update(&mut self) {
        self.target = self.target.normalize();
        let view = Matrix4::look_to_rh(self.eye, self.target, self.up);
        let proj = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);

        let view_proj = OPENGL_TO_WGPU_MATIX * proj * view;

        self.camera_uniform = PerspectiveCam3DUniform {
            view_position: self.eye.to_homogeneous().into(),
            view_projection: view_proj.into(),
        };

        self.gpu_controller.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    /// Provides mutable access to the camera's eye position.
    ///
    /// # Arguments
    /// * `callback` - A closure that receives a mutable reference to the eye position
    pub fn eye<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut Point3<f32>),
    {
        callback(&mut self.eye);
        self.update();
    }

    /// Provides mutable access to the camera's target direction vector.
    ///
    /// # Arguments
    /// * `callback` - A closure that receives a mutable reference to the target vector
    pub fn target<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut Vector3<f32>),
    {
        callback(&mut self.target);
        // TODO: add limiting to the target to prevent camera lock
        self.update();
    }

    /// Provides mutable access to the camera's up vector.
    ///
    /// # Arguments
    /// * `callback` - A closure that receives a mutable reference to the up vector
    pub fn up<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut Vector3<f32>),
    {
        callback(&mut self.up);
        self.update();
    }

    /// Provides mutable access to the camera's aspect ratio.
    ///
    /// # Arguments
    /// * `callback` - A closure that receives a mutable reference to the aspect ratio
    pub fn aspect<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut f32),
    {
        callback(&mut self.aspect);
        self.update();
    }

    /// Provides mutable access to the camera's field of view in the y direction.
    ///
    /// The field of view value is automatically clamped to safe limits (0.1 to 179.9 degrees)
    /// after the callback executes to prevent rendering issues.
    ///
    /// # Arguments
    /// * `callback` - A closure that receives a mutable reference to the field of view in degrees
    pub fn fovy<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut f32),
    {
        callback(&mut self.fovy);
        self.fovy = self.fovy.clamp(FOVY_CLAMP.0, FOVY_CLAMP.1);
        self.update();
    }

    /// Provides mutable access to the camera's near clipping plane distance.
    ///
    /// # Arguments
    /// * `callback` - A closure that receives a mutable reference to the near clipping distance
    pub fn znear<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut f32),
    {
        callback(&mut self.znear);
        self.update();
    }

    /// Provides mutable access to the camera's far clipping plane distance.
    ///
    /// # Arguments
    /// * `callback` - A closure that receives a mutable reference to the far clipping distance
    pub fn zfar<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut f32),
    {
        callback(&mut self.zfar);
        self.update();
    }

    /// Provides mutable access to all camera parameters at once.
    ///
    /// # Arguments
    /// * `callback` - A closure that receives mutable references to all camera parameters
    pub fn all<F>(&mut self, callback: F)
    where
        F: FnOnce(
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
        self.fovy = self.fovy.clamp(FOVY_CLAMP.0, FOVY_CLAMP.1);
        self.update();
    }
}
