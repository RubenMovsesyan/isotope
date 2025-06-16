use std::sync::Arc;

use anyhow::Result;
use renderer::PhotonRenderer;
use window::{DEFAULT_HEIGHT, DEFAULT_WIDTH, PhotonWindow};
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop, window::Window};

use crate::gpu_utils::GpuController;

pub mod instancer;
pub mod render_descriptor;
pub mod renderer;
pub mod window;

#[derive(Debug)]
pub struct PhotonManager {
    pub window: PhotonWindow,
    pub renderer: PhotonRenderer,
}

impl PhotonManager {
    pub fn new(event_loop: &ActiveEventLoop, gpu_controller: Arc<GpuController>) -> Result<Self> {
        let window = PhotonWindow::new(
            event_loop,
            DEFAULT_WIDTH,
            DEFAULT_HEIGHT,
            gpu_controller.clone(),
            "Isotope",
        )?;

        let renderer = PhotonRenderer::new(gpu_controller.clone());

        Ok(Self { window, renderer })
    }

    pub fn window(&self) -> &Window {
        &self.window.window
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.window.resize(new_size);
        self.renderer.resize(new_size);
    }
}
