use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use anyhow::Result;
use boson::{
    BosonAdded, Linkable,
    boson_math::calculate_center_of_mass,
    solver::{
        basic_impulse_solver::BasicImpulseSolver,
        position_solver::PositionSolver,
        // rotational_impulse_solver::RotationalImpulseSolver,
    },
};
use cgmath::{Point3, Vector3};
use compound::Entity;
use debugger::Debugger;
use gpu_utils::GpuController;
use log::*;
use photon::{PhotonManager, renderer::camera::PhotonCamera};
use wgpu::{CommandEncoder, RenderPass};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
};

// Publicly exposed types
pub use boson::collider::ColliderBuilder;
pub use boson::{
    Boson, BosonBody, BosonObject,
    collider::Collider,
    particle_system::{InitialState, ParticleSysytem},
    rigid_body::RigidBody,
    static_collider::StaticCollider,
};
pub use compound::Compound;
pub use element::Element;
pub use element::asset_manager::AssetManager;
pub use element::model::Model;
pub use impulse::{ImpulseManager, KeyIsPressed};
pub use photon::instancer::*;
pub use photon::renderer::{
    camera::{Camera3D, CameraController},
    lights::light::{Color, Light},
};
pub use photon::window::WindowController;
pub use state::IsotopeState;
pub use std::any::Any;
pub use transform::Transform;
pub use winit::{dpi::PhysicalSize, keyboard::KeyCode, window::CursorGrabMode};

pub type InitCallback = fn(&mut Isotope);
pub type UpdateCallback = fn(&mut Compound, &mut AssetManager, &Instant, &Instant);

pub const DEFAULT_TICK_RATE: Duration = Duration::from_micros(50);

mod boson;
pub mod compound;
pub mod debugger;
mod element;
mod gpu_utils;
mod impulse;
mod photon;
mod state;
mod transform;

/// Main struct for the game engine app
#[derive(Debug)]
pub struct Isotope {
    // GPU
    pub gpu_controller: Arc<GpuController>,

    // Managing assets
    pub asset_manager: Arc<RwLock<AssetManager>>,

    // Window and Rendering
    photon: Option<PhotonManager>,

    // User Input
    impulse: ImpulseManager,

    // Keeping User defined variables
    state: Option<Arc<RwLock<dyn IsotopeState>>>,

    // ECS to tie everything together
    compound: Arc<RwLock<Compound>>,

    // Isotope start function
    init_callback: InitCallback,

    // Isotope update function
    update_callback: UpdateCallback,

    // Delta for updating
    pub delta: Instant,
    pub device_delta_t: Instant,
    pub window_delta_t: Instant,
    pub t: Arc<Instant>,

    // For physics
    boson: Arc<RwLock<Boson>>,
    boson_thread: Option<JoinHandle<()>>,

    // Bool and thread handle for multithreading
    running: Arc<RwLock<bool>>,
    state_thread_running: Arc<RwLock<bool>>,
    state_thread: Option<JoinHandle<()>>,
    tick_rate: Duration,
}

pub fn new_isotope(
    init_callback: InitCallback,
    update_callback: UpdateCallback,
) -> Result<Isotope> {
    info!("Creating Gpu Controller");
    let gpu_controller = Arc::new(GpuController::new()?);

    info!("Creating ECS system");
    let compound = Arc::new(RwLock::new(Compound::new()));

    info!("Starting Boson Engine");
    let boson = Arc::new(RwLock::new(Boson::new(gpu_controller.clone())));

    info!("Starting Asset Manager");
    let asset_manager = Arc::new(RwLock::new(AssetManager::new(gpu_controller.clone())));

    Ok(Isotope {
        gpu_controller,
        asset_manager,
        photon: None,
        impulse: ImpulseManager::default(),
        state: None,
        compound,
        init_callback,
        update_callback,
        delta: Instant::now(),
        device_delta_t: Instant::now(),
        window_delta_t: Instant::now(),
        t: Arc::new(Instant::now()),
        running: Arc::new(RwLock::new(false)),
        state_thread_running: Arc::new(RwLock::new(false)),
        tick_rate: DEFAULT_TICK_RATE,
        state_thread: None,
        boson,
        boson_thread: None,
    })
}

