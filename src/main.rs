use isotope::{Isotope, new_isotope, start_isotope};
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

    isotope.impulse().key_is_pressed(|key_code, iso| {});
}

fn main() {
    let mut app = new_isotope(init).expect("Failed");
    _ = start_isotope(&mut app);
}
