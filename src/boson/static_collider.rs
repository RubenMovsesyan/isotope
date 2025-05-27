use cgmath::{One, Quaternion, Vector3};

use super::{BosonBody, collider::Collider};

#[derive(Debug)]
pub struct StaticCollider {
    pub(crate) position: Vector3<f32>,
    pub(crate) orientation: Quaternion<f32>,
    pub(crate) collider: Collider,
}

impl StaticCollider {
    pub fn new(position: Vector3<f32>, mut collider: Collider) -> Self {
        collider.link_pos(&position);
        Self {
            position,
            orientation: Quaternion::one(),
            collider,
        }
    }
}

impl Into<BosonBody> for StaticCollider {
    fn into(self) -> BosonBody {
        BosonBody::StaticCollider(self)
    }
}
