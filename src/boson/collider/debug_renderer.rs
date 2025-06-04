use std::sync::Arc;

use wgpu::{
    Buffer, BufferUsages, PolygonMode, RenderPass,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    bind_group_builder,
    element::{
        buffered::Buffered, mesh::INDEX_FORMAT, model::ModelInstance, model_vertex::ModelVertex,
    },
    gpu_utils::GpuController,
    photon::render_descriptor::{
        PhotonRenderDescriptor, PhotonRenderDescriptorBuilder, STORAGE_RO,
    },
};

// Imple for all colliders
pub(crate) trait DebugRender {
    fn debug_render(&self, render_pass: &mut RenderPass);
}

#[derive(Debug)]
pub(crate) struct DebugRenderer {
    // For collider model
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    position_buffer: Buffer,
    orientation_buffer: Buffer,
    index_buffer_len: u32,

    debug_render_descriptor: PhotonRenderDescriptor,
}

impl DebugRenderer {
    pub(crate) fn new(
        vertices: &[ModelVertex],
        indices: &[u32],
        position: [f32; 3],
        orientation: [f32; 4],
        gpu_controller: Arc<GpuController>,
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

        let debug_render_descriptor = PhotonRenderDescriptorBuilder::default()
            .with_label("Boson Debugger")
            .with_polygon_mode(PolygonMode::Fill)
            .with_vertex_shader(include_str!("debugging_shaders/collider_vert_debug.wgsl"))
            .with_fragment_shader(include_str!("debugging_shaders/collider_frag_debug.wgsl"))
            .with_vertex_buffer_layouts(&[ModelVertex::desc(), ModelInstance::desc()])
            .add_bind_group_with_layout(bind_group_builder!(
                gpu_controller.device,
                "Boson Debugger",
                (0, VERTEX, position_buffer.as_entire_binding(), STORAGE_RO),
                (
                    1,
                    VERTEX,
                    orientation_buffer.as_entire_binding(),
                    STORAGE_RO
                )
            ))
            .build(gpu_controller.clone());

        Self {
            vertex_buffer,
            index_buffer,
            index_buffer_len: indices.len() as u32,
            position_buffer,
            orientation_buffer,
            debug_render_descriptor,
        }
    }

    #[inline]
    pub(crate) fn update_pos(&self, position: impl Into<[f32; 3]>) {
        self.debug_render_descriptor.write_buffer(
            &self.position_buffer,
            bytemuck::cast_slice(&position.into()),
        );
    }

    #[inline]
    pub(crate) fn update_rot(&self, orientation: impl Into<[f32; 4]>) {
        self.debug_render_descriptor.write_buffer(
            &self.orientation_buffer,
            bytemuck::cast_slice(&orientation.into()),
        );
    }

    #[inline]
    pub(crate) fn render(&self, render_pass: &mut RenderPass) {
        // Render the Collider Visualization
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), INDEX_FORMAT);

        self.debug_render_descriptor.setup_render(render_pass);

        render_pass.draw_indexed(0..self.index_buffer_len, 0, 0..1);
    }
}
