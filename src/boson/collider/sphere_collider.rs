use cgmath::Vector3;

use super::{
    Collidable, Collider, CollisionPoints, test_sphere_cube, test_sphere_plane, test_sphere_sphere,
};

#[derive(Debug)]
pub struct SphereCollider {
    pub(crate) center: Vector3<f32>,
    pub(crate) radius: f32,
}

impl Collidable for SphereCollider {
    fn test_with_collider(&self, collider: &Collider) -> Option<CollisionPoints> {
        match collider {
            Collider::Sphere(sphere_collider) => test_sphere_sphere(self, sphere_collider),
            Collider::Plane(plane_collider) => test_sphere_plane(self, plane_collider),
            Collider::Cube(cube_collider) => test_sphere_cube(self, cube_collider),
        }
    }
}
