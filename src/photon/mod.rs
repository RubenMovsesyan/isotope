use std::sync::Arc;

use anyhow::Result;
use renderer::PhotonRenderer;
use wgpu::RenderPass;
use window::{DEFAULT_HEIGHT, DEFAULT_WIDTH, PhotonWindow};
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop, window::Window};

use crate::{Light, gpu_utils::GpuController};

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

        let mut renderer = PhotonRenderer::new(gpu_controller.clone());
        renderer.add_debug_render_pipeline(); // Disable if debug not wanted

        Ok(Self { window, renderer })
    }

    pub fn window(&self) -> &Window {
        &self.window.window
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.window.resize(new_size);
        self.renderer.resize(new_size);
    }

    // Call on request redraw
    pub fn render<F, D>(&mut self, callback: F, lights: &[Light], debug_callback: D) -> Result<()>
    where
        F: FnOnce(&mut RenderPass),
        D: FnOnce(&mut RenderPass),
    {
        self.renderer.update_lights(lights);
        self.renderer
            .render(&self.window.surface, callback, debug_callback)
    }
}
