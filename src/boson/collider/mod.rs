use cgmath::{InnerSpace, Quaternion, Vector3};
use plane_collider::PlaneCollider;
use sphere_collider::SphereCollider;

use super::BosonObject;

pub mod plane_collider;
pub mod sphere_collider;

// Helper struct for the engine
#[derive(Debug)]
pub struct Collision {
    pub(super) object_a: BosonObject,
    pub(super) object_b: BosonObject,
    pub(super) points: CollisionPoints,
}

#[derive(Debug, Copy, Clone)]
pub struct CollisionPoints {
    pub(crate) a_deep: Vector3<f32>, // Furthest point of a into b
    pub(crate) b_deep: Vector3<f32>, // Furthest point of b into a
    pub(crate) normal: Vector3<f32>, // b - a normalized
    pub(crate) depth: f32,           // length of b - a
}

#[derive(Debug)]
pub enum Collider {
    Sphere(SphereCollider),
    Plane(PlaneCollider),
}

pub(crate) trait Collidable {
    fn test_with_collider(&self, collider: &Collider) -> Option<CollisionPoints>;
}

impl Collider {
    pub fn new_sphere(position: Vector3<f32>, radius: f32) -> Self {
        Self::Sphere(SphereCollider {
            center: position,
            radius,
        })
    }

    pub fn new_plane(normal: Vector3<f32>, distance: f32) -> Self {
        Self::Plane(PlaneCollider { normal, distance })
    }

    pub fn test_collision(&self, other: &Collider) -> Option<CollisionPoints> {
        match self {
            Collider::Sphere(sphere_collider) => sphere_collider.test_with_collider(other),
            Collider::Plane(plane_collider) => plane_collider.test_with_collider(other),
        }
    }

    pub(crate) fn link_pos(&mut self, position: &Vector3<f32>) {
        match self {
            Collider::Sphere(sphere_collider) => sphere_collider.center = *position,
            Collider::Plane(_plane_collider) => {}
        }
    }

    pub(crate) fn link_rot(&mut self, rotation: &Quaternion<f32>) {
        match self {
            Collider::Sphere(_) => {}
            Collider::Plane(_) => {}
        }
    }
}

fn test_sphere_sphere(
    sphere_1_collider: &SphereCollider,
    sphere_2_collider: &SphereCollider,
) -> Option<CollisionPoints> {
    let sphere_center_difference = sphere_2_collider.center - sphere_1_collider.center;
    let sphere_center_distance = sphere_center_difference.magnitude();

    if sphere_center_distance <= sphere_1_collider.radius + sphere_2_collider.radius {
        let normal = sphere_center_difference.normalize();
        let a_deep = sphere_1_collider.center + normal * sphere_1_collider.radius;
        let b_deep = sphere_2_collider.center - normal * sphere_2_collider.radius;
        return Some(CollisionPoints {
            a_deep,
            b_deep,
            normal,
            depth: sphere_center_distance,
        });
    }

    None
}

fn test_sphere_plane(
    sphere_collider: &SphereCollider,
    plane_collider: &PlaneCollider,
) -> Option<CollisionPoints> {
    let norm = plane_collider.normal.normalize();

    // Calculate a point on the plane
    let plane_point = norm * plane_collider.distance;

    // Calculate the signed distance from the sphere center to the plane
    // (positive if sphere center is on the side the normal points to)
    let signed_distance = (sphere_collider.center - plane_point).dot(norm);

    // For collision detection, we need the absolute distance
    let distance = signed_distance.abs();

    if distance <= sphere_collider.radius {
        let collision_normal = if signed_distance >= 0.0 { norm } else { -norm };

        return Some(CollisionPoints {
            a_deep: sphere_collider.center - collision_normal * sphere_collider.radius,
            b_deep: sphere_collider.center - collision_normal * distance,
            normal: collision_normal,
            depth: sphere_collider.radius - distance,
        });
    }

    None
}
