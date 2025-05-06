use std::sync::Arc;

use cgmath::{InnerSpace, Quaternion, Rotation};
use isotope::{Isotope, IsotopeState, KeyCode, new_isotope, start_isotope};
use log::*;

#[derive(Debug, Default)]
pub struct GameState {
    pub w_pressed: bool,
    pub s_pressed: bool,
    pub a_pressed: bool,
    pub d_pressed: bool,
    pub shift_pressed: bool,
    pub space_pressed: bool,

    pub mouse_diff: (f64, f64),

    pub mouse_focused: bool,
}

const CAMERA_SPEED: f32 = 7.0;
const CAMERA_LOOK_SPEED: f32 = 0.01;

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

        if self.space_pressed {
            camera.modify(|eye, _, up, _, _, _, _| {
                *eye += up.normalize() * dt * CAMERA_SPEED;
            });
        }

        if self.shift_pressed {
            camera.modify(|eye, _, up, _, _, _, _| {
                *eye -= up.normalize() * dt * CAMERA_SPEED;
            });
        }

        // Change where the camera is looking
        camera.modify(|_, target, up, _, _, _, _| {
            // Change pitch
            let forward_norm = target.normalize();
            let right = forward_norm.cross(*up);

            let rotation = Quaternion {
                v: right * f32::sin(self.mouse_diff.1 as f32 * CAMERA_LOOK_SPEED / 2.0),
                s: f32::cos(self.mouse_diff.0 as f32 * CAMERA_LOOK_SPEED / 2.0),
            }
            .normalize();

            *target = rotation.rotate_vector(*target);

            // Change yaw
            let up_norm = up.normalize();
            let rotation = Quaternion {
                v: up_norm * f32::sin(self.mouse_diff.0 as f32 * CAMERA_LOOK_SPEED / 2.0),
                s: f32::cos(self.mouse_diff.0 as f32 * CAMERA_LOOK_SPEED / 2.0),
            }
            .normalize();

            *target = rotation.rotate_vector(*target);
        });

        self.mouse_diff = (0.0, 0.0);
    }

    fn update_with_window(&mut self, window: &winit::window::Window, delta_t: &std::time::Instant) {
        if self.mouse_focused {
            _ = window.set_cursor_grab(winit::window::CursorGrabMode::Locked);
            window.set_cursor_visible(false);
        } else {
            _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
            window.set_cursor_visible(true);
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
            KeyCode::Escape => {
                iso.with_state_typed_mut::<GameState, _, _>(|game_state| {
                    game_state.mouse_focused = !game_state.mouse_focused;
                });
            }
            KeyCode::Space => {
                iso.with_state_typed_mut::<GameState, _, _>(|game_state| {
                    game_state.space_pressed = true;
                });
            }
            KeyCode::ShiftLeft => {
                iso.with_state_typed_mut::<GameState, _, _>(|game_state| {
                    game_state.shift_pressed = true;
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
            KeyCode::Space => {
                iso.with_state_typed_mut::<GameState, _, _>(|game_state| {
                    game_state.space_pressed = false;
                });
            }
            KeyCode::ShiftLeft => {
                iso.with_state_typed_mut::<GameState, _, _>(|game_state| {
                    game_state.shift_pressed = false;
                });
            }
            _ => {}
        })
        .mouse_is_moved(|delta, iso| {
            iso.with_state_typed_mut::<GameState, _, _>(|game_state| {
                if game_state.mouse_focused {
                    game_state.mouse_diff = (-delta.0, -delta.1);
                }
            });
        });
}

fn update(isotope: &mut Isotope) {}

fn main() {
    let mut app = new_isotope(init, update).expect("Failed");
    _ = start_isotope(&mut app);
}
