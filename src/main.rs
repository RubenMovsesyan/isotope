use cgmath::{Point3, Vector3, Zero};
use isotope::*;

#[allow(unused_imports)]
use log::*;

fn init(isotope: &mut Isotope) {
    let asset_manager = isotope.asset_manager.clone();
    isotope
        .impulse()
        .key_is_pressed(|key_code, isotope| match key_code {
            KeyCode::Digit0 => {
                isotope.set_debbuging(|debugging| {
                    *debugging = !*debugging;
                });
            }
            KeyCode::KeyR => {
                let asset_manager = isotope.asset_manager.clone();
                let model = Model::from_obj("test_files/cube.obj", asset_manager).expect("Failed");

                isotope.ecs(|ecs| {
                    let new_cube = ecs.create_entity();
                    ecs.add_molecule(new_cube, model);
                    ecs.add_molecule(new_cube, String::from(format!("Cube: {}", new_cube)));
                    ecs.add_molecule(new_cube, Transform::default());

                    ecs.add_molecule(
                        new_cube,
                        BosonObject::new({
                            let mut rb =
                                RigidBody::new(10.0, ColliderBuilder::Cube).expect("Failed");

                            rb.position = Vector3 {
                                x: -10.0,
                                y: 0.0,
                                z: -10.0,
                            };

                            rb.velocity = Vector3 {
                                x: 0.0,
                                y: 10.0,
                                z: 5.0,
                            };

                            rb
                        }),
                    );
                });
            }
            _ => {}
        });

    isotope.ecs(|ecs| {
        let cube = ecs.create_entity();
        ecs.add_molecule(
            cube,
            Model::from_obj("test_files/cube.obj", asset_manager.clone()).expect("Failed"),
        );

        ecs.add_molecule(cube, String::from("Cube"));
        ecs.add_molecule(cube, Transform::default());
        ecs.add_molecule(
            cube,
            BosonObject::new({
                let mut rb = RigidBody::new(10.0, ColliderBuilder::Cube).expect("Failed");
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
            Model::from_obj("test_files/plane.obj", asset_manager.clone()).expect("Failed"),
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
    });
}

fn update(_isotope: &mut Isotope) {}

fn main() {
    let mut app = new_isotope(init, update).expect("Failed");

    _ = start_isotope(&mut app);
}
