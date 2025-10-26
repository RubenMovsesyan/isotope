use cube_collider::CubeCollider;
use plane_collider::PlaneCollider;
use sphere_collider::SphereCollider;

mod cube_collider;
mod plane_collider;
mod sphere_collider;

pub enum Collider {
    Empty,
    Sphere(SphereCollider),
    Plane(PlaneCollider),
    Cube(CubeCollider),
}
