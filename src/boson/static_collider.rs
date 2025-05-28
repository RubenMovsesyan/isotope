use cgmath::{One, Quaternion, Vector3};

use super::{BosonBody, collider::Collider};

#[derive(Debug)]
pub struct StaticCollider {
    pub(crate) position: Vector3<f32>,
    pub(crate) orientation: Quaternion<f32>,
    pub(crate) collider: Collider,

    // Physics properties
    pub(crate) static_friction: f32,
    pub(crate) dynamic_friction: f32,
}

impl StaticCollider {
    pub fn new(position: Vector3<f32>, mut collider: Collider) -> Self {
        collider.link_pos(&position);
        Self {
            position,
            orientation: Quaternion::one(),
            collider,
            static_friction: 0.2, // Default for now
            dynamic_friction: 0.1,
        }
    }
}

impl Into<BosonBody> for StaticCollider {
    fn into(self) -> BosonBody {
        BosonBody::StaticCollider(self)
    }
}
