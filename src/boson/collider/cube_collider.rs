use std::sync::Arc;

use cgmath::{Quaternion, Rotation, Vector3, Zero};
use obj_2_rust::obj_2_rust;

use crate::{element::model_vertex::ModelVertex, gpu_utils::GpuController};

use super::{
    Collidable, Collider, CollisionPoints,
    debug_renderer::{DebugRender, DebugRenderer},
    test_cube_cube, test_cube_plane, test_sphere_cube,
};

const NUM_VERTICES: usize = 8;

const CUBE: ([(f32, f32, f32); NUM_VERTICES], [u32; 36], usize) =
    obj_2_rust!("isotope/src/boson/collider/collider_objs/cube_collider.obj");

#[derive(Debug)]
pub struct CubeCollider {
    pub(crate) edge_length: f32,
    pub(crate) center: Vector3<f32>,
    pub(crate) orientation: Quaternion<f32>,

    // For Debug Rendering
    pub(crate) debug_renderer: DebugRenderer,
}

impl CubeCollider {
    pub(crate) fn new(
        center: Vector3<f32>,
        edge_length: f32,
        orientation: Quaternion<f32>,
        gpu_controller: Arc<GpuController>,
    ) -> Self {
        // Create the debug renderer for this collider
        let debug_renderer = DebugRenderer::new(
            &{
                let mut verts = CUBE
                    .0
                    .into_iter()
                    .map(|vertex| vertex.into())
                    .collect::<Vec<ModelVertex>>();

                for vert in verts.iter_mut() {
                    vert.position[0] *= edge_length / 2.0;
                    vert.position[1] *= edge_length / 2.0;
                    vert.position[2] *= edge_length / 2.0;
                }

                verts
            },
            &CUBE.1,
            center.into(),
            orientation.into(),
            gpu_controller,
        );

        Self {
            center,
            edge_length,
            orientation,
            debug_renderer,
        }
    }

    pub(crate) fn get_vertices(&self) -> [Vector3<f32>; NUM_VERTICES] {
        let mut out = [Vector3::zero(); NUM_VERTICES];
        let size = self.edge_length / 2.0;

        for (index, vertex) in out.iter_mut().enumerate() {
            let (x, y, z) = (
                if index & 0x01 > 0 { size } else { -size },
                if index & 0x02 > 0 { size } else { -size },
                if index & 0x04 > 0 { size } else { -size },
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

impl DebugRender for CubeCollider {
    fn debug_render(&self, render_pass: &mut wgpu::RenderPass) {
        self.debug_renderer.update_pos(self.center);
        self.debug_renderer.update_rot(self.orientation);
        self.debug_renderer.render(render_pass);
    }
}
