use std::sync::Arc;

use wgpu::RenderPipeline;

use crate::GpuController;

#[derive(Debug)]
pub struct PhotonRenderer {
    gpu_controller: Arc<GpuController>,
    render_pipeline: RenderPipeline,
}

impl PhotonRenderer {
    pub fn new(gpu_controller: Arc<GpuController>) -> Self {
        todo!()
    }
}
