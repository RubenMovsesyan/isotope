use std::{any::Any, fmt::Debug, time::Instant};

use wgpu::RenderPass;
use winit::{dpi::PhysicalPosition, keyboard::KeyCode, window::Window};

use crate::{Light, PhotonCamera, boson::Boson, compound::Compound};

pub trait IsotopeState: Debug + Send + Sync {
    // This is run once when the game state is first added to Isotope ==========
    #[allow(unused_variables)]
    fn init(&mut self, t: &Instant) {}

    // Updating with and without access to parts of Isotope ====================
    #[allow(unused_variables)]
    fn update(&mut self, delta_t: &Instant, t: &Instant) {}

    #[allow(unused_variables)]
    fn update_with_camera(&mut self, camera: &mut PhotonCamera, delta_t: &Instant, t: &Instant) {}

    #[allow(unused_variables)]
    fn update_with_window(&mut self, window: &Window, delta_t: &Instant, t: &Instant) {}

    #[allow(unused_variables)]
    fn update_with_ecs(&mut self, ecs: &Compound, delta_t: &Instant, t: &Instant) {}

    // Specific for rendering =================================================
    #[allow(unused_variables)]
    fn render_elements(&self, render_pass: &mut RenderPass) {}

    #[allow(unused_variables)]
    fn debug_render_elements(&self, render_pass: &mut RenderPass) {}

    fn get_lights(&self) -> &[Light] {
        &[]
    }

    // For physics updates to the state =======================================
    #[allow(unused_variables)]
    fn init_boson(&mut self, boson: &mut Boson) {}
    #[allow(unused_variables)]
    fn run_boson_updates(&mut self, boson: &mut Boson, delta_t: &Instant) {}

    // Key inputs in the gamestate ============================================
    #[allow(unused_variables)]
    fn key_is_pressed(&mut self, key_code: KeyCode) {}
    #[allow(unused_variables)]
    fn key_is_released(&mut self, key_code: KeyCode) {}
    #[allow(unused_variables)]
    fn cursor_moved(&mut self, position: PhysicalPosition<f64>) {}
    #[allow(unused_variables)]
    fn mouse_is_moved(&mut self, delta: (f64, f64)) {}

    // Required for downcasting ================================================
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
