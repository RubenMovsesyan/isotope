use std::sync::Arc;

use anyhow::Result;
use wgpu::{PresentMode, Surface, SurfaceConfiguration, TextureUsages};
use winit::{
    dpi::{PhysicalSize, Size},
    event_loop::ActiveEventLoop,
    window::Window,
};

use crate::GpuController;

pub const DEFAULT_WIDTH: u32 = 1280;
pub const DEFAULT_HEIGHT: u32 = 720;

#[allow(dead_code)]
#[derive(Debug)]
pub struct PhotonWindow {
    pub window: Arc<Window>,
    surface: Arc<Surface<'static>>,
    pub surface_configuration: SurfaceConfiguration,
}

impl PhotonWindow {
    pub fn new(
        event_loop: &ActiveEventLoop,
        width: u32,
        height: u32,
        gpu_controller: &GpuController,
        title: &str,
    ) -> Result<Self> {
        // Create the window
        let window = Arc::new(
            event_loop.create_window(
                Window::default_attributes()
                    .with_title(title)
                    .with_inner_size(Size::Physical(PhysicalSize { width, height })),
            )?,
        );

        // Create the rendering surface
        let surface = Arc::new(gpu_controller.instance.create_surface(window.clone())?);
        let surface_capabilities = surface.get_capabilities(&gpu_controller.adapter);
        let size = window.inner_size();

        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|texture_format| texture_format.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        let surface_configuration = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::AutoNoVsync,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&gpu_controller.device, &surface_configuration);

        Ok(Self {
            window,
            surface,
            surface_configuration,
        })
    }
}
