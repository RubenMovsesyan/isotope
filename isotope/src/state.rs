use std::any::Any;

use compound::Compound;
use winit::keyboard::KeyCode;

use crate::asset_server::AssetServer;

#[allow(unused_variables)]
pub trait IsotopeState: Send + Sync + 'static {
    fn init(&mut self, ecs: &Compound, assets: &AssetServer) {}

    fn update(&mut self, ecs: &Compound, assets: &AssetServer, delta_t: f32, t: f32) {}

    fn key_is_pressed(
        &mut self,
        ecs: &Compound,
        assets: &AssetServer,
        key: KeyCode,
        delta_t: f32,
        t: f32,
    ) {
    }

    fn key_is_released(
        &mut self,
        ecs: &Compound,
        assets: &AssetServer,
        key: KeyCode,
        delta_t: f32,
        t: f32,
    ) {
    }

    // Window Event
    fn cursor_moved(
        &mut self,
        ecs: &Compound,
        assets: &AssetServer,
        cursor_position: (f32, f32),
        delta_t: f32,
        t: f32,
    ) {
    }

    // Device event
    fn mouse_is_moved(
        &mut self,
        ecs: &Compound,
        assets: &AssetServer,
        cursor_position: (f32, f32),
        delta_t: f32,
        t: f32,
    ) {
    }
}
