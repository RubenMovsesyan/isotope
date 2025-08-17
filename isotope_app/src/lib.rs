use std::sync::Arc;

use anyhow::Result;
use gpu_controller::{GpuController, Mesh, Vertex};
use log::{debug, error, info};
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
                    position: [1.0, 1.0, -1.0],
                    normal_vec: [0.57735027, 0.57735027, -0.57735027],
                    uv_coord: [1.0, 1.0],
                },
                Vertex {
                    position: [1.0, -1.0, -1.0],
                    normal_vec: [0.57735027, -0.57735027, -0.57735027],
                    uv_coord: [1.0, 0.0],
                },
                Vertex {
                    position: [1.0, 1.0, 1.0],
                    normal_vec: [0.57735027, 0.57735027, 0.57735027],
                    uv_coord: [1.0, 1.0],
                },
                Vertex {
                    position: [1.0, -1.0, 1.0],
                    normal_vec: [0.57735027, -0.57735027, 0.57735027],
                    uv_coord: [1.0, 0.0],
                },
                Vertex {
                    position: [-1.0, 1.0, -1.0],
                    normal_vec: [-0.57735027, 0.57735027, -0.57735027],
                    uv_coord: [0.0, 1.0],
                },
                Vertex {
                    position: [-1.0, -1.0, -1.0],
                    normal_vec: [-0.57735027, -0.57735027, -0.57735027],
                    uv_coord: [0.0, 0.0],
                },
                Vertex {
                    position: [-1.0, 1.0, 1.0],
                    normal_vec: [-0.57735027, 0.57735027, 0.57735027],
                    uv_coord: [0.0, 1.0],
                },
                Vertex {
                    position: [-1.0, -1.0, 1.0],
                    normal_vec: [-0.57735027, -0.57735027, 0.57735027],
                    uv_coord: [0.0, 0.0],
                },
            ],
            &[
                5, 3, 1, 3, 8, 4, //
                7, 6, 8, 2, 8, 6, //
                1, 4, 2, 5, 2, 6, //
                5, 7, 3, 3, 7, 8, //
                7, 5, 6, 2, 4, 8, //
                1, 3, 4, 5, 1, 2, //
            ],
        );

        Ok(Self {
            photon: Renderer::new_defered_3d(gpu_controller.clone())?,
            matter_vault: Arc::new(MatterVault::new()),
            // Temp camera setup
            camera: Camera::new_perspective_3d(
                gpu_controller.clone(),
                [5.0, 5.0, 5.0],                         // eye position
                [-0.57735027, -0.57735027, -0.57735027], // direction toward origin (normalized)
                [0.0, 1.0, 0.0],                         // up vector
                gpu_controller.read_surface_config(|sc| sc.width as f32 / sc.height as f32)?,
                45.0,  // FOV
                1.0,   // near plane - try 1.0 instead
                100.0, // far plane
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
                width: 1920,
                height: 1080,
                title: "Isotope".to_string(),
            },
        ) {
            self.window = Some(rendering_window);
            self.isotope.photon =
                match Renderer::new_defered_3d(self.isotope.gpu_controller.clone()) {
                    Ok(photon) => photon,
                    Err(err) => {
                        error!("Failed to create photon renderer: {}", err);
                        panic!();
                    }
                };
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

                            // Display on the surface
                            surface_texture.present();
                        }
                    }
                    WindowEvent::Resized(new_size) => {
                        if self
                            .isotope
                            .gpu_controller
                            .write_surface_config(|sc| {
                                sc.width = new_size.width;
                                sc.height = new_size.height;
                            })
                            .is_ok()
                        {
                            self.isotope
                                .gpu_controller
                                .configure_surface(&window.surface);
                            debug!("Surface resized to {}x{}", new_size.width, new_size.height);
                        } else {
                            error!("Failed to resize surface");
                        }

                        self.isotope
                            .photon
                            .resize((new_size.width, new_size.height));

                        // TEMP
                        self.isotope.camera = Camera::new_perspective_3d(
                            self.isotope.gpu_controller.clone(),
                            [5.0, 5.0, 5.0],                         // eye position
                            [-0.57735027, -0.57735027, -0.57735027], // direction toward origin (normalized)
                            [0.0, 1.0, 0.0],                         // up vector
                            self.isotope
                                .gpu_controller
                                .read_surface_config(|sc| sc.width as f32 / sc.height as f32)
                                .unwrap_or_else(|_| 640.0 / 480.0),
                            45.0,  // FOV
                            1.0,   // near plane - try 1.0 instead
                            100.0, // far plane
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}
