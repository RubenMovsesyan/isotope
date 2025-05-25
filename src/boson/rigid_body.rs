use std::time::Instant;

use cgmath::{ElementWise, InnerSpace, Quaternion, Rad, Rotation3, Vector3};

use super::{DynamicObject, Linkable};

const ANGULAR_ACCELERATION_THRESHOLD: f32 = 0.001;

#[derive(Debug)]
pub struct RigidBody {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,

    pub orientation: Quaternion<f32>,
    pub angular_velocity: Vector3<f32>,
    pub inverse_inertia: Vector3<f32>,

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

    fn apply_torque(&mut self, torque: Vector3<f32>, delta_t: &Instant) {
        let dt = delta_t.elapsed().as_secs_f32();

        // Calculate the angular acceleration from the torque
        let angular_acceleration = torque.mul_element_wise(self.inverse_inertia);

        // Accumulate angular velocity
        self.angular_velocity += angular_acceleration * dt;

        // Converte angular velocity to a quaternion change
        let angle = self.angular_velocity.magnitude() * dt;

        if angle > ANGULAR_ACCELERATION_THRESHOLD {
            // Avoiding division by 0
            let axis = self.angular_velocity.normalize();
            let rotation = Quaternion::from_axis_angle(axis, Rad(angle));

            // Apply the rotation
            self.orientation = (rotation * self.orientation).normalize();
        }
    }

    fn pos(&mut self) -> &mut Vector3<f32> {
        &mut self.position
    }

    fn vel(&mut self) -> &mut Vector3<f32> {
        &mut self.velocity
    }

    #[inline(always)]
    fn get_mass(&self) -> f32 {
        self.mass
    }
}

impl Linkable for RigidBody {
    fn get_position(&self) -> Vector3<f32> {
        self.position
    }

    fn get_rotation(&self) -> Quaternion<f32> {
        self.orientation
    }
}
