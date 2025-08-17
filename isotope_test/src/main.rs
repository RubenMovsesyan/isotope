use isotope_app::IsotopeApplication;
use winit::event_loop::{self, ControlFlow, EventLoop};

fn main() {
    pretty_env_logger::init();
    let mut isotope = IsotopeApplication::new().unwrap();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    _ = event_loop.run_app(&mut isotope);
}