impl Isotope {
    /// This is where starting up every part of the engine happens all tied together with the ecs
    fn initialize(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        // Initialize Photon rendering engine with a window clone for the window controller
        let photon_manager = PhotonManager::new(event_loop, self.gpu_controller.clone())?;
        let window_clone = photon_manager.window.window.clone();
        self.photon = Some(photon_manager);

        // Add a window manager to the ecs for window controll if the user needs it
        if let Ok(mut compund) = self.compound.write() {
            let window_controller = compund.create_entity();

            compund.add_molecule(window_controller, WindowController::new(window_clone));
        }

        // Create references to all the necessary parts that boson needs
        let boson = self.boson.clone();
        let running = self.running.clone();
        let compound = self.compound.clone();

        // Initialize Boson Physics Engine
        self.boson_thread = Some(std::thread::spawn(move || {
            // Wait until The engine starts
            loop {
                if let Ok(running) = running.read() {
                    if *running {
                        break;
                    }
                }
            }

            info!("Starting Boson Thread");

            // Temp
            if let Ok(mut boson) = boson.write() {
                boson.add_solver(PositionSolver);
                boson.add_solver(BasicImpulseSolver);
            }

            let mut delta_t = Instant::now();
            loop {
                // Enable Boson debugging if toggled
                if let Ok(compound) = compound.read() {
                    compound.for_each_molecule_mut(
                        |_entity, debugger: &mut Debugger| match debugger {
                            Debugger::Boson | Debugger::ModelBoson => {
                                if let Ok(mut boson) = boson.write() {
                                    boson.set_debugger(|debugging| {
                                        *debugging = true;
                                    });
                                }

                                debugger.set_activated();
                            }
                            Debugger::None => {
                                if let Ok(mut boson) = boson.write() {
                                    boson.set_debugger(|debugging| {
                                        *debugging = false;
                                    });
                                }
                            }
                            _ => {}
                        },
                    );
                }

                if let Ok(mut boson) = boson.write() {
                    boson.step(&delta_t);
                }

                // Update delta_t
                delta_t = Instant::now();

                if let Ok(compound) = compound.read() {
                    // Update all the transforms
                    compound.for_each_duo_mut(
                        |_entity, transform: &mut Transform, boson_object: &mut BosonObject| {
                            match boson_object.access(|object| {
                                transform.position = object.get_position();
                                transform.orientation = object.get_orientation();
                            }) {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("Failed to Access Boson Object due to: {}", err);
                                }
                            }
                        },
                    );

                    // Add any new boson objects
                    if let Ok(mut boson) = boson.write() {
                        // TODO: Fix this stupidity
                        let mut objects_to_add = Vec::new();
                        let mut entities_to_add = Vec::new();

                        compound.for_each_duo_without_mut::<_, _, BosonAdded, _>(
                            |entity, object: &mut BosonObject, model: &mut Model| {
                                entities_to_add.push(entity);
                                objects_to_add.push(object.clone());

                                // Shifting the center of mass of the model to fit with the boson physics model
                                let center_of_mass = calculate_center_of_mass(model);
                                info!(
                                    "Center of mass: {:#?}\nMoving origin to center",
                                    center_of_mass
                                );

                                model.meshes.iter_mut().for_each(|mesh| {
                                    mesh.shift_vertices(|model_vertex| {
                                        model_vertex.position =
                                            (Vector3::from(model_vertex.position) - center_of_mass)
                                                .into();
                                    });
                                });

                                // Link the boson objects instancer to the model if it contains one
                                model.link_boson(object);
                            },
                        );

                        entities_to_add.into_iter().for_each(|entity| {
                            compound.add_molecule(entity, BosonAdded);
                            debug!("Added To Boson: {}", entity);
                        });

                        for object in objects_to_add {
                            boson.add_dynamic_object(object);
                        }
                    }
                }

                if let Ok(running) = running.read() {
                    if !*running {
                        break;
                    }
                }
            }
        }));

        (self.init_callback)(self);

        // Start the running checker
        if let Ok(mut running) = self.running.write() {
            *running = true;
        }

