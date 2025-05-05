use std::{fmt::Debug, path::Path, sync::Arc};

use anyhow::Result;
use element::model::Model;
use gpu_utils::GpuController;
use log::*;
use photon::PhotonManager;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
};

pub use element::Element;

mod element;
mod gpu_utils;
mod photon;
mod utils;

// Test struct
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

    init_callback: fn(&mut Self),
}

impl Isotope {
    pub fn new(init_callback: fn(&mut Self)) -> Result<Self> {
        let gpu_controller = Arc::new(GpuController::new()?);

        Ok(Self {
            gpu_controller,
            photon: None,
            elements: Vec::new(),
            init_callback,
        })
    }

    fn initialize(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        self.photon = Some(PhotonManager::new(event_loop, self.gpu_controller.clone())?);

        (self.init_callback)(self);

        Ok(())
    }

    pub fn add_element(&mut self, element: Arc<dyn Element>) {
        self.elements.push(element);
    }

    // Test function
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
                _ => {}
            }
        }
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
