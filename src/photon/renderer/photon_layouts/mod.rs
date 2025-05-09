use wgpu::BindGroupLayout;

use crate::GpuController;

pub mod camera;
pub mod lights;
pub mod texture;

#[derive(Debug)]
pub(crate) struct PhotonLayoutsManager {
    pub texture_layout: BindGroupLayout,
    pub camera_layout: BindGroupLayout,
    pub lights_layout: BindGroupLayout,
}

impl PhotonLayoutsManager {
    pub fn new(gpu_controller: &GpuController) -> Self {
        let texture_layout = texture::create_bind_group_layout(&gpu_controller.device);
        let camera_layout = camera::create_bind_group_layout(&gpu_controller.device);
        let lights_layout = lights::create_bind_group_layout(&gpu_controller.device);

        Self {
            texture_layout,
            camera_layout,
            lights_layout,
        }
    }
}
