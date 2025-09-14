use std::sync::Arc;

use anyhow::Result;
use gpu_controller::{GpuController, Surface};
use winit::{
    dpi::{PhysicalSize, Size},
    event_loop::ActiveEventLoop,
    window::Window,
};

pub struct RenderingWindow {
    gpu_controller: Arc<GpuController>,
    pub(crate) window: Arc<Window>,
    pub(crate) surface: Arc<Surface<'static>>,
}

pub struct WindowInitializer {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

impl RenderingWindow {
    pub fn new(
        event_loop: &ActiveEventLoop,
        gpu_controller: Arc<GpuController>,
        window_initializer: WindowInitializer,
    ) -> Result<Self> {
        let window = Arc::new(
            event_loop.create_window(
                Window::default_attributes()
                    .with_inner_size(Size::Physical(PhysicalSize {
                        width: window_initializer.width,
                        height: window_initializer.height,
                    }))
                    .with_title(&window_initializer.title),
            )?,
        );

        let surface = Arc::new(gpu_controller.create_surface(window.clone())?);

        Ok(Self {
            gpu_controller,
            window,
            surface,
        })
    }
}
