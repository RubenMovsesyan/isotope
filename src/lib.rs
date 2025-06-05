use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
    time::Instant,
};

use anyhow::Result;
use boson::{
    BosonAdded, BosonDebugger, Linkable,
    boson_math::calculate_center_of_mass,
    solver::{
        basic_impulse_solver::BasicImpulseSolver,
        position_solver::PositionSolver,
        // rotational_impulse_solver::RotationalImpulseSolver,
    },
};
use cgmath::Vector3;
use compound::Compound;
use element::asset_manager::AssetManager;
use gpu_utils::GpuController;
use log::*;
use photon::PhotonManager;
use wgpu::{CommandEncoder, RenderPass};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::Window,
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
pub use element::Element;
pub use element::model::Model;
pub use impulse::{ImpulseManager, KeyIsPressed};
pub use photon::instancer::*;
pub use photon::renderer::{
    camera::PhotonCamera,
    lights::light::{Color, Light},
};
pub use state::IsotopeState;
pub use transform::Transform;
pub use winit::keyboard::KeyCode; // Temp

mod boson;
pub mod compound;
mod element;
mod gpu_utils;
mod impulse;
mod photon;
mod state;
mod transform;
mod utils;

/// Main struct for the game engine app
#[derive(Debug)]
pub struct Isotope {
    // GPU
    pub gpu_controller: Arc<GpuController>,

    // Managing assets
    pub asset_manager: Arc<RwLock<AssetManager>>,

    // Window and Rendering
    photon: Option<PhotonManager>,

    // Elements and components
    elements: Vec<Arc<dyn Element>>,

    // User Input
    impulse: ImpulseManager,

    // Keeping User defined variables
    state: Option<Arc<RwLock<dyn IsotopeState>>>,

    // ECS to tie everything together
    compound: Arc<RwLock<Compound>>,

    // Isotope start function
    init_callback: fn(&mut Self),

    // Isotope update function
    update_callback: fn(&mut Self),

    // Delta for updating
    pub delta: Instant,
    pub t: Arc<Instant>,

    // For physics
    boson: Arc<RwLock<Boson>>,
    boson_thread: Option<JoinHandle<()>>,

    // Bool and thread handle for multithreading
    running: Arc<RwLock<bool>>,
    thread_handle: Option<JoinHandle<()>>,

    // For enabling or disabling debugging
    debugging: bool,
}

pub fn new_isotope(
    init_callback: fn(&mut Isotope),
    update_callback: fn(&mut Isotope),
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
        elements: Vec::new(),
        impulse: ImpulseManager::default(),
        state: None,
        compound,
        init_callback,
        update_callback,
        delta: Instant::now(),
        t: Arc::new(Instant::now()),
        running: Arc::new(RwLock::new(false)),
        thread_handle: None,
        boson,
        boson_thread: None,
        debugging: false,
    })
}

impl Isotope {
    /// This is where starting up every part of the engine happens all tied together with the ecs
    fn initialize(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        // Initialize Photon rendering engine
        self.photon = Some(PhotonManager::new(event_loop, self.gpu_controller.clone())?);

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
                if let Ok(mut boson) = boson.write() {
                    boson.step(&delta_t);
                }

                // Update delta_t
                delta_t = Instant::now();

                if let Ok(compound) = compound.read() {
                    // Update all the transforms
                    compound.for_each_duo_mut(
                        |_entity, transform: &mut Transform, boson_object: &mut BosonObject| {
                            boson_object.access(|object| {
                                transform.position = object.get_position();
                                transform.orientation = object.get_orientation();
                            });
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
                                    mesh.with_write(|mesh| {
                                        mesh.shift_vertices(|model_vertex| {
                                            model_vertex.position =
                                                (Vector3::from(model_vertex.position)
                                                    - center_of_mass)
                                                    .into();
                                        });
                                    });
                                });
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

    // Set all debugging modes
    pub fn set_debbuging<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut bool),
    {
        callback(&mut self.debugging);

        // Enable debug rendering
        if let Some(photon) = self.photon.as_mut() {
            photon.set_debugger(|debugging| *debugging = self.debugging);
        }

        // Enable the boson debugger
        if let Ok(mut boson) = self.boson.write() {
            boson.set_debugger(if self.debugging {
                BosonDebugger::new_basic(self.gpu_controller.clone())
            } else {
                BosonDebugger::None
            });
        }
    }

    // Only change the model debugging mode
    pub fn set_model_debugging<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut bool),
    {
        if let Some(photon) = self.photon.as_mut() {
            photon.set_debugger(callback);
        }
    }

