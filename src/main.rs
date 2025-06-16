use std::time::Instant;

use cgmath::{Point3, Vector3, Zero};
use isotope::debugger::*;
use isotope::*;

#[allow(unused_imports)]
use log::*;

const CAMERA_SPEED: f32 = 10.0;
const CAMERA_LOOK_SPEED: f32 = 75.0;

#[derive(Debug)]
struct GameState {
    w_pressed: bool,
    s_pressed: bool,
    a_pressed: bool,
    d_pressed: bool,

    q_pressed: bool,
    e_pressed: bool,

    window_focused: bool,

    mouse_diff: (f32, f32),
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
        // ecs.add_molecule(
        //     camera,
        //     Transform {
        //         position: Vector3 {
        //             x: 10.0,
        //             y: 10.0,
        //             z: 10.0,
        //         },
        //         ..Default::default()
        //     },
        // );
        ecs.add_molecule(camera, CameraController::default());

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
        ecs.for_each_molecule_mut(|_entity, camera_controller: &mut CameraController| {
            if self.w_pressed {
                camera_controller.forward(CAMERA_SPEED * delta_t);
            }

            if self.s_pressed {
                camera_controller.backward(CAMERA_SPEED * delta_t);
            }

            if self.a_pressed {
                camera_controller.strafe_left(CAMERA_SPEED * delta_t);
            }

            if self.d_pressed {
                camera_controller.strafe_right(CAMERA_SPEED * delta_t);
            }

            if self.e_pressed {
                camera_controller.zoom_in(CAMERA_SPEED * delta_t);
            }

            if self.q_pressed {
                camera_controller.zoom_out(CAMERA_SPEED * delta_t);
            }
        });
    }

    fn mouse_is_moved(
        &mut self,
        ecs: &mut Compound,
        asset_manager: &mut AssetManager,
        delta: (f64, f64),
        delta_t: f32,
        t: f32,
    ) {
        self.mouse_diff = (-delta.0 as f32, -delta.1 as f32);

        if self.window_focused {
            ecs.for_each_molecule_mut(|_entity, camera_controller: &mut CameraController| {
                camera_controller.look((
                    self.mouse_diff.0 as f32 * CAMERA_LOOK_SPEED * delta_t,
                    self.mouse_diff.1 as f32 * CAMERA_LOOK_SPEED * delta_t,
                ));
            });
        }

        self.mouse_diff = (0.0, 0.0);
    }

    fn key_is_pressed(
        &mut self,
        ecs: &mut Compound,
        asset_manager: &mut AssetManager,
        key_code: KeyCode,
        delta_t: f32,
        t: f32,
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
                    debugger.toggle_boson();
                });
            }
            KeyCode::KeyW => self.w_pressed = true,
            KeyCode::KeyS => self.s_pressed = true,
            KeyCode::KeyA => self.a_pressed = true,
            KeyCode::KeyD => self.d_pressed = true,
            KeyCode::KeyQ => self.q_pressed = true,
            KeyCode::KeyE => self.e_pressed = true,
            KeyCode::Escape => {
                ecs.for_each_molecule_mut(|_entity, window_controller: &mut WindowController| {
                    window_controller.cursor_grab_mode(|cursor_grab_mode| match cursor_grab_mode {
                        CursorGrabMode::None => {
                            *cursor_grab_mode = CursorGrabMode::Locked;
                        }
                        CursorGrabMode::Locked => {
                            *cursor_grab_mode = CursorGrabMode::None;
                        }
                        _ => {}
                    });

                    window_controller.cursor_visible(|cursor_visible| {
                        *cursor_visible = !*cursor_visible;
                    });
                });

                self.window_focused = !self.window_focused;
            }
            _ => {}
        }
    }

    fn key_is_released(
        &mut self,
        ecs: &mut Compound,
        asset_manager: &mut AssetManager,
        key_code: KeyCode,
        delta_t: f32,
        t: f32,
    ) {
        match key_code {
            KeyCode::KeyW => self.w_pressed = false,
            KeyCode::KeyS => self.s_pressed = false,
            KeyCode::KeyA => self.a_pressed = false,
            KeyCode::KeyD => self.d_pressed = false,
            KeyCode::KeyQ => self.q_pressed = false,
            KeyCode::KeyE => self.e_pressed = false,
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
            let game_state = GameState {
                a_pressed: false,
                s_pressed: false,
                d_pressed: false,
                w_pressed: false,

                q_pressed: false,
                e_pressed: false,

                window_focused: false,

                mouse_diff: (0.0, 0.0),
            };
            isotope.set_state(game_state);
        },
        update,
    )
    .expect("Failed");

    _ = start_isotope(&mut app);
}
