use std::sync::Arc;

use anyhow::Result;
use gpu_utils::GpuController;
use log::*;
use photon::PhotonManager;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
};

mod element;
mod gpu_utils;
mod photon;
mod utils;

/// Main struct for the game engine app
#[derive(Debug)]
pub struct Isotope {
    // GPU
    gpu_controller: Arc<GpuController>,

    // Window and Rendering
    photon: Option<PhotonManager>,
}

impl Isotope {
    pub fn new() -> Result<Self> {
        let gpu_controller = Arc::new(GpuController::new()?);

        Ok(Self {
            gpu_controller,
            photon: None,
        })
    }

    fn initialize(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        self.photon = Some(PhotonManager::new(event_loop, self.gpu_controller.clone())?);

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

pub fn start_isotope() -> Result<()> {
    pretty_env_logger::init();
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = Isotope::new()?;
    _ = event_loop.run_app(&mut app);

    Ok(())
}
