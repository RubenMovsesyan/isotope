use cgmath::{InnerSpace, Point3, Vector3};
use log::warn;
use photon::camera::{PerspectiveCamera3D, PhotonCamera};

use crate::AssetServer;

const DEFAULT_FOVY: f32 = 45.0;
const DEFAULT_NEAR: f32 = 0.1;
const DEFAULT_FAR: f32 = 1000.0;

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

    pub fn perspective_3d_default(asset_server: &AssetServer) -> Self {
        let aspect = asset_server
            .gpu_controller
            .read_surface_config(|sc| sc.width as f32 / sc.height as f32)
            .unwrap_or_else(|err| {
                warn!("Error getting surface config for camera: {}", err);
                1.0
            });

        Self::PerspectiveCamera3D(PerspectiveCamera3D::new(
            asset_server.gpu_controller.clone(),
            Point3::from([0.0, 0.0, 0.0]),
            Vector3::from([-1.0, 0.0, -1.0]).normalize(),
            Vector3::unit_y(),
            aspect,
            DEFAULT_FOVY,
            DEFAULT_NEAR,
            DEFAULT_FAR,
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
