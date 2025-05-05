use isotope::{Isotope, start_isotope};
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
}

fn main() {
    let mut app = Isotope::new(init).expect("Failed");
    _ = start_isotope(&mut app);
}
