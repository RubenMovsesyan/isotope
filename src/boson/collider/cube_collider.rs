use cgmath::{Quaternion, Rotation, Vector3, Zero};

use super::{
    Collidable, Collider, CollisionPoints, test_cube_cube, test_cube_plane, test_sphere_cube,
};

const NUM_VERTICES: usize = 8;

#[derive(Debug)]
pub struct CubeCollider {
    pub(crate) edge_length: f32,
    pub(crate) center: Vector3<f32>,
    pub(crate) orientation: Quaternion<f32>,
}

impl CubeCollider {
    pub(crate) fn get_vertices(&self) -> [Vector3<f32>; NUM_VERTICES] {
        let mut out = [Vector3::zero(); NUM_VERTICES];
        let size = self.edge_length / 2.0;

        for (index, vertex) in out.iter_mut().enumerate() {
            let (x, y, z) = (
                if index & 0x01 > 0 { size } else { -size },
                if index & 0x02 > 0 { size } else { -size },
                if index & 0x03 > 0 { size } else { -size },
            );

            *vertex = self.orientation.rotate_vector(Vector3 { x, y, z }) + self.center
        }

        out
    }
}

impl Collidable for CubeCollider {
    fn test_with_collider(&self, collider: &Collider) -> Option<CollisionPoints> {
        match collider {
            Collider::Empty => None,
            Collider::Sphere(sphere_collider) => test_sphere_cube(sphere_collider, self),
            Collider::Plane(plane_collider) => test_cube_plane(self, plane_collider),
            Collider::Cube(cube_collider) => test_cube_cube(self, cube_collider),
        }
    }
}
