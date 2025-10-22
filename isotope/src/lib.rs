use std::{
    sync::{Arc, RwLock},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use anyhow::Result;
pub use asset_server::AssetServer;
use boson::Boson;
pub use boson::{BosonBody, BosonObject, PointMass, RigidBody, StaticCollider};
pub use cgmath::*;
pub use compound::Compound;
pub use compound::Entity;
pub use elements::*;
pub use gpu_controller::Instance;
use gpu_controller::{
    CompositeAlphaMode, Features, GpuController, PresentMode, SurfaceConfiguration, TextureFormat,
    TextureUsages,
};
pub use log::*;
use matter_vault::MatterVault;
pub use model::Model;
pub use photon::Light;
use photon::renderer::Renderer;
use physics::BosonCompat;
use rendering_window::{RenderingWindow, WindowInitializer};
use smol::block_on;
pub use state::IsotopeState;
pub use winit::keyboard::KeyCode;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, ElementState, KeyEvent, WindowEvent},
};

pub const ISOTOPE_DEFAULT_TICK_RATE: Duration = Duration::from_micros(50);

mod asset_server;
mod elements;
mod material;
mod model;
mod physics;
mod rendering_window;
mod state;
mod texture;

// Structs for bookkeeping in ecs
struct BosonCompliant;

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

    // Physics Engine
    boson: Arc<RwLock<Boson>>,

    // ============== Multi-Threading ==============
    state_thread: (Arc<RwLock<bool>>, JoinHandle<()>),

    // Timing
    time: Arc<Instant>,
    tick_rate: Duration,

    // Master Running state
    running: Arc<RwLock<bool>>,
}

