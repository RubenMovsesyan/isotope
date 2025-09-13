use std::{
    sync::{Arc, RwLock},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use anyhow::Result;
pub use asset_server::AssetServer;
pub use compound::Compound;
use gpu_controller::{
    CompositeAlphaMode, GpuController, Mesh, PresentMode, SurfaceConfiguration, TextureFormat,
    TextureUsages, Vertex,
};
pub use log::*;
use matter_vault::MatterVault;
pub use model::Model;
pub use photon::Light;
use photon::{
    camera::Camera,
    renderer::{Renderer, defered_renderer::DeferedRenderer3D},
};
use rendering_window::{RenderingWindow, WindowInitializer};
use smol::block_on;
pub use state::IsotopeState;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalSize, Size},
    event::WindowEvent,
    window::Window,
};

pub const ISOTOPE_DEFAULT_TICK_RATE: Duration = Duration::from_micros(50);

mod asset_server;
mod material;
mod model;
mod rendering_window;
mod state;
mod texture;

pub struct Isotope {
    // GPU
    gpu_controller: Arc<GpuController>,

    // Asset Server
    asset_server: Arc<AssetServer>,

    // Rendering
    photon: Renderer,

    // Entity component system
    compound: Arc<Compound>,

    // State for interacting with the engine
    state: Arc<RwLock<dyn IsotopeState>>,

    // ============== Multi-Threading ==============
    state_thread: (Arc<RwLock<bool>>, JoinHandle<()>),

    // Timing
    time: Arc<Instant>,
    tick_rate: Duration,

    // Master Running state
    running: Arc<RwLock<bool>>,

    // Temp
    camera: Camera,
}

impl Isotope {
    pub fn new<I>(gpu_controller: Arc<GpuController>, mut state: I) -> Result<Self>
    where
        I: IsotopeState,
    {
        let photon = Renderer::new_defered_3d(gpu_controller.clone())?;
        let asset_server = Arc::new(AssetServer::new(
            Arc::new(MatterVault::new()),
            gpu_controller.clone(),
        ));
        let compound = Arc::new(Compound::new());
        // TEMP camera
        let camera = Camera::new_perspective_3d(
            gpu_controller.clone(),
            [5.0, 5.0, 5.0],                         // eye position
            [-0.57735027, -0.57735027, -0.57735027], // direction toward origin (normalized)
            [0.0, 1.0, 0.0],                         // up vector
            gpu_controller.read_surface_config(|sc| sc.width as f32 / sc.height as f32)?,
            45.0,  // FOV
            1.0,   // near plane - try 1.0 instead
            100.0, // far plane
        );
        let running = Arc::new(RwLock::new(false));
        let time = Arc::new(Instant::now());
        let tick_rate = ISOTOPE_DEFAULT_TICK_RATE;

        // Initialize the game state and start the update thread
        state.init(&compound, &asset_server);
        let state = Arc::new(RwLock::new(state));
        let state_running = Arc::new(RwLock::new(true));

        let state_ecs = compound.clone();
        let state_asset_server = asset_server.clone();
        let state_isotope_running = running.clone();
        let state_state = state.clone();
        let state_time = time.clone();
        let state_state_running = state_running.clone();
        let state_tick_rate = tick_rate.clone();
        let state_thread_handle = std::thread::spawn(move || {
            info!("Running State Update Thread");

            if let Ok(running) = state_isotope_running.read() {
                if !*running {
                    debug!("Isotope not running. Waiting for initialization...");
                }
            }

            // Wait for Isotope to start running
            while let Ok(running) = state_isotope_running.read() {
                if *running {
                    break;
                }
            }

            debug!("Isotope has started running! Starting State Thread...");

            let mut delta_t = Instant::now();

            loop {
                if let Ok(mut state) = state_state.write() {
                    let dt = delta_t.elapsed().as_secs_f32();
                    let t = state_time.elapsed().as_secs_f32();

                    state.update(&state_ecs, &state_asset_server, dt, t);
                }

                delta_t = Instant::now();

                if let Ok(running) = state_state_running.read() {
                    if !*running {
                        warn!("State Update Thread Exiting....");
                        break;
                    }
                }

                // TODO: make tick rate dependent on how long update takes
                // Sleep for a little so that the rest of Isotope can catch up
                std::thread::sleep(state_tick_rate);
            }
        });

        Ok(Self {
            photon,
            asset_server,
            compound,
            camera,
            state,
            running,
            time,
            tick_rate,
            gpu_controller,
            state_thread: (state_running, state_thread_handle),
        })
    }
}

pub struct IsotopeApplication {
    window: Option<RenderingWindow>,
    isotope: Isotope,
}

impl IsotopeApplication {
    pub fn new<I>(state: I) -> Result<Self>
    where
        I: IsotopeState,
    {
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
            isotope: Isotope::new(gpu_controller, state)?,
        })
    }
}

impl ApplicationHandler for IsotopeApplication {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        info!("Isotope Resumed");

        // TEMP ======

        // self.isotope.compound.spawn((Light::new(
        //     [-10.0, 0.0, 0.0],
        //     [0.0, 0.0, 0.0],
        //     [1.0, 1.0, 1.0],
        //     10.0,
        // ),));

        // self.isotope.compound.spawn((Light::new(
        //     [0.0, 10.0, 0.0],
        //     [0.0, 0.0, 0.0],
        //     [1.0, 1.0, 1.0],
        //     10.0,
        // ),));
        // TEMP ======

        // Initialize the rendering window with Photon
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

        _ = self.isotope.running.write().and_then(|mut running| {
            *running = true;
            Ok(())
        });
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
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
                        info!("Shutting Down Isotope...");

                        _ = self.isotope.running.write().and_then(|mut running| {
                            *running = false;
                            Ok(())
                        });

                        event_loop.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        if let Ok(surface_texture) = window.surface.get_current_texture() {
                            // TEMP
                            // let time = self.isotope.time.elapsed().as_secs_f32();

                            // self.isotope
                            //     .compound
                            //     .iter_mut_mol(|_entity, light: &mut Light| {
                            //         light.pos(|position| {
                            //             *position =
                            //                 [5.0 * f32::cos(time), 2.0, 5.0 * f32::sin(time)];
                            //         });
                            //     });

                            // Update the lights if there are any modified lights
                            let mut lights_changed = false;
                            self.isotope
                                .compound
                                .iter_mol_mod(|_entity, _light: &Light| {
                                    lights_changed = true;
                                    return;
                                });

                            if lights_changed {
                                let mut lights = Vec::new();
                                self.isotope.compound.iter_mol(|_entity, light: &Light| {
                                    lights.push(light.clone());
                                });
                                self.isotope.photon.update_lights(&lights);
                            }

                            self.isotope.photon.render(
                                &self.isotope.camera,
                                &surface_texture.texture,
                                |render_pass| {
                                    // Temp
                                    self.isotope.compound.iter_mol(|_entity, model: &Model| {
                                        model.render(render_pass);
                                    });
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
