use isotope::*;
use winit::event_loop::{ControlFlow, EventLoop};

const CAMERA_SPEED: f32 = 10.0;

#[derive(Default)]
struct GameState {
    window_focused: bool,

    w_pressed: bool,
    a_pressed: bool,
    s_pressed: bool,
    d_pressed: bool,

    space_pressed: bool,
    shift_pressed: bool,
}

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

        ecs.spawn((
            Camera::perspective_3d_default(assets),
            Transform3D::new(
                Vector3::new(0.0, 0.0, 10.0),
                Quaternion::from_axis_angle(Vector3::unit_y(), Deg(180.0)),
            ),
        ));
    }

    fn update(&mut self, ecs: &Compound, assets: &AssetServer, delta_t: f32, t: f32) {
        ecs.iter_mut_mol(|_entity, light: &mut Light| {
            light.pos(|position| {
                *position = [5.0 * f32::cos(t), 2.0, 5.0 * f32::sin(t)];
            });
        });

        if self.window_focused {
            ecs.iter_mut_duo(
                |_entity, camera: &mut Camera, transform: &mut Transform3D| {
                    transform.position(|pos| {
                        if self.w_pressed {
                            camera.target(|target| *pos += *target * CAMERA_SPEED * delta_t);
                        }

                        if self.s_pressed {
                            camera.target(|target| *pos -= *target * CAMERA_SPEED * delta_t);
                        }

                        if self.a_pressed {
                            camera.all(|_, target, up, _, _, _, _| {
                                *pos -= target.cross(*up).normalize() * CAMERA_SPEED * delta_t
                            });
                        }

                        if self.d_pressed {
                            camera.all(|_, target, up, _, _, _, _| {
                                *pos += target.cross(*up).normalize() * CAMERA_SPEED * delta_t
                            });
                        }

                        if self.space_pressed {
                            camera.up(|up| *pos += *up * CAMERA_SPEED * delta_t);
                        }

                        if self.shift_pressed {
                            camera.up(|up| *pos -= *up * CAMERA_SPEED * delta_t);
                        }
                    });
                },
            )
        }
    }

    fn mouse_is_moved(&mut self, ecs: &Compound, assets: &AssetServer, delta: (f64, f64), t: f32) {
        if self.window_focused {
            ecs.iter_mut_duo(
                |_entity, _camera: &mut Camera, transform: &mut Transform3D| {
                    transform.rotation(|rot| {
                        let sens = 0.002;
                        let yaw_delta = Rad(-delta.0 as f32 * sens);
                        let pitch_delta = Rad(delta.1 as f32 * sens);

                        let yaw_rot = Quaternion::from_axis_angle(Vector3::unit_y(), yaw_delta);
                        let pitch_rot = Quaternion::from_axis_angle(Vector3::unit_x(), pitch_delta);

                        let target_rotation = yaw_rot * *rot * pitch_rot;

                        *rot = rot.slerp(target_rotation, 0.8);
                    });
                },
            );
        }
    }

    fn key_is_pressed(&mut self, ecs: &Compound, assets: &AssetServer, key: KeyCode, t: f32) {
        match key {
            KeyCode::Escape => {
                ecs.iter_mut_mol(|_entity, window_controller: &mut WindowController| {
                    window_controller.all(|cursor_grab_mode, cursor_visible| {
                        if *cursor_visible {
                            *cursor_grab_mode = CursorGrabMode::Locked;
                        } else {
                            *cursor_grab_mode = CursorGrabMode::None;
                        }
                        *cursor_visible = !*cursor_visible;

                        self.window_focused = !*cursor_visible;
                    });
                });
            }
            KeyCode::KeyW => {
                self.w_pressed = true;
            }
            KeyCode::KeyS => {
                self.s_pressed = true;
            }
            KeyCode::KeyA => {
                self.a_pressed = true;
            }
            KeyCode::KeyD => {
                self.d_pressed = true;
            }
            KeyCode::Space => {
                self.space_pressed = true;
            }
            KeyCode::ShiftLeft => {
                self.shift_pressed = true;
            }
            _ => {}
        }
    }

    fn key_is_released(&mut self, ecs: &Compound, assets: &AssetServer, key: KeyCode, t: f32) {
        match key {
            KeyCode::KeyW => {
                self.w_pressed = false;
            }
            KeyCode::KeyS => {
                self.s_pressed = false;
            }
            KeyCode::KeyA => {
                self.a_pressed = false;
            }
            KeyCode::KeyD => {
                self.d_pressed = false;
            }
            KeyCode::Space => {
                self.space_pressed = false;
            }
            KeyCode::ShiftLeft => {
                self.shift_pressed = false;
            }
            _ => {}
        }
    }
}

fn main() {
    pretty_env_logger::init();

    let game_state = GameState::default();

    let mut isotope = IsotopeApplication::new(game_state).unwrap();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    _ = event_loop.run_app(&mut isotope);
}
