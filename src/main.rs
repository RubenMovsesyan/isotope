use isotope::{Isotope, IsotopeState, KeyCode, new_isotope, start_isotope};
use log::*;

#[derive(Debug, Default)]
pub struct GameState {
    pub w_pressed: bool,
    pub s_pressed: bool,
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

    isotope.add_state(IsotopeState {
        w_pressed: false,
        s_pressed: false,
    });

    isotope
        .impulse()
        .key_is_pressed(|key_code, iso| match key_code {
            KeyCode::KeyW => {
                if let Some(state) = iso.state_mut() {
                    state.w_pressed = true;
                }
            }
            KeyCode::KeyS => {
                if let Some(state) = iso.state_mut() {
                    state.s_pressed = true;
                }
            }
            _ => {}
        })
        .key_is_released(|key_code, iso| match key_code {
            KeyCode::KeyW => {
                if let Some(state) = iso.state_mut() {
                    state.w_pressed = false;
                }
            }
            KeyCode::KeyS => {
                if let Some(state) = iso.state_mut() {
                    state.s_pressed = false;
                }
            }
            _ => {}
        });
}

fn update(isotope: &mut Isotope) {
    let mut w_pressed: bool = false;
    let mut s_pressed: bool = false;

    if let Some(state) = isotope.state() {
        w_pressed = state.w_pressed;
        s_pressed = state.s_pressed;
    }

    if w_pressed {
        if let Some(cam) = isotope.camera() {
            cam.set_fovy(|fovy| {
                *fovy += 0.1;
            });
        }
    }

    if s_pressed {
        if let Some(cam) = isotope.camera() {
            cam.set_fovy(|fovy| {
                *fovy -= 0.1;
            });
        }
    }
}

fn main() {
    let mut app = new_isotope(init, update).expect("Failed");
    _ = start_isotope(&mut app);
}
