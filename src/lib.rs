use std::{
    fmt::Debug,
    path::Path,
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
    time::Instant,
};

use anyhow::Result;
use element::model::Model;
use gpu_utils::GpuController;
use log::*;
use photon::PhotonManager;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
};

// Publicly exposed types
pub use element::Element;
pub use impulse::{ImpulseManager, KeyIsPressed};
pub use photon::renderer::camera::PhotonCamera;
pub use state::IsotopeState;
pub use winit::keyboard::KeyCode;

mod compound;
mod element;
mod gpu_utils;
mod impulse;
mod photon;
mod state;
mod utils;

// Test struct
#[deprecated]
#[derive(Debug)]
pub struct TestElement {
    model: Model,
}

impl Element for TestElement {
    fn render(&self, render_pass: &mut wgpu::RenderPass) {
        self.model.render(render_pass);
    }
}

/// Main struct for the game engine app
#[derive(Debug)]
pub struct Isotope {
    // GPU
    gpu_controller: Arc<GpuController>,

    // Window and Rendering
    photon: Option<PhotonManager>,

    // Elements and components
    elements: Vec<Arc<dyn Element>>,

    // User Input
    impulse: ImpulseManager,

    // Keeping User defined variables
    // state: Option<IsotopeState>,
    state: Option<Arc<RwLock<dyn IsotopeState>>>,

    // Isotope start function
    init_callback: fn(&mut Self),

    // Isotope update function
    update_callback: fn(&mut Self),

    // Delta for updating
    pub delta: Instant,

    // Bool and thread handle for multithreading
    running: Arc<RwLock<bool>>,
    thread_handle: Option<JoinHandle<()>>,
}

pub fn new_isotope(
    init_callback: fn(&mut Isotope),
    update_callback: fn(&mut Isotope),
) -> Result<Isotope> {
    let gpu_controller = Arc::new(GpuController::new()?);

    Ok(Isotope {
        gpu_controller,
        photon: None,
        elements: Vec::new(),
        impulse: ImpulseManager::default(),
        state: None,
        init_callback,
        update_callback,
        delta: Instant::now(),
        running: Arc::new(RwLock::new(false)),
        thread_handle: None,
    })
}

impl Isotope {
    fn initialize(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        self.photon = Some(PhotonManager::new(event_loop, self.gpu_controller.clone())?);

        (self.init_callback)(self);

        Ok(())
    }

    /// Add Elements to isotope
    pub fn add_element(&mut self, element: Arc<dyn Element>) {
        self.elements.push(element);
    }

    /// For handling input
    pub fn impulse(&mut self) -> &mut ImpulseManager {
        &mut self.impulse
    }

    /// Access the the engine camera
    pub fn camera(&mut self) -> Option<&mut PhotonCamera> {
        Some(&mut self.photon.as_mut()?.renderer.camera)
    }

    /// Add a state to the isotope engine
    pub fn add_state<S: IsotopeState + 'static>(&mut self, state: S) {
        self.state.replace(Arc::new(RwLock::new(state)));
        info!("Added Game State: {:#?}", self.state);
        info!("Starting State Update Thread");

        let state_clone = unsafe { self.state.as_ref().unwrap_unchecked().clone() };
        let running_clone = self.running.clone();

        // Start an update thread that will run however fast it feels like
        self.thread_handle = Some(thread::spawn(move || {
            let mut delta_t = Instant::now();
            loop {
                if let Ok(mut state) = state_clone.write() {
                    state.update(&delta_t);
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

    // Test function
    #[deprecated]
    pub fn add_from_obj<P>(&mut self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let model = Model::from_obj(
            &path,
            &self.gpu_controller,
            &self.photon.as_ref().unwrap().renderer.layouts,
        )?;

        let test = Arc::new(TestElement { model });
        self.elements.push(test);

        Ok(())
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
                        state_guard.update_with_camera(&mut *camera_ptr, &self.delta);
                    }
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
                        thread.join();
                    }

                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    if let Some(photon) = &self.photon {
                        match photon.render(&self.elements) {
                            Ok(()) => {}
                            Err(err) => {
                                error!("Rendering failed with Error: {}", err);
                                event_loop.exit();
                            }
                        }
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
                                if let Some(callback) = self.impulse.key_is_released {
                                    match physical_key {
                                        winit::keyboard::PhysicalKey::Code(code) => {
                                            callback(code, self);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        },
                    }
                }
                _ => {}
            }
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
