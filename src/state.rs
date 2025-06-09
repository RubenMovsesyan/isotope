use std::{any::Any, fmt::Debug};
use winit::{dpi::PhysicalPosition, keyboard::KeyCode};

use crate::{AssetManager, compound::Compound};

pub trait IsotopeState: Debug + Send + Sync {
    // This is run once when the game state is first added to Isotope ==========
    #[allow(unused_variables)]
    fn init(&mut self, ecs: &mut Compound, asset_manager: &mut AssetManager, delta_t: f32, t: f32) {
    }

    #[allow(unused_variables)]
    fn update(
        &mut self,
        ecs: &mut Compound,
        asset_manager: &mut AssetManager,
        delta_t: f32,
        t: f32,
    ) {
    }

    // Updating with and without access to parts of Isotope ====================
    // #[allow(unused_variables)]
    // fn update(&mut self, delta_t: &Instant, t: &Instant) {}

    // #[allow(unused_variables)]
    // fn update_with_camera(&mut self, camera: &mut PhotonCamera, delta_t: &Instant, t: &Instant) {}

    // #[allow(unused_variables)]
    // fn update_with_window(&mut self, window: &Window, delta_t: &Instant, t: &Instant) {}

    // #[allow(unused_variables)]
    // fn update_with_ecs(&mut self, ecs: &Compound, delta_t: &Instant, t: &Instant) {}

    // // Specific for rendering =================================================
    // #[allow(unused_variables)]
    // fn render_elements(&self, render_pass: &mut RenderPass) {}

    // #[allow(unused_variables)]
    // fn debug_render_elements(&self, render_pass: &mut RenderPass) {}

    // fn get_lights(&self) -> &[Light] {
    //     &[]
    // }

    // // For physics updates to the state =======================================
    // #[allow(unused_variables)]
    // fn init_boson(&mut self, boson: &mut Boson) {}
    // #[allow(unused_variables)]
    // fn run_boson_updates(&mut self, boson: &mut Boson, delta_t: &Instant) {}

    // Key inputs in the gamestate ============================================
    #[allow(unused_variables)]
    fn key_is_pressed(
        &mut self,
        ecs: &mut Compound,
        asset_manager: &mut AssetManager,
        key_code: KeyCode,
    ) {
    }

    #[allow(unused_variables)]
    fn key_is_released(
        &mut self,
        ecs: &mut Compound,
        asset_manager: &mut AssetManager,
        key_code: KeyCode,
    ) {
    }

    #[allow(unused_variables)]
    fn cursor_moved(
        &mut self,
        ecs: &mut Compound,
        asset_manager: &mut AssetManager,
        position: PhysicalPosition<f64>,
    ) {
    }

    #[allow(unused_variables)]
    fn mouse_is_moved(
        &mut self,
        ecs: &mut Compound,
        asset_manager: &mut AssetManager,
        delta: (f64, f64),
    ) {
    }

    // Required for downcasting ================================================
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
