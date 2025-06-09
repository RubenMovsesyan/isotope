use std::{sync::Arc, time::Instant};

use cgmath::{ElementWise, InnerSpace, Matrix3, One, Quaternion, Rad, Rotation3, Vector3, Zero};
use wgpu::RenderPass;

use crate::{ColliderBuilder, Instancer, element::model::ModelInstance};

use super::{
    BosonBody, Linkable,
    collider::{Collider, CollisionPoints},
    debug_renderer::BosonDebugRenderer,
};

const ANGULAR_ACCELERATION_THRESHOLD: f32 = 0.001;

#[derive(Debug)]
pub struct RigidBody {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub current_acceleration: Vector3<f32>,
    pub scale_factor: f32,

    pub orientation: Quaternion<f32>,
    pub angular_velocity: Vector3<f32>,
    pub inverse_inertia: Vector3<f32>,
    pub inertia_tensor: Matrix3<f32>,

    // Physical Properties
    pub(crate) mass: f32,
    pub(crate) inv_mass: f32,
    pub(crate) restitution: f32,
    pub(crate) static_friction: f32,
    pub(crate) dynamic_friction: f32,
    pub(crate) gravity: Vector3<f32>,

    pub(crate) collider: Collider,
    pub collider_builder: ColliderBuilder,

    pub(crate) debug_renderer: Option<BosonDebugRenderer>,
}

impl RigidBody {
    pub fn new(mass: f32, collider_builder: ColliderBuilder) -> Self {
        Self {
            position: Vector3::zero(),
            velocity: Vector3::zero(),
            current_acceleration: Vector3::zero(),
            scale_factor: 1.0,
            orientation: Quaternion::one(),
            angular_velocity: Vector3::zero(),
            inverse_inertia: Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            // inertia_tensor: {
            //     let mat = Matrix3::one();
            //     (2.0 / 5.0) * mass * 1.0 * 1.0 * mat // Temp
            // },
            // inertia_tensor: Matrix3 {
            //     x: Vector3::new(2.0 / 3.0, -1.0 / 4.0, -1.0 / 4.0),
            //     y: Vector3::new(-1.0 / 4.0, 2.0 / 3.0, -1.0 / 4.0),
            //     z: Vector3::new(-1.0 / 4.0, -1.0 / 4.0, 2.0 / 3.0),
            // },
            inertia_tensor: Matrix3 {
                x: Vector3::new(mass * 4.0, 0.0, 0.0),
                y: Vector3::new(0.0, mass * 4.0, 0.0),
                z: Vector3::new(0.0, 0.0, mass * 4.0),
            },
            mass,
            inv_mass: 1.0 / mass,
            restitution: 0.01, // Defaults
            static_friction: 0.1,
            dynamic_friction: 0.05,
            gravity: Vector3 {
                x: 0.0,
                y: -9.81,
                z: 0.0,
            },
            collider: Collider::Empty,
            collider_builder,
            debug_renderer: None,
        }
    }

    pub fn apply_force(&mut self, force: Vector3<f32>, delta_t: &Instant) {
        // For simplicity
        let dt = delta_t.elapsed().as_secs_f32();

        self.current_acceleration = force / self.mass;

        // v = v_0 + F/m * t
        self.velocity += self.current_acceleration * dt;
        // x = x_0 + v * t
        self.position += self.velocity * dt;

        // Link the position to the collider
        self.collider.link_pos(&self.position);
    }

    pub fn apply_torque(&mut self, torque: Vector3<f32>, delta_t: &Instant) {
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

            // Link the rotation to the collider
            self.collider.link_rot(&self.orientation);
        }
    }

    // regular update with no forces in place
    pub fn update(&mut self, delta_t: &Instant) {
        let dt = delta_t.elapsed().as_secs_f32();

        self.current_acceleration = self.gravity;

        // Gravity
        self.velocity += self.current_acceleration * dt;

        self.position += self.velocity * dt;
        self.collider.link_pos(&self.position);

        let angle = self.angular_velocity.magnitude() * dt;

        if angle > ANGULAR_ACCELERATION_THRESHOLD {
            // Avoiding division by 0
            let axis = self.angular_velocity.normalize();
            let rotation = Quaternion::from_axis_angle(axis, Rad(angle));

            // Apply the rotation
            self.orientation = (rotation * self.orientation).normalize();

            // Link the rotation to the collider
            self.collider.link_rot(&self.orientation);
        }
    }

    pub(crate) fn debug_render(&self, render_pass: &mut RenderPass) {
        // Render the collider
        self.collider.debug_render(render_pass);

        // Render the velocity
        if let Some(debug_renderer) = self.debug_renderer.as_ref() {
            debug_renderer.update_pos(self.position);
            debug_renderer.update_vel(self.velocity);
            debug_renderer.update_acc(self.current_acceleration);
            debug_renderer.update_ang_vel(self.angular_velocity);

            debug_renderer.render(render_pass);
        }
    }

    pub fn test_collision(&self, other: &Collider) -> Option<CollisionPoints> {
        self.collider.test_collision(other)
    }
}

impl Linkable for RigidBody {
    fn get_position(&self) -> Vector3<f32> {
        self.position
    }

    fn get_orientation(&self) -> Quaternion<f32> {
        self.orientation
    }

    fn get_instancer(&self) -> Option<Arc<Instancer<ModelInstance>>> {
        None
    }
}

impl Into<BosonBody> for RigidBody {
    fn into(self) -> BosonBody {
        BosonBody::RigidBody(self)
    }
}
