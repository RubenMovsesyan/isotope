use std::sync::Arc;

use anyhow::Result;
use renderer::PhotonRenderer;
use window::{DEFAULT_HEIGHT, DEFAULT_WIDTH, PhotonWindow};
use winit::{event_loop::ActiveEventLoop, window::Window};

use crate::{Element, gpu_utils::GpuController};

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
            &gpu_controller,
            "Isotope",
        )?;

        let renderer = PhotonRenderer::new(gpu_controller, &window.surface_configuration);

        Ok(Self { window, renderer })
    }

    pub fn window(&self) -> &Window {
        &self.window.window
    }

    pub fn render(&self, elements: &[Arc<dyn Element>]) -> Result<()> {
        self.renderer.render(&self.window.surface, elements)
    }
}
