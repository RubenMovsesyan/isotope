use cgmath::{InnerSpace, MetricSpace, Vector3, Zero};
use log::debug;

use crate::BosonBody;

pub const GRAVITATIONAL_CONSTANT: f64 = 6.674e-11;

pub enum Gravity {
    None,
    World {
        gravitational_acceleration: Vector3<f64>,
    },
    Point {
        location: Vector3<f64>,
        mass: f64,
    },
    WorldPoint {
        gravitational_acceleration: Vector3<f64>,
        location: Vector3<f64>,
        mass: f64,
    },
}

pub trait Gravitational {
    fn apply_gravity(&mut self, gravity: &Gravity, timestep: f64);
}

impl Gravitational for BosonBody {
    fn apply_gravity(&mut self, gravity: &Gravity, timestep: f64) {
        match self {
            BosonBody::RigidBody(rigid_body) => {
                match gravity {
                    Gravity::None => {}
                    Gravity::World {
                        gravitational_acceleration,
                    } => {
                        rigid_body.apply_acceleration(*gravitational_acceleration, timestep);
                    }
                    Gravity::Point { location, mass } => {
                        let distance = rigid_body.position.distance(*location);
                        if distance < 1e-6 {
                            return; // Avoid division by zero
                        }

                        let force =
                            GRAVITATIONAL_CONSTANT * rigid_body.mass * mass / distance.powi(2);
                        rigid_body.apply_force(
                            (location - rigid_body.position).normalize_to(force),
                            timestep,
                        );
                    }
                    Gravity::WorldPoint {
                        gravitational_acceleration,
                        location,
                        mass,
                    } => {
                        let world_accel = *gravitational_acceleration;

                        let distance = rigid_body.position.distance(*location);

                        let point_accel = if distance >= 1e-6 {
                            let force = GRAVITATIONAL_CONSTANT * mass / distance.powi(2);
                            (location - rigid_body.position).normalize_to(force)
                        } else {
                            Vector3::zero()
                        };

                        let total_acceleration = world_accel + point_accel;

                        rigid_body.apply_acceleration(total_acceleration, timestep);
                    }
                }
            }
            _ => {}
        }
    }
}
