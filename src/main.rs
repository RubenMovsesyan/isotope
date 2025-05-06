use isotope::{Isotope, KeyCode, new_isotope, start_isotope};
use log::*;

fn init(isotope: &mut Isotope) {
    match isotope.add_from_obj("test_files/cube.obj") {
        Ok(()) => {
            info!("Cube Added successfully");
        }
        Err(err) => {
            error!("Cube failed with error: {err}");
        }
    }

    isotope
        .impulse()
        .key_is_pressed(|key_code, iso| match key_code {
            KeyCode::KeyW => {
                if let Some(camera) = iso.camera() {
                    camera.set_fovy(|fovy| {
                        *fovy += 1.0;
                    });
                }
            }
            KeyCode::KeyS => {
                if let Some(camera) = iso.camera() {
                    camera.set_fovy(|fovy| {
                        *fovy -= 1.0;
                    });
                }
            }
            _ => {}
        });
}

fn main() {
    let mut app = new_isotope(init).expect("Failed");
    _ = start_isotope(&mut app);
}