    // Only set boson debugging mode
    pub fn set_boson_debbuging<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut bool),
    {
        // Enable the boson debugger
        if let Ok(mut boson) = self.boson.write() {
            let mut debugging = match boson.boson_debugger {
                BosonDebugger::None => false,
                BosonDebugger::BasicDebugger { .. } => true,
            };

            callback(&mut debugging);

            boson.set_debugger(if debugging {
                BosonDebugger::new_basic(self.gpu_controller.clone())
            } else {
                BosonDebugger::None
            });
        }
    }

    pub fn modify_boson<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut Boson),
    {
        if let Ok(mut boson) = self.boson.write() {
            callback(&mut boson);
        }
    }

    /// Add Elements to isotope
    pub fn add_element(&mut self, element: Arc<dyn Element>) {
        self.elements.push(element);
    }

    /// Allows modifications to the ecs and updates the game based on those modifications
    pub fn ecs<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut Compound),
    {
        if let Ok(mut compound) = self.compound.write() {
            callback(&mut compound);
        }
    }

    /// For handling input
    pub fn impulse(&mut self) -> &mut ImpulseManager {
        &mut self.impulse
    }

    /// Access the the engine camera
    pub fn camera(&mut self) -> Option<&mut PhotonCamera> {
        Some(&mut self.photon.as_mut()?.renderer.camera)
    }

    /// Change the update callback
    pub fn set_update_callback(&mut self, update_callback: fn(&mut Isotope)) {
        self.update_callback = update_callback;
    }

    /// Add a state to the isotope engine if empty or replace it if occupied
    pub fn set_state<S: IsotopeState + 'static>(&mut self, state: S) {
        // If there is a state thread running stop it before changing state
        if let Some(state_thread) = self.thread_handle.take() {
            if let Ok(mut running) = self.running.write() {
                // Stop the thread, join it and then start it again when the state has been replaced
                *running = false;
                _ = state_thread.join();
                *running = true;
            }
        }

        self.state.replace(Arc::new(RwLock::new(state)));

        if let Some(state) = self.state.as_mut() {
            if let Ok(mut state) = state.write() {
                state.init(&self.t);
            }
        }

        info!("Added Game State");
        info!("Starting State Update Thread");

        let state_clone = unsafe { self.state.as_ref().unwrap_unchecked().clone() };
        let t_clone = self.t.clone();
        let running_clone = self.running.clone();
        let boson_clone = self.boson.clone();

        // Start an update thread that will run however fast it feels like
        self.thread_handle = Some(thread::spawn(move || {
            info!("Running Thread");

            if let Ok(mut boson) = boson_clone.write() {
                info!("Initializing Boson");
                if let Ok(mut state) = state_clone.write() {
                    state.init_boson(&mut boson);
                }

                // Temp
                boson.add_solver(PositionSolver);
                boson.add_solver(BasicImpulseSolver);
                // boson.add_solver(RotationalImpulseSolver);
            }

            let mut delta_t = Instant::now();
            loop {
                if let Ok(mut state) = state_clone.write() {
                    state.update(&delta_t, &t_clone);

                    // Handle Boson updates here
                    if let Ok(mut boson) = boson_clone.write() {
                        boson.step(&delta_t);
                    }
                }

                // update delta_t
                delta_t = Instant::now();

                if let Ok(running) = running_clone.read() {
                    if !*running {
                        break;
                    }
                }
            }
        }));
    }

    /// Immutable access to the state
    pub fn with_state<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&dyn IsotopeState) -> R,
    {
        let state_guard = self.state.as_ref()?.read().ok()?;
        Some(f(&*state_guard))
    }

    /// Mutable access to the state
    pub fn with_state_mut<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn IsotopeState) -> R,
    {
        let mut state_guard = self.state.as_ref()?.write().ok()?;
        Some(f(&mut *state_guard))
    }

    /// Immutable access to the typed state
    pub fn with_state_typed<S, F, R>(&self, f: F) -> Option<R>
    where
        S: 'static,
        F: FnOnce(&S) -> R,
    {
        self.with_state(|state| {
            let typed_state = state.as_any().downcast_ref::<S>()?;
            Some(f(typed_state))
        })?
    }

    /// Mutable access to the typed state
    pub fn with_state_typed_mut<S, F, R>(&self, f: F) -> Option<R>
    where
        S: 'static,
        F: FnOnce(&mut S) -> R,
    {
        self.with_state_mut(|state| {
            let typed_state = state.as_any_mut().downcast_mut::<S>()?;
            Some(f(typed_state))
        })?
    }

    /// Modifying window characteristics
    pub fn modify_window<F>(&self, callback: F)
    where
        F: FnOnce(&Window),
    {
        if let Some(photon) = &self.photon {
            callback(&photon.window.window);
        }
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
        // Run the fixed update
        // TODO: extract into thread
        (self.update_callback)(self);

        // Run the fixed update for the camera
        if let Some(camera) = self.camera() {
            // Store the camera pointer temporarily
            let camera_ptr = camera as *mut PhotonCamera;

            // Now access the state without the camera borrow active
            if let Some(state) = &self.state {
                if let Ok(mut state_guard) = state.write() {
                    // This is safe because we are ensuring no other references
                    // to the camera exist
                    unsafe {
                        state_guard.update_with_camera(&mut *camera_ptr, &self.delta, &self.t);
                    }
                }
            }
        }

        // Run the fixed update for the window
        if let Some(photon) = &self.photon {
            if let Some(state) = &self.state {
                if let Ok(mut state_guard) = state.write() {
                    state_guard.update_with_window(&photon.window.window, &self.delta, &self.t);
                }
            }
        }

        // Update delta for the next go-around
        self.delta = Instant::now();

        if unsafe { self.photon.as_ref().unwrap_unchecked().window().id() } == window_id {
            match event {
                WindowEvent::CloseRequested => {
                    info!("Shutting Down Isotope");

                    // Change running to false
                    if let Ok(mut running) = self.running.write() {
                        *running = false;
                    }

                    if let Some(thread) = self.thread_handle.take() {
                        _ = thread.join();
                    }

                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    if let Some(photon) = &mut self.photon {
                        // Get all the lights in the scene
                        let mut lights: Vec<Light> = Vec::new();

                        if let Ok(compound) = self.compound.read() {
                            compound.for_each_molecule(|_entity, light: &Light| {
                                lights.push(*light);
                            });
                        }

                        photon.renderer.update_lights(&lights);

                        // Render all the models
                        _ = photon.render(
                            |render_pass: &mut RenderPass| {
                                if let Ok(compound) = self.compound.read() {
                                    // Update all the transform buffers of the models
                                    compound.for_each_duo(
                                        |_entity, model: &Model, transform: &Transform| {
                                            model.link_transform(transform);
                                        },
                                    );

                                    compound.for_each_molecule_mut(|_entity, model: &mut Model| {
                                        model.render(render_pass);
                                    });
                                }
                            },
                            |encoder: &mut CommandEncoder| {
                                if let Ok(mut boson) = self.boson.write() {
                                    boson.update_instances(encoder);
                                }
                            },
                            &lights,
                            |render_pass: &mut RenderPass| {
                                if let Ok(compound) = self.compound.read() {
                                    compound.for_each_molecule_mut(
                                        |_entity, model: &mut Model| unsafe {
                                            model.debug_render(render_pass);
                                        },
                                    );
                                }

                                if let Ok(boson) = self.boson.write() {
                                    boson.debug_render(render_pass);
                                }
                            },
                        );
                    }
                }
                WindowEvent::Resized(new_size) => {
                    if let Some(photon) = &mut self.photon {
                        photon.resize(new_size);
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
                                        match physical_key {
                                            winit::keyboard::PhysicalKey::Code(code) => {
                                                state.key_is_pressed(code);
                                            }
                                            _ => {}
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
                                        match physical_key {
                                            winit::keyboard::PhysicalKey::Code(code) => {
                                                state.key_is_released(code);
                                            }
                                            _ => {}
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
                            state.cursor_moved(position);
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
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        // Run the fixed update
        // TODO: extract into thread
        (self.update_callback)(self);

        // Run the fixed update for the camera
        if let Some(camera) = self.camera() {
            // Store the camera pointer temporarily
            let camera_ptr = camera as *mut PhotonCamera;

            // Now access the state without the camera borrow active
            if let Some(state) = &self.state {
                if let Ok(mut state_guard) = state.write() {
                    // This is safe because we are ensuring no other references
                    // to the camera exist
                    unsafe {
                        state_guard.update_with_camera(&mut *camera_ptr, &self.delta, &self.t);
                    }
                }
            }
        }

        // Run the fixed update for the window
        if let Some(photon) = &self.photon {
            if let Some(state) = &self.state {
                if let Ok(mut state_guard) = state.write() {
                    state_guard.update_with_window(&photon.window.window, &self.delta, &self.t);
                }
            }
        }

        // Update delta for the next go-around
        self.delta = Instant::now();

        match event {
            DeviceEvent::MouseMotion { delta } => {
                // Game state update
                if let Some(state) = self.state.as_mut() {
                    if let Ok(mut state) = state.write() {
                        state.mouse_is_moved(delta);
                    }
                }

                // Run the Isotope callback for mouse movement
                if let Some(callback) = self.impulse.mouse_is_moved {
                    callback(delta, self);
                }
            }
            _ => {}
        }
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
