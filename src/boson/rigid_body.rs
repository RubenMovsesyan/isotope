use std::time::Instant;

use cgmath::Vector3;

use super::{DynamicObject, Linkable};

#[derive(Debug)]
pub struct RigidBody {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,

    pub mass: f32,
}

impl DynamicObject for RigidBody {
    fn apply_force(&mut self, force: Vector3<f32>, delta_t: &Instant) {
        // For simplicity
        let dt = delta_t.elapsed().as_secs_f32();

        // v = v_0 + F/m * t
        self.velocity += (force / self.mass) * dt;
        // x = x_0 + v * t
        self.position += self.velocity * dt;
    }

    #[inline]
    fn get_mass(&self) -> f32 {
        self.mass
    }
}

impl Linkable for RigidBody {
    fn get_position(&self) -> Vector3<f32> {
        self.position
    }
}
