use wgpu::BindGroupLayout;

use crate::GpuController;

pub mod texture;

#[derive(Debug)]
pub(crate) struct PhotonLayoutsManager {
    pub texture_layout: BindGroupLayout,
}

impl PhotonLayoutsManager {
    pub fn new(gpu_controller: &GpuController) -> Self {
        let texture_layout = texture::create_bind_group_layout(&gpu_controller.device);

        Self { texture_layout }
    }
}
