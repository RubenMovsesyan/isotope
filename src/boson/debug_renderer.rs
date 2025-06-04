use std::sync::Arc;

use wgpu::{
    Buffer, BufferAddress, BufferDescriptor, BufferUsages, CompareFunction, DepthBiasState,
    DepthStencilState, PipelineCompilationOptions, PolygonMode, PrimitiveTopology, RenderPass,
    StencilState, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    bind_group_builder,
    element::{buffered::Buffered, mesh::INDEX_FORMAT},
    gpu_utils::GpuController,
    photon::{
        render_descriptor::{PhotonRenderDescriptor, PhotonRenderDescriptorBuilder, STORAGE_RO},
        renderer::texture::PHOTON_TEXTURE_DEPTH_FORMAT,
    },
};

const VECTOR_IND_LEN: u32 = 2;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct VectorVertex {
    vector_vertex: [f32; 3],
}

impl Buffered for VectorVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<VectorVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                // Vector Vertex
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
            ],
        }
    }
}

#[derive(Debug)]
pub(crate) struct BosonDebugRenderer {
    // For Velocity
    pub(crate) velocity_buffer: Buffer,

    // For Acceleration
    pub(crate) acceleration_buffer: Buffer,

    // For Angular Velocity
    pub(crate) angular_velocity_buffer: Buffer,

    // For Position
    position_buffer: Buffer,

    pub(crate) vector_index_buffer: Buffer,

    // For debug rendering
    velocity_render_descriptor: PhotonRenderDescriptor,
    acceleration_render_descriptor: PhotonRenderDescriptor,
    angular_velocity_render_descriptor: PhotonRenderDescriptor,
}

