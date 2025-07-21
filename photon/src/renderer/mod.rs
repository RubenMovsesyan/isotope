use std::sync::Arc;

use anyhow::Result;
use defered_renderer::DeferedRenderer3D;
use gpu_controller::GpuController;

mod defered_renderer;

const CAMERA_BIND_GROUP: u32 = 0;

pub enum Renderer {
    Defered3D(DeferedRenderer3D),
}

impl Renderer {
    pub fn new_defered_3d(gpu_controller: Arc<GpuController>) -> Result<Self> {
        Ok(Self::Defered3D(DeferedRenderer3D::new(gpu_controller)?))
    }

    pub fn render(&self) {}
}