impl Isotope {
    pub fn new<I>(gpu_controller: Arc<GpuController>, mut state: I) -> Result<Self>
    where
        I: IsotopeState,
    {
        let asset_server = Arc::new(AssetServer::new(
            Arc::new(MatterVault::new()),
            gpu_controller.clone(),
        ));
        let photon = Renderer::new_defered_3d(gpu_controller.clone())?;
        let compound = Arc::new(Compound::new());
        let running = Arc::new(RwLock::new(false));
        let time = Arc::new(Instant::now());
        let tick_rate = ISOTOPE_DEFAULT_TICK_RATE;

        // Initialize the physics engine
        let boson = Arc::new(RwLock::new(Boson::new(gpu_controller.clone())));

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
        let state_boson = boson.clone();
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

            let mut last_frame_time = Instant::now();

            loop {
                let now = Instant::now();
                let dt = now.duration_since(last_frame_time).as_secs_f32();
                last_frame_time = now;

                if let Ok(mut state) = state_state.write() {
                    let t = state_time.elapsed().as_secs_f32();

                    state.update(&state_ecs, &state_asset_server, dt, t);
                }

                // Add any new boson objects
                {
                    let mut to_add_as_boson_compliant: Vec<Entity> = Vec::new();

                    state_ecs.iter_without_mol_mod::<BosonCompliant, _, _>(
                        |entity, boson_object: &BosonObject| {
                            if let Ok(mut boson) = state_boson.write() {
                                info!("Adding Boson Object");
                                boson.add_object(boson_object);
                                to_add_as_boson_compliant.push(entity);
                            }
                        },
                    );

                    for entity in to_add_as_boson_compliant.into_iter() {
                        state_ecs.add_molecule(entity, BosonCompliant);
                        info!("Added Boson Object");
                    }
                }

                // Update boson objects with any changed transforms first
                {
                    state_ecs.iter_mut_duo_mod(
                        |_entity, transform: &mut Transform3D, boson_object: &mut BosonObject| {
                            debug!("Writing Transform");
                            boson_object.write_transform(transform);
                        },
                    );
                }

                // Update transforms with the new boson values
                {
                    // Unmodified so the transform update is not triggered at the next goaround
                    state_ecs.iter_mut_duo_unmod(
                        |_entity, transform: &mut Transform3D, boson_object: &mut BosonObject| {
                            boson_object.read_position(|boson_pos| {
                                transform.position(|transform_pos| {
                                    transform_pos.x = boson_pos.x as f32;
                                    transform_pos.y = boson_pos.y as f32;
                                    transform_pos.z = boson_pos.z as f32;
                                })
                            })
                        },
                    );
                }

                // Run the instancer on any objects that have an instancer
                {
                    state_ecs.iter_mut_duo(
                        |_entity, model: &mut Model, instancer: &mut Instancer| {
                            model.apply_instancer(instancer);
                        },
                    )
                }

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
            state,
            boson,
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
            Some(Features::MAPPABLE_PRIMARY_BUFFERS),
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
            // Add the window controller to Isotope
            self.isotope
                .compound
                .spawn((WindowController::new(rendering_window.window.clone()),));

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
                            // Update the lights if there are any modified lights
                            {
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
                            }

                            // Update the camera if there are any modifications
                            {
                                self.isotope.compound.iter_mut_duo_mod(
                                    |_entity, transform: &mut Transform3D, camera: &mut Camera| {
                                        match camera {
                                            Camera::PerspectiveCamera3D(camera) => {
                                                camera.all(|eye, target, _, _, _, _, _| {
                                                    *eye = transform.position.into();

                                                    let forward = Vector3::new(0.0, 0.0, 1.0);

                                                    *target = transform
                                                        .rotation(|rot| *rot * forward)
                                                        .normalize();
                                                });
                                            }
                                        }
                                    },
                                );
                            }

                            // Update the model with the transform if it has been modified
                            {
                                // Update Modified Transforms
                                self.isotope.compound.iter_mut_duo_mod(
                                    |_entity, transform: &mut Transform3D, model: &mut Model| {
                                        model.set_transform(transform);
                                    },
                                );

                                // Update All BosonCompliant Objects
                                self.isotope.compound.iter_mut_trio_unmod(
                                    |_entity, transform: &mut Transform3D, model: &mut Model, _boson_compliant: &mut BosonCompliant| {
                                        model.set_transform(transform);
                                    }
                                );
                            }

                            // Render to the display
                            self.isotope.compound.iter_mol(|_entity, camera: &Camera| {
                                self.isotope.photon.render(
                                    camera,
                                    &surface_texture.texture,
                                    |render_pass| {
                                        // Temp
                                        self.isotope.compound.iter_mol(|_entity, model: &Model| {
                                            model.render(render_pass);
                                        });
                                    },
                                );
                            });

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

                        self.isotope
                            .compound
                            .iter_mut_mol(|_entity, camera: &mut Camera| match camera {
                                Camera::PerspectiveCamera3D(camera) => {
                                    camera.aspect(|aspect| {
                                        *aspect = self.isotope.gpu_controller.read_surface_config(|sc| {
                                            sc.width as f32 / sc.height as f32
                                        }).unwrap_or_else(|err| {
                                            warn!("Reading Surface Configuration Failed: {}, Continuing with aspect ration of 1.0...", err);
                                            1.0
                                        });
                                    });
                                }
                            });
                    }
                    WindowEvent::KeyboardInput { event, .. } => match event {
                        KeyEvent {
                            physical_key,
                            state,
                            ..
                        } => match state {
                            ElementState::Pressed => match physical_key {
                                winit::keyboard::PhysicalKey::Code(code) => {
                                    self.isotope.state.write().and_then(|mut state| {
                                        state.key_is_pressed(
                                            &self.isotope.compound,
                                            &self.isotope.asset_server,
                                            code,
                                            self.isotope.time.elapsed().as_secs_f32(),
                                        );
                                        Ok(())
                                    }).unwrap_or_else(|err| {
                                        warn!("Failed to update game state with key: {} continuing...", err);
                                    });
                                }
                                _ => {}
                            },
                            ElementState::Released => match physical_key {
                                winit::keyboard::PhysicalKey::Code(code) => {
                                    self.isotope.state.write().and_then(|mut state| {
                                        state.key_is_released(
                                            &self.isotope.compound,
                                            &self.isotope.asset_server,
                                            code,
                                            self.isotope.time.elapsed().as_secs_f32(),
                                        );
                                        Ok(())
                                    }).unwrap_or_else(|err| {
                                        warn!("Failed to update game state with key: {} continuing...", err);
                                    });
                                }
                                _ => {}
                            },
                        },
                    },
                    WindowEvent::CursorMoved { position, .. } => {
                        self.isotope.state.write().and_then(|mut state| {
                            state.cursor_moved(
                                &self.isotope.compound,
                                &self.isotope.asset_server,
                                position.into(),
                                self.isotope.time.elapsed().as_secs_f32(),
                            );
                            Ok(())
                        }).unwrap_or_else(|err| {
                            warn!("Failed to update game state with cursor position: {} continuing...", err);
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.isotope
                    .state
                    .write()
                    .and_then(|mut state| {
                        state.mouse_is_moved(
                            &self.isotope.compound,
                            &self.isotope.asset_server,
                            delta,
                            self.isotope.time.elapsed().as_secs_f32(),
                        );
                        Ok(())
                    })
                    .unwrap_or_else(|err| {
                        warn!(
                            "Failed to update game state with mouse movement: {} continuing...",
                            err
                        );
                    });
            }
            _ => {}
        }
    }
}
