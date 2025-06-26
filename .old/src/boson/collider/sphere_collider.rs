use std::sync::Arc;

use cgmath::Vector3;
use obj_2_rust::obj_2_rust;
use wgpu::RenderPass;

use crate::{GpuController, element::model_vertex::ModelVertex};

use super::{
    Collidable, Collider, CollisionPoints,
    debug_renderer::{DebugRender, ColliderDebugRenderer},
    test_sphere_cube, test_sphere_plane, test_sphere_sphere,
};

const SPHERE: ([(f32, f32, f32); 482], [u32; 2880], usize) =
    obj_2_rust!("isotope/src/boson/collider/collider_objs/sphere_collider.obj");

#[derive(Debug)]
pub struct SphereCollider {
    pub(crate) center: Vector3<f32>,
    pub(crate) radius: f32,

    // For Debug Rendering
    pub(crate) debug_renderer: ColliderDebugRenderer,
}

impl SphereCollider {
    pub(crate) fn new(
        center: Vector3<f32>,
        radius: f32,
        gpu_controller: Arc<GpuController>,
    ) -> Self {
        // Create the debug renderer for this collider
        let debug_renderer = ColliderDebugRenderer::new(
            &{
                let mut verts = SPHERE
                    .0
                    .into_iter()
                    .map(|vertex| vertex.into())
                    .collect::<Vec<ModelVertex>>();

                for vert in verts.iter_mut() {
                    vert.position[0] *= radius;
                    vert.position[1] *= radius;
                    vert.position[2] *= radius;
                }

                verts
            },
            &SPHERE.1,
            center.into(),
            [0.0, 0.0, 0.0, 1.0],
            gpu_controller,
        );

        Self {
            center,
            radius,
            debug_renderer,
        }
    }
}

impl Collidable for SphereCollider {
    fn test_with_collider(&self, collider: &Collider) -> Option<CollisionPoints> {
        match collider {
            Collider::Empty => None,
            Collider::Sphere(sphere_collider) => test_sphere_sphere(self, sphere_collider),
            Collider::Plane(plane_collider) => test_sphere_plane(self, plane_collider),
            Collider::Cube(cube_collider) => test_sphere_cube(self, cube_collider),
        }
    }
}

impl DebugRender for SphereCollider {
    fn debug_render(&self, render_pass: &mut RenderPass) {
        self.debug_renderer.update_pos(self.center);
        self.debug_renderer.render(render_pass);
    }
}
