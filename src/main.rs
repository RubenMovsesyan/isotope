use std::f32::consts::PI;

use cgmath::{Deg, InnerSpace, Quaternion, Rad, Rotation, Rotation3, Vector3, Zero};
use isotope::{compound::Compound, *};

#[allow(unused_imports)]
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

    pub ecs: Compound,
    pub lights: [Light; 3],
}

#[derive(Debug)]
pub struct TestElement {
    model: Model,
}

impl Element for TestElement {
    fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        self.model.render(render_pass);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}

#[derive(Debug)]
pub struct MonkeyElement {
    model: Model,
    rigid_body: BosonObject,
}

impl Element for MonkeyElement {
    fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        self.model.render(render_pass);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}

#[derive(Debug)]
pub struct ConeElement {
    model: Model,
    rigid_body: BosonObject,
}

impl Element for ConeElement {
    fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        self.model.render(render_pass);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}

const CAMERA_SPEED: f32 = 7.0;
const CAMERA_LOOK_SPEED: f32 = 0.01;

impl IsotopeState for GameState {
    fn update_with_camera(
        &mut self,
        camera: &mut isotope::PhotonCamera,
        delta_t: &std::time::Instant,
        _t: &std::time::Instant,
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

    fn get_lights(&self) -> &[Light] {
        &self.lights
    }

    fn init_boson(&mut self, boson: &mut Boson) {
        self.ecs
            .for_each_molecule(|_entity, monkey: &MonkeyElement| {
                boson.add_dynamic_object(monkey.rigid_body.clone());
            });

        self.ecs.for_each_molecule(|_entity, cone: &ConeElement| {
            boson.add_dynamic_object(cone.rigid_body.clone());
        });

        boson.add_dynamic_object(BosonObject::new(StaticCollider::new(
            Vector3::zero(),
            Collider::new_plane(Vector3::unit_y(), 0.0),
        )));
    }

    fn update(&mut self, _delta_t: &std::time::Instant, t: &std::time::Instant) {
        self.lights[0].pos(|x, y, z| {
            *x = 10.0 * f32::cos(t.elapsed().as_secs_f32());
            *y = 10.0 * f32::sin(t.elapsed().as_secs_f32());
            *z = 10.0 * f32::cos(t.elapsed().as_secs_f32());
        });

        self.lights[1].pos(|x, y, z| {
            *x = 10.0 * f32::sin(t.elapsed().as_secs_f32());
            *y = 10.0 * f32::cos(t.elapsed().as_secs_f32());
            *z = 10.0 * f32::sin(t.elapsed().as_secs_f32());
        });

        self.lights[2].pos(|x, y, z| {
            *x = 10.0 * f32::cos(t.elapsed().as_secs_f32());
            *y = 10.0 * f32::cos(t.elapsed().as_secs_f32());
            *z = 10.0 * f32::sin(t.elapsed().as_secs_f32());
        });

        // const SCALAR: f32 = 30.0;
        // self.ecs
        //     .for_each_molecule_mut(|_entity, cube: &mut TestElement| {
        //         cube.model.modify_instances(|instances| {
        //             instances[0].rotation = Quaternion::from_axis_angle(
        //                 Vector3::unit_x(),
        //                 Deg(SCALAR * t.elapsed().as_secs_f32()),
        //             )
        //             .normalize()
        //             .into();

        //             instances[1].rotation = Quaternion::from_axis_angle(
        //                 Vector3::unit_y(),
        //                 Deg(SCALAR * t.elapsed().as_secs_f32()),
        //             )
        //             .normalize()
        //             .into();

        //             instances[2].rotation = Quaternion::from_axis_angle(
        //                 Vector3::unit_z(),
        //                 Deg(SCALAR * t.elapsed().as_secs_f32()),
        //             )
        //             .normalize()
        //             .into();
        //         });

        //         cube.model.pos(|position| {
        //             position.x = f32::cos(SCALAR * t.elapsed().as_secs_f32() * PI / 180.0);
        //             position.y = f32::sin(SCALAR * t.elapsed().as_secs_f32() * PI / 180.0);
        //         });
        //     });
    }

    fn update_with_window(
        &mut self,
        window: &winit::window::Window,
        _delta_t: &std::time::Instant,
        _t: &std::time::Instant,
    ) {
        if self.mouse_focused {
            _ = window.set_cursor_grab(winit::window::CursorGrabMode::Locked);
            window.set_cursor_visible(false);
        } else {
            _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
            window.set_cursor_visible(true);
        }
    }

    fn render_elements(&self, render_pass: &mut wgpu::RenderPass) {
        self.ecs
            .for_each_molecule_mut(|_entity, cube: &mut TestElement| {
                cube.render(render_pass);
            });

        self.ecs
            .for_each_molecule_mut(|_entity, monkey: &mut MonkeyElement| {
                monkey.render(render_pass);
            });

        self.ecs
            .for_each_molecule_mut(|_entity, cone: &mut ConeElement| {
                cone.render(render_pass);
            });

        self.ecs
            .for_each_molecule_mut(|_entity, model: &mut Model| {
                model.render(render_pass);
            });
    }

    fn key_is_pressed(&mut self, key_code: KeyCode) {
        match key_code {
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
            KeyCode::Escape => {
                self.mouse_focused = !self.mouse_focused;
            }
            KeyCode::Space => {
                self.space_pressed = true;
            }
            KeyCode::ShiftLeft => {
                self.shift_pressed = true;
            }
            KeyCode::KeyR => {
                self.ecs
                    .for_each_molecule_mut(|_entity, monkey: &mut MonkeyElement| {
                        monkey.rigid_body.modify(|boson_body| {
                            boson_body.pos(|position| {
                                *position = Vector3 {
                                    x: 0.0,
                                    y: 5.0,
                                    z: 0.0,
                                };
                            });

                            boson_body.vel(|velocity| {
                                *velocity = Vector3 {
                                    x: 10.0,
                                    y: 10.0,
                                    z: 0.0,
                                };
                            });
                        });
                    });

                self.ecs
                    .for_each_molecule_mut(|_entity, cone: &mut ConeElement| {
                        cone.rigid_body.modify(|boson_body| {
                            boson_body.pos(|position| {
                                *position = Vector3 {
                                    x: 0.0,
                                    y: 10.0,
                                    z: 0.0,
                                };
                            });

                            boson_body.vel(|velocity| {
                                *velocity = Vector3 {
                                    x: 0.0,
                                    y: 5.0,
                                    z: 0.0,
                                };
                            });

                            boson_body.angular_vel(|angular_velocity| {
                                *angular_velocity = Vector3 {
                                    x: 0.0,
                                    y: 0.0,
                                    z: 1.0,
                                }
                            });
                        });
                    });
            }
            _ => {}
        }
    }

    fn key_is_released(&mut self, key_code: KeyCode) {
        match key_code {
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

    fn mouse_is_moved(&mut self, delta: (f64, f64)) {
        if self.mouse_focused {
            self.mouse_diff = (-delta.0, -delta.1);
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
    isotope.set_state({
        let mut state = GameState::default();

        // let cube = state.ecs.create_entity();
        // state.ecs.add_molecule(
        //     cube,
        //     TestElement {
        //         model: {
        //             let mut model =
        //                 Model::from_obj("test_files/cube.obj", &isotope).expect("Failed");

        //             model.set_instances(&[
        //                 ModelInstance {
        //                     position: [0.0, 0.0, 0.0],
        //                     rotation: Quaternion::from_axis_angle(Vector3::unit_x(), Deg(45.0))
        //                         .normalize()
        //                         .into(),
        //                 },
        //                 ModelInstance {
        //                     position: [5.0, 0.0, 0.0],
        //                     rotation: Quaternion::from_axis_angle(Vector3::unit_y(), Deg(45.0))
        //                         .normalize()
        //                         .into(),
        //                 },
        //                 ModelInstance {
        //                     position: [0.0, 0.0, 5.0],
        //                     rotation: Quaternion::from_axis_angle(Vector3::unit_z(), Deg(45.0))
        //                         .normalize()
        //                         .into(),
        //                 },
        //             ]);

        //             model
        //         },
        //     },
        // );

        // let monkey = state.ecs.create_entity();
        // let rb = BosonObject::new({
        //     let mut body = RigidBody::new(10.0).unwrap();
        //     body.position = Vector3 {
        //         x: 0.0,
        //         y: 5.0,
        //         z: 0.0,
        //     };

        //     body.velocity = Vector3 {
        //         x: 10.0,
        //         y: 10.0,
        //         z: 0.0,
        //     };

        //     body
        // });
        // state.ecs.add_molecule(
        //     monkey,
        //     MonkeyElement {
        //         model: {
        //             let mut model =
        //                 Model::from_obj("test_files/monkey.obj", &isotope).expect("Failed");

        //             model.link(rb.clone());

        //             model
        //         },
        //         rigid_body: rb,
        //     },
        // );

        let cone = state.ecs.create_entity();
        let cone_rb = BosonObject::new({
            let mut body = RigidBody::new(10.0).unwrap();
            body.position = Vector3 {
                x: 0.0,
                y: 10.0,
                z: 0.0,
            };
            body.velocity = Vector3 {
                x: 0.0,
                y: 5.0,
                z: 5.0,
            };
            body.angular_velocity = Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            body
        });

        state.ecs.add_molecule(
            cone,
            ConeElement {
                model: {
                    let mut model =
                        Model::from_obj("test_files/cube.obj", &isotope).expect("Failed");

                    model.link(cone_rb.clone());
                    model
                },
                rigid_body: cone_rb,
            },
        );

        let plane = state.ecs.create_entity();
        state.ecs.add_molecule(plane, {
            let mut model = Model::from_obj("test_files/plane.obj", &isotope).expect("Failed");

            model.rot(|orientation| {
                *orientation = Quaternion::from_axis_angle(Vector3::unit_z(), Deg(180.0));
            });

            model
        });

        let other_cube = state.ecs.create_entity();
        state.ecs.add_molecule(
            other_cube,
            TestElement {
                model: Model::from_obj("test_files/other_cube.obj", &isotope).expect("Failed"),
            },
        );

        // state.lights[0].color = [1.0, 0.0, 0.0];
        state.lights[0].color = [1.0, 1.0, 1.0];
        state.lights[0].intensity = 1.0;
        // state.lights[1].color = [0.0, 0.0, 1.0];
        state.lights[1].color = [1.0, 1.0, 1.0];
        state.lights[1].intensity = 1.0;
        // state.lights[2].color = [0.0, 1.0, 0.0];
        state.lights[2].color = [1.0, 1.0, 1.0];
        state.lights[2].intensity = 1.0;
        state
    });
}

fn update(_isotope: &mut Isotope) {}

fn main() {
    let mut app = new_isotope(init, update).expect("Failed");
    _ = start_isotope(&mut app);
}
