use cgmath::Vector3;

use super::{Collidable, Collider, CollisionPoints, test_cube_plane, test_sphere_plane};

#[derive(Debug)]
pub struct PlaneCollider {
    pub(crate) normal: Vector3<f32>,
    pub(crate) distance: f32,
}

impl Collidable for PlaneCollider {
    fn test_with_collider(&self, collider: &Collider) -> Option<CollisionPoints> {
        match collider {
            Collider::Sphere(sphere_collider) => test_sphere_plane(sphere_collider, self),
            Collider::Plane(_plane_collider) => {
                None // TODO: implement
            }
            Collider::Cube(cube_collider) => test_cube_plane(cube_collider, self),
        }
    }
}
