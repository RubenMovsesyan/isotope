use std::{any::Any, fmt::Debug, path::Path, sync::Arc};

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
    state: Option<IsotopeState>,

    // Isotope start function
    init_callback: fn(&mut Self),

    // Isotope update function
    update_callback: fn(&mut Self),
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
    pub fn add_state(&mut self, state: IsotopeState) {
        self.state.replace(state);
        info!("Added Game State: {:#?}", self.state);
    }

    /// Immutable access to the state
    pub fn state(&self) -> Option<&IsotopeState> {
        self.state.as_ref()
    }

    /// Mutable access to the state
    pub fn state_mut(&mut self) -> Option<&mut IsotopeState> {
        self.state.as_mut()
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
        if unsafe { self.photon.as_ref().unwrap_unchecked().window().id() } == window_id {
            match event {
                WindowEvent::CloseRequested => {
                    info!("Shutting Down Isotope");
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

        // Run the fixed update
        // TODO: extract into thread
        (self.update_callback)(self);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
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
