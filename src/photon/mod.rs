use std::sync::Arc;

use anyhow::Result;
use window::{DEFAULT_HEIGHT, DEFAULT_WIDTH, PhotonWindow};
use winit::{event_loop::ActiveEventLoop, window::Window};

use crate::gpu_utils::GpuController;

pub mod renderer;
pub mod window;

#[derive(Debug)]
pub struct PhotonManager {
    pub window: PhotonWindow,
}

impl PhotonManager {
    pub fn new(event_loop: &ActiveEventLoop, gpu_controller: Arc<GpuController>) -> Result<Self> {
        Ok(Self {
            window: PhotonWindow::new(
                event_loop,
                DEFAULT_WIDTH,
                DEFAULT_HEIGHT,
                &gpu_controller,
                "Isotope",
            )?,
        })
    }

    pub fn window(&self) -> &Window {
        &self.window.window
    }
}
