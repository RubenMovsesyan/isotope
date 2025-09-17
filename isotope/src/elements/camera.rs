use cgmath::{Point3, Vector3};
use log::warn;
use photon::camera::{PerspectiveCamera3D, PhotonCamera};

use crate::AssetServer;

pub enum Camera {
    PerspectiveCamera3D(PerspectiveCamera3D),
}

impl PhotonCamera for Camera {
    #[inline]
    fn bind_group(&self) -> &gpu_controller::BindGroup {
        match self {
            Self::PerspectiveCamera3D(camera) => camera.bind_group(),
        }
    }
}

impl Camera {
    pub fn new_perspective_3d<V1, V2, V3>(
        asset_server: &AssetServer,
        eye: V1,
        target: V2,
        up: V3,
        fovy: f32,
        near: f32,
        far: f32,
    ) -> Self
    where
        V1: Into<[f32; 3]>,
        V2: Into<[f32; 3]>,
        V3: Into<[f32; 3]>,
    {
        let aspect = asset_server
            .gpu_controller
            .read_surface_config(|sc| sc.width as f32 / sc.height as f32)
            .unwrap_or_else(|err| {
                warn!("Error getting surface config for camera: {}", err);
                1.0
            });

        Self::PerspectiveCamera3D(PerspectiveCamera3D::new(
            asset_server.gpu_controller.clone(),
            Point3::from(eye.into()),
            Vector3::from(target.into()),
            Vector3::from(up.into()),
            aspect,
            fovy,
            near,
            far,
        ))
    }

    pub fn eye<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut Point3<f32>),
    {
        match self {
            Self::PerspectiveCamera3D(camera) => camera.eye(callback),
        }
    }

    pub fn target<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut Vector3<f32>),
    {
        match self {
            Self::PerspectiveCamera3D(camera) => camera.target(callback),
        }
    }

    pub fn up<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut Vector3<f32>),
    {
        match self {
            Self::PerspectiveCamera3D(camera) => camera.up(callback),
        }
    }

    pub fn aspect<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut f32),
    {
        match self {
            Self::PerspectiveCamera3D(camera) => camera.aspect(callback),
        }
    }

    pub fn fovy<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut f32),
    {
        match self {
            Self::PerspectiveCamera3D(camera) => camera.fovy(callback),
        }
    }

    pub fn znear<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut f32),
    {
        match self {
            Self::PerspectiveCamera3D(camera) => camera.znear(callback),
        }
    }

    pub fn zfar<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut f32),
    {
        match self {
            Self::PerspectiveCamera3D(camera) => camera.zfar(callback),
        }
    }

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
        match self {
            Self::PerspectiveCamera3D(camera) => camera.all(callback),
        }
    }
}
