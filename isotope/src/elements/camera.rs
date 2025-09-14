use photon::camera::Camera;

use crate::AssetServer;

pub fn new_perspective_camera<V1, V2, V3>(
    asset_server: &AssetServer,
    eye: V1,
    target: V2,
    up: V3,
    fovy: f32,
    near: f32,
    far: f32,
) -> Camera
where
    V1: Into<[f32; 3]>,
    V2: Into<[f32; 3]>,
    V3: Into<[f32; 3]>,
{
    Camera::new_perspective_3d(
        asset_server.gpu_controller.clone(),
        eye,
        target,
        up,
        fovy,
        near,
        far,
    )
}
