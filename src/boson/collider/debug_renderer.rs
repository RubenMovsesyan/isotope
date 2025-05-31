use std::sync::Arc;

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferUsages, RenderPass,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    element::model_vertex::ModelVertex, gpu_utils::GpuController,
    photon::renderer::photon_layouts::PhotonLayoutsManager,
};

// Imple for all colliders
pub(crate) trait DebugRender {
    fn debug_render(&self, render_pass: &mut RenderPass);
}

#[derive(Debug)]
pub(crate) struct DebugRenderer {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    position_buffer: Buffer,
    orientation_buffer: Buffer,
    bind_group: BindGroup,
    index_buffer_len: u32,
    gpu_controller: Arc<GpuController>,
}

impl DebugRenderer {
    pub(crate) fn new(
        vertices: &[ModelVertex],
        indices: &[u32],
        position: [f32; 3],
        orientation: [f32; 4],
        gpu_controller: Arc<GpuController>,
        photon_layout_manager: &PhotonLayoutsManager,
    ) -> Self {
        let vertex_buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Debug Collider Vertex Buffer"),
                usage: BufferUsages::VERTEX,
                contents: bytemuck::cast_slice(vertices),
            });

        let index_buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Debug Collider Index Buffer"),
                usage: BufferUsages::INDEX,
                contents: bytemuck::cast_slice(indices),
            });

        let position_buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Debug Collider Postion Buffer"),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&position),
            });

        let orientation_buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Debug Collider Orientation Buffer"),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&orientation),
            });

        let bind_group = gpu_controller
            .device
            .create_bind_group(&BindGroupDescriptor {
                label: Some("Debug Collider Bind Group"),
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
            vertex_buffer,
            index_buffer,
            index_buffer_len: indices.len() as u32,
            position_buffer,
            orientation_buffer,
            bind_group,
            gpu_controller,
        }
    }

    #[inline]
    pub(crate) fn update_pos(&self, position: impl Into<[f32; 3]>) {
        self.gpu_controller.queue.write_buffer(
            &self.position_buffer,
            0,
            bytemuck::cast_slice(&position.into()),
        );
    }

    #[inline]
    pub(crate) fn update_rot(&self, orientation: impl Into<[f32; 4]>) {
        self.gpu_controller.queue.write_buffer(
            &self.orientation_buffer,
            0,
            bytemuck::cast_slice(&orientation.into()),
        );
    }

    #[inline]
    pub(crate) fn render(&self, render_pass: &mut RenderPass) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.set_bind_group(1, &self.bind_group, &[]);

        render_pass.draw_indexed(0..self.index_buffer_len, 0, 0..1);
    }
}
