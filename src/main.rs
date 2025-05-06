use std::sync::Arc;

use cgmath::InnerSpace;
use isotope::{Isotope, IsotopeState, KeyCode, new_isotope, start_isotope};
use log::*;

#[derive(Debug, Default)]
pub struct GameState {
    pub w_pressed: bool,
    pub s_pressed: bool,
    pub a_pressed: bool,
    pub d_pressed: bool,
}

const CAMERA_SPEED: f32 = 5.0;

impl IsotopeState for GameState {
    fn update_with_camera(
        &mut self,
        camera: &mut isotope::PhotonCamera,
        delta_t: &std::time::Instant,
    ) {
        let dt = delta_t.elapsed().as_secs_f32();

        if self.w_pressed {
            camera.modify(|eye, target, _, _, _, _, _| {
                *eye += target.normalize() * dt * CAMERA_SPEED;
            });
        }

        if self.s_pressed {
            camera.modify(|eye, target, _, _, _, _, _| {
                *eye -= target.normalize() * dt * CAMERA_SPEED;
            });
        }

        if self.a_pressed {
            camera.modify(|eye, target, up, _, _, _, _| {
                *eye += up.clone().cross(*target).normalize() * dt * CAMERA_SPEED;
            });
        }

        if self.d_pressed {
            camera.modify(|eye, target, up, _, _, _, _| {
                *eye -= up.clone().cross(*target).normalize() * dt * CAMERA_SPEED;
            });
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}

fn init(isotope: &mut Isotope) {
    match isotope.add_from_obj("test_files/cube.obj") {
        Ok(()) => {
            info!("Cube Added successfully");
        }
        Err(err) => {
            error!("Cube failed with error: {err}");
        }
    }

    isotope.add_state(GameState::default());

    isotope
        .impulse()
        .key_is_pressed(|key_code, iso| match key_code {
            KeyCode::KeyW => {
                iso.with_state_typed_mut::<GameState, _, _>(|game_state| {
                    game_state.w_pressed = true;
                });
            }
            KeyCode::KeyS => {
                iso.with_state_typed_mut::<GameState, _, _>(|game_state| {
                    game_state.s_pressed = true;
                });
            }
            KeyCode::KeyA => {
                iso.with_state_typed_mut::<GameState, _, _>(|game_state| {
                    game_state.a_pressed = true;
                });
            }
            KeyCode::KeyD => {
                iso.with_state_typed_mut::<GameState, _, _>(|game_state| {
                    game_state.d_pressed = true;
                });
            }
            _ => {}
        })
        .key_is_released(|key_code, iso| match key_code {
            KeyCode::KeyW => {
                iso.with_state_typed_mut::<GameState, _, _>(|game_state| {
                    game_state.w_pressed = false;
                });
            }
            KeyCode::KeyS => {
                iso.with_state_typed_mut::<GameState, _, _>(|game_state| {
                    game_state.s_pressed = false;
                });
            }
            KeyCode::KeyA => {
                iso.with_state_typed_mut::<GameState, _, _>(|game_state| {
                    game_state.a_pressed = false;
                });
            }
            KeyCode::KeyD => {
                iso.with_state_typed_mut::<GameState, _, _>(|game_state| {
                    game_state.d_pressed = false;
                });
            }
            _ => {}
        });
}

fn update(isotope: &mut Isotope) {}

fn main() {
    let mut app = new_isotope(init, update).expect("Failed");
    _ = start_isotope(&mut app);
}
