use cgmath::{Matrix3, Quaternion, Vector3, Zero};
use log::debug;

use crate::{BosonBody, BosonObject, collider::Collider};

#[derive(Default)]
pub struct RigidBodyBuilder {
    // Linear
    position: Option<Vector3<f64>>,
    velocity: Option<Vector3<f64>>,
    mass: Option<f64>,
    restitution: Option<f64>,
    static_friction: Option<f64>,
    dynamic_friction: Option<f64>,

    // Angular
    orientation: Option<Quaternion<f64>>,
    angular_velocity: Option<Vector3<f64>>,
    inertia: Option<Matrix3<f64>>,

    collider: Option<Collider>,
}

impl RigidBodyBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn position(mut self, position: impl Into<Vector3<f64>>) -> Self {
        self.position = Some(position.into());
        self
    }

    pub fn velocity(mut self, velocity: impl Into<Vector3<f64>>) -> Self {
        self.velocity = Some(velocity.into());
        self
    }

    pub fn mass(mut self, mass: f64) -> Self {
        self.mass = Some(mass);
        self
    }

    pub fn restitution(mut self, restitution: f64) -> Self {
        self.restitution = Some(restitution);
        self
    }

    pub fn static_friction(mut self, static_friction: f64) -> Self {
        self.static_friction = Some(static_friction);
        self
    }

    pub fn dynamic_friction(mut self, dynamic_friction: f64) -> Self {
        self.dynamic_friction = Some(dynamic_friction);
        self
    }

    pub fn orientation(mut self, orientation: impl Into<Quaternion<f64>>) -> Self {
        self.orientation = Some(orientation.into());
        self
    }

    pub fn angular_velocity(mut self, angular_velocity: impl Into<Vector3<f64>>) -> Self {
        self.angular_velocity = Some(angular_velocity.into());
        self
    }

    pub fn inertia(mut self, inertia: impl Into<Matrix3<f64>>) -> Self {
        self.inertia = Some(inertia.into());
        self
    }

    pub fn collider(mut self, collider: Collider) -> Self {
        self.collider = Some(collider);
        self
    }

    pub fn build(self) -> BosonObject {
        BosonObject::new(BosonBody::RigidBody(RigidBody {
            position: self.position.unwrap_or_else(|| Vector3::zero()),
            velocity: self.velocity.unwrap_or_else(|| Vector3::zero()),
            acceleration: Vector3::zero(),
            orientation: self
                .orientation
                .unwrap_or_else(|| Quaternion::new(1.0, 0.0, 0.0, 0.0)),
            angular_velocity: self.angular_velocity.unwrap_or_else(|| Vector3::zero()),
            inverse_inertia: Vector3::zero(),
            inertia_tensor: self.inertia.unwrap_or_else(|| Matrix3::zero()),
            mass: self.mass.unwrap_or(0.0),
            inv_mass: 1.0 / self.mass.unwrap_or(0.0),
            restitution: self.restitution.unwrap_or(0.0),
            static_friction: self.static_friction.unwrap_or(0.0),
            dynamic_friction: self.dynamic_friction.unwrap_or(0.0),
            collider: self.collider.unwrap_or_else(|| Collider::Empty),
        }))
    }
}

impl From<RigidBodyBuilder> for BosonObject {
    fn from(value: RigidBodyBuilder) -> Self {
        value.build()
    }
}

pub struct RigidBody {
    // Linear Kinematics
    pub position: Vector3<f64>,
    pub velocity: Vector3<f64>,
    pub acceleration: Vector3<f64>,

    // Angular Kinematics
    pub orientation: Quaternion<f64>,
    pub angular_velocity: Vector3<f64>,
    pub inverse_inertia: Vector3<f64>,
    pub inertia_tensor: Matrix3<f64>,

    // Physical properties
    pub mass: f64,
    pub inv_mass: f64,
    pub restitution: f64,
    pub static_friction: f64,
    pub dynamic_friction: f64,

    pub collider: Collider,
}

impl RigidBody {
    #[inline]
    pub fn update(&mut self, timestep: f64) {
        // x = x_0 + v * t
        self.position += self.velocity * timestep;
    }

    #[inline]
    pub fn apply_acceleration(&mut self, acceleration: Vector3<f64>, timestep: f64) {
        // v = v_0 + a * t
        self.velocity += acceleration * timestep;
    }

    pub fn apply_force(&mut self, force: Vector3<f64>, timestep: f64) {
        if self.mass == 0.0 {
            return;
        }

        let acceleration = force * self.inv_mass;

        self.apply_acceleration(acceleration, timestep);
    }
}
