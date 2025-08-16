use std::sync::Arc;

use anyhow::Result;
use gpu_controller::{GpuController, Mesh, Vertex};
use log::info;
use matter_vault::MatterVault;
use photon::{
    camera::Camera,
    renderer::{Renderer, defered_renderer::DeferedRenderer3D},
};
use rendering_window::{RenderingWindow, WindowInitializer};
use smol::block_on;
use wgpu::{CompositeAlphaMode, PresentMode, SurfaceConfiguration, TextureFormat, TextureUsages};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalSize, Size},
    event::WindowEvent,
    window::Window,
};

mod rendering_window;

pub struct Isotope {
    // GPU
    gpu_controller: Arc<GpuController>,

    // Asset Manager
    matter_vault: Arc<MatterVault>,

    // Rendering
    photon: Renderer,

    // Temp
    camera: Camera,
    temp_cube: Mesh,
}

impl Isotope {
    pub fn new(gpu_controller: Arc<GpuController>) -> Result<Self> {
        let temp_cube = Mesh::new(
            gpu_controller.clone(),
            "Cube".to_string(),
            &[
                Vertex {
                    position: [0.0, 0.0, 0.0],
                    normal_vec: [0.0, 0.0, 0.0],
                    uv_coord: [0.0, 0.0],
                },
                Vertex {
                    position: [1.0, 0.0, 0.0],
                    normal_vec: [0.0, 0.0, 0.0],
                    uv_coord: [1.0, 0.0],
                },
                Vertex {
                    position: [1.0, 1.0, 0.0],
                    normal_vec: [0.0, 0.0, 0.0],
                    uv_coord: [1.0, 1.0],
                },
                Vertex {
                    position: [0.0, 1.0, 0.0],
                    normal_vec: [0.0, 0.0, 0.0],
                    uv_coord: [0.0, 1.0],
                },
                Vertex {
                    position: [0.0, 0.0, 1.0],
                    normal_vec: [0.0, 0.0, 0.0],
                    uv_coord: [0.0, 0.0],
                },
                Vertex {
                    position: [1.0, 0.0, 1.0],
                    normal_vec: [0.0, 0.0, 0.0],
                    uv_coord: [1.0, 0.0],
                },
                Vertex {
                    position: [1.0, 1.0, 1.0],
                    normal_vec: [0.0, 0.0, 0.0],
                    uv_coord: [1.0, 1.0],
                },
                Vertex {
                    position: [0.0, 1.0, 1.0],
                    normal_vec: [0.0, 0.0, 0.0],
                    uv_coord: [0.0, 1.0],
                },
            ],
            &[
                0, 1, 2, 2, 3, 0, // Front face
                4, 5, 6, 6, 7, 4, // Back face
                0, 4, 7, 7, 3, 0, // Left face
                1, 5, 6, 6, 2, 1, // Right face
                3, 7, 6, 6, 2, 3, // Top face
                0, 1, 5, 5, 4, 0, // Bottom face
            ],
        );

        Ok(Self {
            photon: Renderer::new_defered_3d(gpu_controller.clone())?,
            matter_vault: Arc::new(MatterVault::new()),
            // Temp camera setup
            camera: Camera::new_perspective_3d(
                gpu_controller.clone(),
                [10.0, 10.0, 10.0],
                [-5.0, -5.0, -5.0],
                [0.0, 1.0, 0.0],
                gpu_controller.with_surface_config(|sc| sc.width as f32 / sc.height as f32)?,
                90.0,
                0.1,
                100.0,
            ),
            gpu_controller,
            temp_cube,
        })
    }
}

pub struct IsotopeApplication {
    window: Option<RenderingWindow>,
    isotope: Isotope,
}

impl IsotopeApplication {
    pub fn new() -> Result<Self> {
        info!("Creating Gpu Controller");
        let gpu_controller = block_on(GpuController::new(
            None,
            None,
            Some(SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: TextureFormat::Rgba8UnormSrgb,
                width: 1,
                height: 1,
                present_mode: PresentMode::AutoNoVsync,
                desired_maximum_frame_latency: 2,
                alpha_mode: CompositeAlphaMode::Auto,
                view_formats: vec![],
            }),
        ))?;

        Ok(Self {
            window: None,
            isotope: Isotope::new(gpu_controller)?,
        })
    }
}

impl ApplicationHandler for IsotopeApplication {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        info!("Isotope Resumed");

        if let Ok(rendering_window) = RenderingWindow::new(
            event_loop,
            self.isotope.gpu_controller.clone(),
            WindowInitializer {
                width: 640,
                height: 480,
                title: "Isotope".to_string(),
            },
        ) {
            self.window = Some(rendering_window);
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(window) = self.window.as_ref() {
            if window.window.id() == window_id {
                match event {
                    WindowEvent::CloseRequested => {
                        info!("Shutting Down Isotope");

                        event_loop.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        if let Ok(surface_texture) = window.surface.get_current_texture() {
                            self.isotope.photon.render(
                                &self.isotope.camera,
                                &surface_texture.texture,
                                |render_pass| {
                                    self.isotope.temp_cube.render(render_pass);
                                },
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
