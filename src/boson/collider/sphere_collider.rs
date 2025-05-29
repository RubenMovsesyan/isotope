use std::sync::Arc;

use cgmath::Vector3;
use log::debug;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferUsages, RenderPass,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    GpuController, element::model_vertex::ModelVertex,
    photon::renderer::photon_layouts::PhotonLayoutsManager,
};

use super::{
    Collidable, Collider, CollisionPoints, test_sphere_cube, test_sphere_plane, test_sphere_sphere,
};

const SPHERE_VERTICES: [ModelVertex; 6] = [
    ModelVertex::new_const([0.0, 1.0, 0.0], [0.0, 0.0], [0.0, 1.0, 0.0]),
    ModelVertex::new_const([1.0, 0.0, 0.0], [0.0, 0.0], [1.0, 0.0, 0.0]),
    ModelVertex::new_const([0.0, -1.0, 0.0], [0.0, 0.0], [0.0, -1.0, 0.0]),
    ModelVertex::new_const([-1.0, 0.0, 0.0], [0.0, 0.0], [-1.0, 0.0, 0.0]),
    ModelVertex::new_const([0.0, 0.0, 1.0], [0.0, 0.0], [0.0, 0.0, 1.0]),
    ModelVertex::new_const([0.0, 0.0, -1.0], [0.0, 0.0], [0.0, 0.0, -1.0]),
];

const SPHERE_INDICES: [u32; 24] = [
    0, 1, 4, 1, 2, 4, 2, 3, 4, 3, 0, 4, 0, 1, 5, 1, 2, 5, 2, 3, 5, 3, 0, 5,
];

#[derive(Debug)]
pub struct SphereCollider {
    pub(crate) center: Vector3<f32>,
    pub(crate) radius: f32,

    pub(crate) vertex_buffer: Buffer,
    pub(crate) index_buffer: Buffer,
    pub(crate) position_buffer: Buffer,
    pub(crate) orientation_buffer: Buffer,

    pub(crate) bind_group: BindGroup,
    gpu_controller: Arc<GpuController>,
}

impl SphereCollider {
    pub(crate) fn new(
        center: Vector3<f32>,
        radius: f32,
        gpu_controller: Arc<GpuController>,
        photon_layout_manager: &PhotonLayoutsManager,
    ) -> Self {
        let vertex_buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Sphere Collider Vertex Buffer"),
                usage: BufferUsages::VERTEX,
                contents: bytemuck::cast_slice(&{
                    let mut verts: [ModelVertex; 6] = SPHERE_VERTICES;

                    for vert in verts.iter_mut() {
                        vert.position[0] *= radius;
                        vert.position[1] *= radius;
                        vert.position[2] *= radius;
                    }

                    verts
                }),
            });

        let index_buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Sphere Collider Index Buffer"),
                usage: BufferUsages::INDEX,
                contents: bytemuck::cast_slice(&SPHERE_INDICES),
            });

        let position: [f32; 3] = center.into();

        let position_buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Sphere Collider Position Buffer"),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice::<f32, u8>(&position),
            });

        let orientation_buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Sphere Collider Orientation Buffer"),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&[0.0, 0.0, 0.0, 1.0]),
            });

        let bind_group = gpu_controller
            .device
            .create_bind_group(&BindGroupDescriptor {
                label: Some("Sphere Collider Bind Group"),
                layout: &photon_layout_manager.collider_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: position_buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: orientation_buffer.as_entire_binding(),
                    },
                ],
            });

        Self {
            center,
            radius,
            vertex_buffer,
            index_buffer,
            position_buffer,
            orientation_buffer,
            bind_group,
            gpu_controller,
        }
    }

    pub(crate) fn render(&self, render_pass: &mut RenderPass) {
        let position: [f32; 3] = self.center.into();

        self.gpu_controller.queue.write_buffer(
            &self.position_buffer,
            0,
            bytemuck::cast_slice(&position),
        );

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.set_bind_group(1, &self.bind_group, &[]);

        render_pass.draw_indexed(0..24, 0, 0..1);
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
