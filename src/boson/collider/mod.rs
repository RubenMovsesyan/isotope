use std::sync::Arc;

use cgmath::{InnerSpace, Quaternion, Vector3};
use cube_collider::CubeCollider;
use debug_renderer::DebugRender;
use plane_collider::PlaneCollider;
use sphere_collider::SphereCollider;

use wgpu::RenderPass;

use crate::GpuController;

use super::BosonObject;

pub mod cube_collider;
mod debug_renderer;
pub mod plane_collider;
pub mod sphere_collider;

// Helper struct for the engine
#[derive(Debug)]
pub struct Collision {
    pub(super) object_a: BosonObject,
    pub(super) object_b: BosonObject,
    pub(super) points: CollisionPoints,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub struct CollisionPoints {
    pub(crate) a_deep: Vector3<f32>, // Furthest point of a into b
    pub(crate) b_deep: Vector3<f32>, // Furthest point of b into a
    pub(crate) normal: Vector3<f32>, // b - a normalized
    pub(crate) depth: f32,           // length of b - a

    // For rotation
    pub(crate) contact_point: Vector3<f32>, // World-space contact point
}

#[derive(Debug)]
pub enum ColliderBuilder {
    Sphere,
    Plane,
    Cube,
}

#[derive(Debug)]
pub enum Collider {
    Empty,
    Sphere(SphereCollider),
    Plane(PlaneCollider),
    Cube(CubeCollider),
}

pub(crate) trait Collidable {
    fn test_with_collider(&self, collider: &Collider) -> Option<CollisionPoints>;
}

#[allow(dead_code)]
impl Collider {
    pub(crate) fn new_sphere(
        position: Vector3<f32>,
        radius: f32,
        gpu_controller: Arc<GpuController>,
    ) -> Self {
        Self::Sphere(SphereCollider::new(position, radius, gpu_controller))
    }

    pub fn new_plane(normal: Vector3<f32>, distance: f32) -> Self {
        Self::Plane(PlaneCollider { normal, distance })
    }

    pub(crate) fn new_cube(
        position: Vector3<f32>,
        orientation: Quaternion<f32>,
        edge_length: f32,
        gpu_controller: Arc<GpuController>,
    ) -> Self {
        Self::Cube(CubeCollider::new(
            position,
            edge_length,
            orientation,
            gpu_controller,
        ))
    }

    pub fn test_collision(&self, other: &Collider) -> Option<CollisionPoints> {
        match self {
            Collider::Empty => None,
            Collider::Sphere(sphere_collider) => sphere_collider.test_with_collider(other),
            Collider::Plane(plane_collider) => plane_collider.test_with_collider(other),
            Collider::Cube(cube_collider) => cube_collider.test_with_collider(other),
        }
    }

    pub(crate) fn link_pos(&mut self, position: &Vector3<f32>) {
        match self {
            Collider::Empty => {}
            Collider::Sphere(sphere_collider) => sphere_collider.center = *position,
            Collider::Plane(_plane_collider) => {}
            Collider::Cube(cube_collider) => cube_collider.center = *position,
        }
    }

    pub(crate) fn link_rot(&mut self, rotation: &Quaternion<f32>) {
        match self {
            Collider::Empty => {}
            Collider::Sphere(_) => {}
            Collider::Plane(_) => {}
            Collider::Cube(cube_collider) => cube_collider.orientation = *rotation,
        }
    }

    pub(crate) fn debug_render(&self, render_pass: &mut RenderPass) {
        match self {
            Collider::Empty => {}
            Collider::Sphere(sphere_collider) => sphere_collider.debug_render(render_pass),
            Collider::Plane(_) => {}
            Collider::Cube(cube_collider) => cube_collider.debug_render(render_pass),
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
        let contact_point = (a_deep + b_deep) * 0.5;

        return Some(CollisionPoints {
            a_deep,
            b_deep,
            normal,
            depth: sphere_center_distance,
            contact_point,
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
        let a_deep = sphere_collider.center - collision_normal * sphere_collider.radius;
        let b_deep = sphere_collider.center - collision_normal * distance;
        let contact_point = (a_deep + b_deep) * 0.5;

        return Some(CollisionPoints {
            a_deep,
            b_deep,
            normal: collision_normal,
            depth: sphere_collider.radius - distance,
            contact_point,
        });
    }

    None
}

#[allow(unused_variables)]
fn test_sphere_cube(
    sphere_collider: &SphereCollider,
    cube_collider: &CubeCollider,
) -> Option<CollisionPoints> {
    todo!()
}

fn test_cube_plane(
    cube_collider: &CubeCollider,
    plane_collider: &PlaneCollider,
) -> Option<CollisionPoints> {
    // For convenience
    let plane_normal = plane_collider.normal.normalize();
    let plane_distance = plane_collider.distance;

    let cube_vertices = cube_collider.get_vertices();

    // Returns the distance from the surface of the plane
    let check_point = |point: Vector3<f32>| -> f32 { plane_normal.dot(point) - plane_distance };

    fn most_negative_distance(distances: Vec<f32>) -> Option<(usize, f32)> {
        #[derive(PartialEq, Eq)]
        enum Sign {
            Positive,
            Negative,
        }

        fn get_sign(distance: f32) -> Sign {
            if distance < 0.0 {
                Sign::Negative
            } else {
                Sign::Positive
            }
        }

        let sign = get_sign(distances[0]);
        let mut colliding = false;
        let mut most_negative_distance = (0, distances[0]);

        for (point_index, point) in distances.iter().enumerate() {
            if get_sign(*point) != sign {
                colliding = true;
            }

            if *point < most_negative_distance.1 {
                most_negative_distance = (point_index, *point);
            }
        }

        if colliding {
            Some(most_negative_distance)
        } else {
            None
        }
    }

    let distances = cube_vertices
        .iter()
        .map(|vertex| check_point(*vertex))
        .collect::<Vec<f32>>();

    if let Some((point_index, point_distance)) = most_negative_distance(distances) {
        let depth = f32::abs(point_distance);

        let a_deep = cube_vertices[point_index];
        let b_deep = a_deep - plane_normal * depth;
        let contact_point = (a_deep + b_deep) * 0.5;

        Some(CollisionPoints {
            a_deep,
            b_deep,
            normal: plane_normal,
            depth,
            contact_point,
        })
    } else {
        None
    }

    // // Keep track of extreme points
    // let mut most_negative_distance = f32::MAX;
    // let mut most_negative_vertex = cube_vertices[0];

    // cube_vertices.iter().for_each(|vertex| {
    //     let vertex_plane_distance = vertex.dot(plane_normal) - plane_distance;

    //     if vertex_plane_distance < most_negative_distance {
    //         most_negative_distance = vertex_plane_distance;
    //         most_negative_vertex = *vertex;
    //     }
    // });

    // // If the most negative distance is negative, there is a collision
    // if most_negative_distance < 0.0 {
    //     let depth = -most_negative_distance;
    //     let a_deep = most_negative_vertex;
    //     let b_deep = a_deep - plane_normal * depth;
    //     let contact_point = (a_deep + b_deep) * 0.5;

    //     return Some(CollisionPoints {
    //         a_deep,
    //         b_deep,
    //         normal: plane_normal,
    //         depth,
    //         contact_point,
    //     });
    // }

    // None
}

#[allow(unused_variables)]
fn test_cube_cube(
    cube_1_collider: &CubeCollider,
    cube_2_collider: &CubeCollider,
) -> Option<CollisionPoints> {
    todo!()
}
