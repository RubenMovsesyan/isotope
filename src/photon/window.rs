use log::*;

use std::sync::Arc;

use anyhow::Result;
use wgpu::{PresentMode, Surface, SurfaceConfiguration, TextureUsages};
use winit::{
    dpi::{PhysicalSize, Size},
    event_loop::ActiveEventLoop,
    window::{CursorGrabMode, Window},
};

use crate::GpuController;

/// Default window width in pixels
pub const DEFAULT_WIDTH: u32 = 1920;
/// Default window height in pixels
pub const DEFAULT_HEIGHT: u32 = 1080;

/// Controller for managing window cursor behavior
#[derive(Debug)]
pub struct WindowController {
    window: Arc<Window>,
    cursor_grab_mode: CursorGrabMode,
    size: PhysicalSize<u32>,
    cursor_visible: bool,
}

impl WindowController {
    pub(crate) fn new(window: Arc<Window>) -> Self {
        Self {
            window,
            cursor_grab_mode: CursorGrabMode::None,
            size: PhysicalSize {
                width: DEFAULT_WIDTH,
                height: DEFAULT_HEIGHT,
            },
            cursor_visible: true,
        }
    }

    pub fn cursor_grab_mode<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut CursorGrabMode),
    {
        callback(&mut self.cursor_grab_mode);

        match self.window.set_cursor_grab(self.cursor_grab_mode) {
            Err(err) => {
                error!(
                    "Error {} setting cursor grab mode to: {:#?}",
                    err, self.cursor_grab_mode
                );
            }
            _ => {}
        }
    }

    pub fn cursor_visible<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut bool),
    {
        callback(&mut self.cursor_visible);

        debug!("Setting Cursor Visible to: {}", self.cursor_visible);
        self.window.set_cursor_visible(self.cursor_visible);
    }

    pub fn resize<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut PhysicalSize<u32>),
    {
        callback(&mut self.size);
    }

    // This is the stuff that needs to be set every frame
    pub(crate) fn run_frame_updates(&self) {
        self.window.set_cursor_visible(self.cursor_visible);

        if let Err(err) = self.window.set_cursor_grab(self.cursor_grab_mode) {
            error!("Failed to set the cursor grab mode: {}", err);
        }
    }
}

/// Main window structure that manages the rendering surface and GPU resources
#[allow(dead_code)]
#[derive(Debug)]
pub struct PhotonWindow {
    gpu_controller: Arc<GpuController>,
    pub window: Arc<Window>,
    pub surface: Arc<Surface<'static>>,
}

impl PhotonWindow {
    /// Creates a new PhotonWindow with the specified dimensions and GPU controller
    ///
    /// # Parameters
    /// - `event_loop`: Active event loop for window creation
    /// - `width`: Window width in pixels
    /// - `height`: Window height in pixels
    /// - `gpu_controller`: Shared GPU controller for rendering operations
    /// - `title`: Window title string
    ///
    /// # Returns
    /// - `Result<Self>`: New PhotonWindow instance or error if creation failed
    pub fn new(
        event_loop: &ActiveEventLoop,
        width: u32,
        height: u32,
        gpu_controller: Arc<GpuController>,
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

        if let Ok(mut surface_config) = gpu_controller.surface_configuration.write() {
            *surface_config = surface_configuration;
        }

        Ok(Self {
            gpu_controller,
            window,
            surface,
        })
    }

    /// Resizes the surface configuration of the window
    ///
    /// # Parameters
    /// - `new_size`: New physical size dimensions for the window
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if let Ok(mut surface_config) = self.gpu_controller.surface_configuration.write() {
            surface_config.width = new_size.width;
            surface_config.height = new_size.height;

            self.surface
                .configure(&self.gpu_controller.device, &surface_config);
        }

        debug!("Window resized to: {:#?}", new_size);
    }
}
