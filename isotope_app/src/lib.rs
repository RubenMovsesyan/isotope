use std::sync::Arc;

use anyhow::Result;
use gpu_controller::GpuController;
use log::info;
use matter_vault::MatterVault;
use photon::renderer::{Renderer, defered_renderer::DeferedRenderer3D};
use wgpu::{CompositeAlphaMode, PresentMode, SurfaceConfiguration, TextureFormat, TextureUsages};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalSize, Size},
    event::WindowEvent,
    window::Window,
};

pub struct Isotope {
    // GPU
    gpu_controller: Arc<GpuController>,

    // Asset Manager
    // matter_vault: Arc<MatterVault>,

    // Rendering
    photon: Renderer,
}

impl Isotope {
    pub fn new(gpu_controller: Arc<GpuController>) -> Self {
        Self {
            photon: Renderer::new_defered_3d(gpu_controller.clone()),
            gpu_controller,
        }
    }
}

pub struct IsotopeApplication {
    window: Option<Arc<Window>>,
    isotope: Isotope,
}

impl IsotopeApplication {
    pub fn new() -> Result<Self> {
        info!("Creating Gpu Controller");
        let gpu_controller = GpuController::new(
            None,
            None,
            Some(SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: TextureFormat::Rbga8UnormSrgb,
                width: 1,
                height: 1,
                present_mode: PresentMode::AutoNoVsync,
                desired_maximum_frame_latency: 2,
                alpha_mode: CompositeAlphaMode::Auto,
                view_formats: vec![],
            }),
        )?;

        Ok(Self {
            window: None,
            isotope: Isotope::new(gpu_controller),
        })
    }
}

impl ApplicationHandler for IsotopeApplication {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        info!("Isotope Resumed");

        self.window = Some(Arc::new(
            event_loop.create_window(
                Window::default_attributes()
                    .with_title("Isotope")
                    .with_inner_size(Size::Physical(PhysicalSize {
                        width: 640,
                        height: 480,
                    })),
            ),
        ));
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(window) = self.window.as_ref() {
            if window.id() == window_id {
                match event {
                    WindowEvent::CloseRequested => {
                        info!("Shutting Down Isotope");

                        event_loop.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        self.isotope.photon.render();
                    }
                    _ => {}
                }
            }
        }
    }
}
