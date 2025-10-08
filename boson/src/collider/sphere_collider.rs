use super::Collider;

pub struct SphereCollider;

impl SphereCollider {
    pub fn new() -> Collider {
        Collider::Sphere(SphereCollider)
    }
}