        Ok(())
    }

    /// Change the update callback
    pub fn set_update_callback(&mut self, update_callback: UpdateCallback) {
        self.update_callback = update_callback;
    }

    /// Change the tick rate of the engine
    pub fn set_tick_rate(&mut self, tick_rate: Duration) {
        self.tick_rate = tick_rate;
    }

    /// Add a state to the isotope engine if empty or replace it if occupied
    pub fn set_state<S: IsotopeState + 'static>(&mut self, state: S) {
        // Wait until Isotope has started

        // If there is a state thread running stop it before changing state
        if let Some(state_thread) = self.state_thread.take() {
            // Stop the thread, join it and then start it again when the state has been replaced
            if let Ok(mut running) = self.state_thread_running.write() {
                *running = false;
            }

            _ = state_thread.join();
        }

        if let Ok(mut running) = self.state_thread_running.write() {
            *running = true;
        }

        let new_state = Arc::new(RwLock::new(state));
        self.state.replace(new_state.clone());

        let compound_clone = self.compound.clone();
        let asset_manager_clone = self.asset_manager.clone();
        let t_clone = self.t.clone();
        let state_running_clone = self.state_thread_running.clone();
        let running_clone = self.running.clone();
        let tick_rate_clone = self.tick_rate.clone();

        // Run the states initialization function
        if let Ok(mut state) = new_state.write() {
            if let Ok(mut compound) = compound_clone.write() {
                if let Ok(mut asset_manager) = asset_manager_clone.write() {
                    let dt = self.delta.elapsed().as_secs_f32();
                    let t = t_clone.elapsed().as_secs_f32();
                    state.init(&mut compound, &mut asset_manager, dt, t);
                }
            }
        }

        info!("Added Game State");
        info!("Starting State Update Thread");

        // Start an update thread that will run however fast it feels like
        // And calls the update function on the game state
        self.state_thread = Some(thread::spawn(move || {
            info!("Running State Thread");
            let mut isotope_running = false;
            while !isotope_running {
                debug!("Isotope not running");
                if let Ok(running) = running_clone.read() {
                    isotope_running = *running;
                }
            }
            debug!("Isotope Running");

            let mut delta_t = Instant::now();
            loop {
                if let Ok(mut state) = new_state.write() {
                    if let Ok(mut compound) = compound_clone.write() {
                        if let Ok(mut asset_manager) = asset_manager_clone.write() {
                            let dt = delta_t.elapsed().as_secs_f32();
                            let t = t_clone.elapsed().as_secs_f32();
                            state.update(&mut compound, &mut asset_manager, dt, t);
                        }
                    }
                }

                // update delta_t
                delta_t = Instant::now();

                if let Ok(running) = state_running_clone.read() {
                    if !*running {
                        warn!("State Thread not running");
                        break;
                    }
                }

                // Sleep for a little so that the rest of isotope can catch up
                std::thread::sleep(tick_rate_clone);
            }
        }));
    }
}

impl ApplicationHandler for Isotope {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        info!("Isotope Resumed");

