use std::time::Instant;

use cgmath::{Point3, Vector3, Zero};
use isotope::debugger::*;
use isotope::*;

#[allow(unused_imports)]
use log::*;

#[derive(Debug)]
enum GameState {
    InitialState,
}

impl IsotopeState for GameState {
    fn init(&mut self, ecs: &mut Compound, asset_manager: &mut AssetManager, delta_t: f32, t: f32) {
        let cube = ecs.create_entity();
        ecs.add_molecule(cube, Model::from_obj("test_files/cube.obj", asset_manager));

        ecs.add_molecule(cube, String::from("Cube"));
        ecs.add_molecule(cube, Transform::default());
        ecs.add_molecule(
            cube,
            BosonObject::new({
                let mut rb = RigidBody::new(10.0, ColliderBuilder::Cube);
                rb.position = Vector3 {
                    x: 0.0,
                    y: 10.0,
                    z: 0.0,
                };

                rb.velocity = Vector3 {
                    x: 0.0,
                    y: 10.0,
                    z: 0.0,
                };

                rb
            }),
        );

        let plane = ecs.create_entity();
        ecs.add_molecule(
            plane,
            Model::from_obj("test_files/plane.obj", asset_manager),
        );
        ecs.add_molecule(plane, Transform::default());
        ecs.add_molecule(
            plane,
            BosonObject::new(StaticCollider::new(
                Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                Collider::new_plane(Vector3::unit_y(), 0.0),
            )),
        );

        let light = ecs.create_entity();
        ecs.add_molecule(
            light,
            Light::new(
                Point3 {
                    x: 10.0,
                    y: 10.0,
                    z: 10.0,
                },
                Vector3::zero(),
                Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                },
                1.0,
            ),
        );

        let camera = ecs.create_entity();
        ecs.add_molecule(camera, Camera3D);
        ecs.add_molecule(
            camera,
            Transform {
                position: Vector3 {
                    x: 10.0,
                    y: 10.0,
                    z: 10.0,
                },
                ..Default::default()
            },
        );

        let debugger = ecs.create_entity();
        ecs.add_molecule(debugger, Debugger::None);
    }

    fn update(
        &mut self,
        ecs: &mut Compound,
        asset_manager: &mut AssetManager,
        delta_t: f32,
        t: f32,
    ) {
        ecs.for_each_duo_mut(
            |_entity, _camera: &mut Camera3D, transform: &mut Transform| {
                transform.position = Vector3 {
                    x: f32::sin(t) + 10.0,
                    y: 5.0,
                    z: f32::cos(t) + 10.0,
                };
            },
        );
    }

    fn key_is_pressed(
        &mut self,
        ecs: &mut Compound,
        asset_manager: &mut AssetManager,
        key_code: KeyCode,
    ) {
        match key_code {
            KeyCode::KeyR => {
                let monkey = ecs.create_entity();
                ecs.add_molecule(
                    monkey,
                    Model::from_obj("test_files/monkey.obj", asset_manager),
                );

                ecs.add_molecule(monkey, Transform::default());

                ecs.add_molecule(
                    monkey,
                    BosonObject::new({
                        let mut rb = RigidBody::new(10.0, ColliderBuilder::Cube);

                        rb.position = Vector3::new(-5.0, 0.0, -5.0);
                        rb.velocity = Vector3::new(5.0, 10.0, 0.0);

                        rb
                    }),
                );
            }
            KeyCode::Digit0 => {
                ecs.for_each_molecule_mut(|_entity, debugger: &mut Debugger| {
                    *debugger = debugger.toggle_boson();
                });
            }
            _ => {}
        }
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }
}

fn update(ecs: &mut Compound, asset_manager: &mut AssetManager, delta_t: &Instant, t: &Instant) {}

fn main() {
    let mut app = new_isotope(
        |isotope: &mut Isotope| {
            let game_state = GameState::InitialState;
            isotope.set_state(game_state);
        },
        update,
    )
    .expect("Failed");

    _ = start_isotope(&mut app);
}
