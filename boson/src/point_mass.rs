use cgmath::{InnerSpace, MetricSpace, Vector3};
use log::info;

use crate::properties::gravity::{GRAVITATIONAL_CONSTANT, Gravitational, Gravity};

pub struct PointMass {
    pub position: Vector3<f64>,
    pub velocity: Vector3<f64>,
    pub acceleration: Vector3<f64>,

    pub mass: f64,
    pub inv_mass: f64,
}

impl Gravitational for PointMass {
    fn apply_gravity(&mut self, gravity: &Gravity, timestep: f64) {
        match gravity {
            Gravity::None => {}
            Gravity::World(gravity_vector) => {
                self.apply_force(*gravity_vector, timestep);
            }
            Gravity::Point(location, mass) => {
                let distance = self.position.distance(*location);
                let force = GRAVITATIONAL_CONSTANT * self.mass * mass / distance.powi(2);
                self.apply_force((self.position - location).normalize_to(force), timestep);
            }
            Gravity::WorldPoint(gravity_vector, location, mass) => {
                let distance = self.position.distance(*location);
                let force = GRAVITATIONAL_CONSTANT * self.mass * mass / distance.powi(2);
                let point_force = (self.position - location).normalize_to(force);
                self.apply_force(point_force + *gravity_vector, timestep);
            }
        }
    }
}

impl PointMass {
    pub fn new(mass: f64) -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            acceleration: Vector3::new(0.0, 0.0, 0.0),

            mass,
            inv_mass: if mass == 0.0 { 0.0 } else { 1.0 / mass },
        }
    }

    pub fn apply_force(&mut self, force: Vector3<f64>, timestep: f64) {
        info!("Applying Force: {:#?}", force);
        if self.mass == 0.0 {
            return;
        }

        // a = F / m
        self.acceleration = force / self.mass;

        // v = v_0 + a * t
        self.velocity += self.acceleration * timestep;

        // x = x_0 + v * t
        self.position += self.velocity * timestep;
    }
}