        match self.initialize(event_loop) {
            Ok(()) => {}
            Err(err) => error!("Failed to initialize Isotope: {err}"),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Ok(mut compound) = self.compound.write() {
            if let Ok(mut asset_manager) = self.asset_manager.write() {
                (self.update_callback)(&mut compound, &mut asset_manager, &self.delta, &self.t);
            }
        }

        // Check for any cameras that have been added
        if let Ok(compound) = self.compound.write() {
            let mut camera: Option<Entity> = None;
            compound.for_each_duo_without::<_, _, PhotonCamera, _>(
                |entity, _camera: &Camera3D, _camera_controller: &CameraController| {
                    camera = Some(entity);
                },
            );

            if let Some(camera) = camera {
                info!("Adding Camera");
                compound.add_molecule(
                    camera,
                    PhotonCamera::create_new_camera_3d(
                        self.gpu_controller.clone(),
                        Point3 {
                            x: 10.0,
                            y: 10.0,
                            z: 10.0,
                        },
                        Vector3 {
                            x: -5.0,
                            y: -5.0,
                            z: -5.0,
                        },
                        Vector3::unit_y(),
                        self.gpu_controller.surface_configuration().width as f32
                            / self.gpu_controller.surface_configuration().height as f32,
                        90.0,
                        0.1,
                        100.0,
                    ),
                );

                // Now set the aspect ratio of the camera controller
                compound.for_each_molecule_mut(
                    |entity, camera_controller: &mut CameraController| {
                        if entity == camera {
                            camera_controller.set_aspect(
                                self.gpu_controller.surface_configuration().width as f32
                                    / self.gpu_controller.surface_configuration().height as f32,
                            );
                        }
                    },
                );
            }

            // Check for cameras without a camera controller
            let mut camera: Option<Entity> = None;
            compound.for_each_molecule_without::<_, PhotonCamera, _>(
                |entity, _camera: &Camera3D| {
                    camera = Some(entity);
                },
            );

            if let Some(camera) = camera {
                info!("Adding Camera");
                compound.add_molecule(
                    camera,
                    PhotonCamera::create_new_camera_3d(
                        self.gpu_controller.clone(),
                        Point3 {
                            x: 10.0,
                            y: 10.0,
                            z: 10.0,
                        },
                        Vector3 {
                            x: -5.0,
                            y: -5.0,
                            z: -5.0,
                        },
                        Vector3::unit_y(),
                        self.gpu_controller.surface_configuration().width as f32
                            / self.gpu_controller.surface_configuration().height as f32,
                        90.0,
                        0.1,
                        100.0,
                    ),
                );
            }
        }

        if unsafe { self.photon.as_ref().unwrap_unchecked().window().id() } == window_id {
            match event {
                WindowEvent::CloseRequested => {
                    info!("Shutting Down Isotope");

                    // Change running to false
                    if let Ok(mut running) = self.running.write() {
                        *running = false;
                    }

                    if let Some(thread) = self.state_thread.take() {
                        _ = thread.join();
                    }

                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    if let Some(photon) = &mut self.photon {
                        // Get all the lights in the scene
                        let mut lights: Vec<Light> = Vec::new();

                        // TODO: Fix this rendering to prevent deadlock
                        if let Ok(compound) = self.compound.read() {
                            compound.for_each_molecule(|_entity, light: &Light| {
                                lights.push(*light);
                            });

                            compound.for_each_molecule(
                                |_entity, debugger: &Debugger| match debugger {
                                    Debugger::None => {
                                        photon.renderer.debugging = false;
                                    }
                                    _ => {
                                        photon.renderer.debugging = true;
                                    }
                                },
                            );

                            photon.renderer.update_lights(&lights);

                            // ================ Camera Updated ================

                            // TODO: Make the camera controller update the Transform instead of the camera directly

                            // Update any camera that has a transform
                            compound.for_each_duo_without_mut::<_, _, CameraController, _>(
                                |_entity, camera: &mut PhotonCamera, transform: &mut Transform| {
                                    camera.link_transform(transform);
                                },
                            );

                            // Update any camera that has a controller
                            compound.for_each_duo_without_mut::<_, _, Transform, _>(
                                |_entity,
                                 camera: &mut PhotonCamera,
                                 camera_controller: &mut CameraController| {
                                    camera.link_cam_controller(camera_controller);
                                },
                            );
                            // ================ Camera Updated ================

                            // Render all the models
                            compound.for_each_molecule(|_entity, camera: &PhotonCamera| {
                                match photon.renderer.render(
                                    &photon.window.surface,
                                    camera,
                                    // Pre compute passes
                                    |encoder: &mut CommandEncoder| {
                                        if let Ok(mut boson) = self.boson.write() {
                                            boson.update_instances(encoder);
                                        }
                                    },
                                    // Main Render Pass
                                    |render_pass: &mut RenderPass| {
                                        // Update all the transform buffers of the models
                                        compound.for_each_duo_mut(
                                            |_entity, model: &mut Model, transform: &mut Transform| {
                                                model.link_transform(transform);
                                            },
                                        );

                                        // Render all the models (with and without transform)
                                        compound.for_each_molecule_mut(
                                            |_entity, model: &mut Model| {
                                                model.render(render_pass, camera);
                                            },
                                        );
                                    },
                                    // Debugging Render pass
                                    |debug_render_pass: &mut RenderPass| {
                                        compound.for_each_molecule_mut(
                                            |_entity, model: &mut Model| unsafe {
                                                model.debug_render(debug_render_pass, camera);
                                            },
                                        );

                                        if let Ok(boson) = self.boson.write() {
                                            boson.debug_render(debug_render_pass);
                                        }
                                    },
                                ) {
                                    Ok(_) => {}
                                    Err(err) => error!("Rendering failed with Error: {}", err),
                                }
                            });
                        }
                    }
                }
                WindowEvent::Resized(new_size) => {
                    if let Some(photon) = &mut self.photon {
                        photon.resize(new_size);
                    }

                    // Update the camera if there is one
                    if let Ok(compound) = self.compound.write() {
                        // Update cameras without the controller
                        compound.for_each_molecule_without_mut::<_, CameraController, _>(
                            |_entity, camera: &mut PhotonCamera| {
                                camera.set_aspect(new_size.width as f32 / new_size.height as f32);
                            },
                        );

                        // Update cameras with the controller
                        compound.for_each_molecule_mut(
                            |_entity, camera_controller: &mut CameraController| {
                                camera_controller
                                    .set_aspect(new_size.width as f32 / new_size.height as f32);
                            },
                        );

                        // Update the window controller with the new size
                        compound.for_each_molecule_mut(
                            |_entity, window_controller: &mut WindowController| {
                                window_controller.resize(|size| {
                                    *size = new_size;
                                });
                            },
                        );
                    }
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    // Run the callback of the Impulse Manager
                    match event {
                        KeyEvent {
                            physical_key,
                            state,
                            ..
                        } => match state {
                            ElementState::Pressed => {
                                // Game state update
                                if let Some(state) = self.state.as_mut() {
                                    if let Ok(mut state) = state.write() {
                                        if let Ok(mut compound) = self.compound.write() {
                                            if let Ok(mut asset_manager) =
                                                self.asset_manager.write()
                                            {
                                                match physical_key {
                                                    winit::keyboard::PhysicalKey::Code(code) => {
                                                        state.key_is_pressed(
                                                            &mut compound,
                                                            &mut asset_manager,
                                                            code,
                                                            self.window_delta_t
                                                                .elapsed()
                                                                .as_secs_f32(),
                                                            self.t.elapsed().as_secs_f32(),
                                                        );
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }

                                // Isotope update
                                if let Some(callback) = self.impulse.key_is_pressed {
                                    match physical_key {
                                        winit::keyboard::PhysicalKey::Code(code) => {
                                            callback(code, self);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            ElementState::Released => {
                                // Game state update
                                if let Some(state) = self.state.as_mut() {
                                    if let Ok(mut state) = state.write() {
                                        if let Ok(mut compound) = self.compound.write() {
                                            if let Ok(mut asset_manager) =
                                                self.asset_manager.write()
                                            {
                                                match physical_key {
                                                    winit::keyboard::PhysicalKey::Code(code) => {
                                                        state.key_is_released(
                                                            &mut compound,
                                                            &mut asset_manager,
                                                            code,
                                                            self.window_delta_t
                                                                .elapsed()
                                                                .as_secs_f32(),
                                                            self.t.elapsed().as_secs_f32(),
                                                        );
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }

                                // Isotope update
                                if let Some(callback) = self.impulse.key_is_released {
                                    match physical_key {
                                        winit::keyboard::PhysicalKey::Code(code) => {
                                            callback(code, self);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        },
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    // Game state update
                    if let Some(state) = self.state.as_mut() {
                        if let Ok(mut state) = state.write() {
                            if let Ok(mut compound) = self.compound.write() {
                                if let Ok(mut asset_manager) = self.asset_manager.write() {
                                    state.cursor_moved(
                                        &mut compound,
                                        &mut asset_manager,
                                        position,
                                        self.window_delta_t.elapsed().as_secs_f32(),
                                        self.t.elapsed().as_secs_f32(),
                                    );
                                }
                            }
                        }
                    }

                    // Run the callback of the Impulse manager for cursor movement
                    if let Some(callback) = self.impulse.cursor_moved {
                        callback(position, self);
                    }
                }
                _ => {}
            }
        }

        // Update delta for the next go-around
        self.window_delta_t = Instant::now();
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        // Run the fixed update
        // TODO: extract into thread
        if let Ok(mut compound) = self.compound.write() {
            if let Ok(mut asset_manager) = self.asset_manager.write() {
                (self.update_callback)(&mut compound, &mut asset_manager, &self.delta, &self.t);
            }
        }

        match event {
            DeviceEvent::MouseMotion { delta } => {
                // Game state update
                if let Some(state) = self.state.as_mut() {
                    if let Ok(mut state) = state.write() {
                        if let Ok(mut compound) = self.compound.write() {
                            if let Ok(mut asset_manager) = self.asset_manager.write() {
                                state.mouse_is_moved(
                                    &mut compound,
                                    &mut asset_manager,
                                    delta,
                                    self.device_delta_t.elapsed().as_secs_f32(),
                                    self.t.elapsed().as_secs_f32(),
                                );
                            }
                        }
                    }
                }

                // Run the Isotope callback for mouse movement
                if let Some(callback) = self.impulse.mouse_is_moved {
                    callback(delta, self);
                }
            }
            _ => {}
        }

        self.device_delta_t = Instant::now();
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Safety: This is safe because we know photo exists at this point
        unsafe {
            self.photon
                .as_ref()
                .unwrap_unchecked()
                .window()
                .request_redraw()
        };
    }
}

pub fn start_isotope(isotope: &mut Isotope) -> Result<()> {
    pretty_env_logger::init();
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);
    _ = event_loop.run_app(isotope);

    Ok(())
}
