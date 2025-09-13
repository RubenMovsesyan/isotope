use isotope_app::*;
use winit::event_loop::{self, ControlFlow, EventLoop};

struct GameState {}

impl IsotopeState for GameState {
    fn init(&mut self, ecs: &Compound, assets: &AssetServer) {
        match Model::from_obj("test_files/monkey.obj", assets) {
            Ok(model) => {
                ecs.spawn((model,));
            }
            Err(err) => {
                error!("Failed to load model: {}", err);
            }
        }

        ecs.spawn((Light::new(
            [10.0, 2.0, 3.0],
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            5.0,
        ),));
    }

    fn update(&mut self, ecs: &Compound, assets: &AssetServer, delta_t: f32, t: f32) {
        debug!("Ran Update");
        ecs.iter_mut_mol(|_entity, light: &mut Light| {
            light.pos(|position| {
                *position = [5.0 * f32::cos(t), 2.0, 5.0 * f32::sin(t)];
            });
        });
    }
}

fn main() {
    pretty_env_logger::init();

    let game_state = GameState {};

    let mut isotope = IsotopeApplication::new(game_state).unwrap();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    _ = event_loop.run_app(&mut isotope);
}
