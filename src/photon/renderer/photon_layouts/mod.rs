use wgpu::BindGroupLayout;

use crate::GpuController;

pub mod camera;
pub mod collider;
pub mod lights;
pub mod material;
pub mod model;
pub mod texture;

#[derive(Debug)]
pub(crate) struct PhotonLayoutsManager {
    pub texture_layout: BindGroupLayout,
    pub camera_layout: BindGroupLayout,
    pub lights_layout: BindGroupLayout,
    pub model_layout: BindGroupLayout,
    pub material_layout: BindGroupLayout,
    pub collider_layout: BindGroupLayout,
}

impl PhotonLayoutsManager {
    pub fn new(gpu_controller: &GpuController) -> Self {
        let texture_layout = texture::create_bind_group_layout(&gpu_controller.device);
        let camera_layout = camera::create_bind_group_layout(&gpu_controller.device);
        let lights_layout = lights::create_bind_group_layout(&gpu_controller.device);
        let model_layout = model::create_bind_group_layout(&gpu_controller.device);
        let material_layout = material::create_bind_group_layout(&gpu_controller.device);
        let collider_layout = collider::create_bind_group_layout(&gpu_controller.device);

        Self {
            texture_layout,
            camera_layout,
            lights_layout,
            model_layout,
            material_layout,
            collider_layout,
        }
    }
}