impl BosonDebugRenderer {
    pub(crate) fn new(gpu_controller: Arc<GpuController>) -> Self {
        let velocity_buffer = gpu_controller.device.create_buffer(&BufferDescriptor {
            label: Some("Boson Velocity Debugger"),
            mapped_at_creation: false,
            size: std::mem::size_of::<[f32; 6]>() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let acceleration_buffer = gpu_controller.device.create_buffer(&BufferDescriptor {
            label: Some("Boson Acceleration Debugger"),
            mapped_at_creation: false,
            size: std::mem::size_of::<[f32; 6]>() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let angular_velocity_buffer = gpu_controller.device.create_buffer(&BufferDescriptor {
            label: Some("Boson Acceleration Debugger"),
            mapped_at_creation: false,
            size: std::mem::size_of::<[f32; 6]>() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let vector_index_buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Boson Velocity Debugger Indices"),
                contents: bytemuck::cast_slice(&[0, 1]), // Vectors always only have 2 indices
                usage: BufferUsages::INDEX,
            });

        let position_buffer = gpu_controller.device.create_buffer(&BufferDescriptor {
            label: Some("Boson Position Debugger Buffer"),
            mapped_at_creation: false,
            size: std::mem::size_of::<[f32; 3]>() as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        let (position_bind_group_layout, position_bind_group) = bind_group_builder!(
            gpu_controller.device,
            "Boson Velocity Debugger",
            (0, VERTEX, position_buffer.as_entire_binding(), STORAGE_RO)
        );

        let velocity_render_descriptor = PhotonRenderDescriptorBuilder::default()
            .with_label("Boson Velocity Debugger")
            .with_polygon_mode(PolygonMode::Line)
            .with_vertex_shader(include_str!("shaders/debug/vector_vert_debug.wgsl"))
            .with_fragment_shader(include_str!("shaders/debug/vector_frag_debug.wgsl"))
            .with_vertex_buffer_layouts(&[VectorVertex::desc()])
            .with_primitive_topology(PrimitiveTopology::LineList)
            .with_fragment_pipeline_compilation_options(PipelineCompilationOptions {
                constants: &[("RED", 1.0), ("GREEN", 0.0), ("BLUE", 0.0)],
                ..Default::default()
            })
            .with_depth_stencil_state(DepthStencilState {
                bias: DepthBiasState::default(),
                depth_compare: CompareFunction::Always,
                depth_write_enabled: false,
                format: PHOTON_TEXTURE_DEPTH_FORMAT,
                stencil: StencilState::default(),
            })
            .add_bind_group_with_layout((
                position_bind_group_layout.clone(),
                position_bind_group.clone(),
            ))
            .build(gpu_controller.clone());

        let acceleration_render_descriptor = PhotonRenderDescriptorBuilder::default()
            .with_label("Boson Accleration Debugger")
            .with_polygon_mode(PolygonMode::Line)
            .with_vertex_shader(include_str!("shaders/debug/vector_vert_debug.wgsl"))
            .with_fragment_shader(include_str!("shaders/debug/vector_frag_debug.wgsl"))
            .with_fragment_pipeline_compilation_options(PipelineCompilationOptions {
                constants: &[("RED", 1.0), ("GREEN", 1.0), ("BLUE", 0.0)],
                ..Default::default()
            })
            .with_vertex_buffer_layouts(&[VectorVertex::desc()])
            .with_primitive_topology(PrimitiveTopology::LineList)
            .with_depth_stencil_state(DepthStencilState {
                bias: DepthBiasState::default(),
                depth_compare: CompareFunction::Always,
                depth_write_enabled: false,
                format: PHOTON_TEXTURE_DEPTH_FORMAT,
                stencil: StencilState::default(),
            })
            .add_bind_group_with_layout((
                position_bind_group_layout.clone(),
                position_bind_group.clone(),
            ))
            .build(gpu_controller.clone());

        let angular_velocity_render_descriptor = PhotonRenderDescriptorBuilder::default()
            .with_label("Boson Accleration Debugger")
            .with_polygon_mode(PolygonMode::Line)
            .with_vertex_shader(include_str!("shaders/debug/vector_vert_debug.wgsl"))
            .with_fragment_shader(include_str!("shaders/debug/vector_frag_debug.wgsl"))
            .with_fragment_pipeline_compilation_options(PipelineCompilationOptions {
                constants: &[("RED", 1.0), ("GREEN", 0.0), ("BLUE", 1.0)],
                ..Default::default()
            })
            .with_vertex_buffer_layouts(&[VectorVertex::desc()])
            .with_primitive_topology(PrimitiveTopology::LineList)
            .with_depth_stencil_state(DepthStencilState {
                bias: DepthBiasState::default(),
                depth_compare: CompareFunction::Always,
                depth_write_enabled: false,
                format: PHOTON_TEXTURE_DEPTH_FORMAT,
                stencil: StencilState::default(),
            })
            .add_bind_group_with_layout((position_bind_group_layout, position_bind_group))
            .build(gpu_controller);

        Self {
            velocity_buffer,
            acceleration_buffer,
            angular_velocity_buffer,
            vector_index_buffer,
            position_buffer,
            velocity_render_descriptor,
            acceleration_render_descriptor,
            angular_velocity_render_descriptor,
        }
    }

    #[inline]
    pub(crate) fn update_pos(&self, position: impl Into<[f32; 3]>) {
        self.velocity_render_descriptor.write_buffer(
            &self.position_buffer,
            bytemuck::cast_slice(&position.into()),
        );
    }

    #[inline]
    pub(crate) fn update_vel(&self, velocity: impl Into<[f32; 3]>) {
        unsafe {
            self.velocity_render_descriptor.write_buffer_offset(
                &self.velocity_buffer,
                std::mem::size_of::<[f32; 3]>() as u64,
                bytemuck::cast_slice(&velocity.into()),
            );
        }
    }

    #[inline]
    pub(crate) fn update_acc(&self, acceleration: impl Into<[f32; 3]>) {
        unsafe {
            self.acceleration_render_descriptor.write_buffer_offset(
                &self.acceleration_buffer,
                std::mem::size_of::<[f32; 3]>() as u64,
                bytemuck::cast_slice(&acceleration.into()),
            );
        }
    }

    #[inline]
    pub(crate) fn update_ang_vel(&self, angular_velocity: impl Into<[f32; 3]>) {
        unsafe {
            self.angular_velocity_render_descriptor.write_buffer_offset(
                &self.angular_velocity_buffer,
                std::mem::size_of::<[f32; 3]>() as u64,
                bytemuck::cast_slice(&angular_velocity.into()),
            );
        }
    }

    #[inline]
    pub(crate) fn render(&self, render_pass: &mut RenderPass) {
        // Index buffer is the same for all vectors
        render_pass.set_index_buffer(self.vector_index_buffer.slice(..), INDEX_FORMAT);

        // Render the Acceleration Visualization
        render_pass.set_vertex_buffer(0, self.acceleration_buffer.slice(..));

        self.acceleration_render_descriptor
            .setup_render(render_pass);

        render_pass.draw_indexed(0..VECTOR_IND_LEN, 0, 0..1);

        // Render the Velocity Visualization
        render_pass.set_vertex_buffer(0, self.velocity_buffer.slice(..));

        self.velocity_render_descriptor.setup_render(render_pass);

        render_pass.draw_indexed(0..VECTOR_IND_LEN, 0, 0..1);

        // Render the Angular Velocity Visualization
        render_pass.set_vertex_buffer(0, self.angular_velocity_buffer.slice(..));

        self.angular_velocity_render_descriptor
            .setup_render(render_pass);

        render_pass.draw_indexed(0..VECTOR_IND_LEN, 0, 0..1);
    }
}
