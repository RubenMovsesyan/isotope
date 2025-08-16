use std::sync::Arc;

use anyhow::Result;
use gpu_controller::{Buffered, GpuController, Instance, Vertex};
use wgpu::{
    BlendComponent, BlendFactor, BlendOperation, BlendState, Buffer, BufferUsages, Color,
    ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, Extent3d,
    Face, FragmentState, FrontFace, LoadOp, MultisampleState, Operations,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, StencilState, StoreOp, Texture,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, VertexState,
    util::BufferInitDescriptor, wgt::TextureViewDescriptor,
};

use crate::camera::{CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR, Camera};

use super::CAMERA_BIND_GROUP;

pub struct DeferedRenderer3D {
    gpu_controller: Arc<GpuController>,
    geometry_render_pipeline: RenderPipeline,
    depth_texture: Texture,

    // G-buffer textures
    albedo_texture: Texture,
    normal_texture: Texture,
    material_texture: Texture,
    // G-buffer bind group for lighting pass
    // gbuffer_bind_group: BindGroup,

    // TEMP
    instance_buffer: Buffer,
}

impl DeferedRenderer3D {
    pub(crate) fn new(gpu_controller: Arc<GpuController>) -> Result<Self> {
        let texture_size = gpu_controller.with_surface_config(|config| Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        })?;

        let albedo_texture = gpu_controller.create_texture(&TextureDescriptor {
            label: Some("G-Buffer Albedo"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let normal_texture = gpu_controller.create_texture(&TextureDescriptor {
            label: Some("G-Buffer Normal"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let material_texture = gpu_controller.create_texture(&TextureDescriptor {
            label: Some("G-Buffer Materials"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth_texture = gpu_controller.create_texture(&TextureDescriptor {
            label: Some("G-Buffer Depth"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let camera_bind_group_layout =
            gpu_controller.create_bind_group_layout(&CAMERA_BIND_GROUP_LAYOUT_DESCRIPTOR);

        let geometry_pipeline_layout =
            gpu_controller.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Geometry Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader_module =
            gpu_controller.create_shader(include_str!("shaders/defered_3d_geom.wgsl"));

        let geometry_render_pipeline =
            gpu_controller.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Defered Renderer Render Pipeline"),
                cache: None,
                multiview: None,
                layout: Some(&geometry_pipeline_layout),
                vertex: VertexState {
                    module: &shader_module,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::desc(), Instance::desc()],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment: Some(FragmentState {
                    module: &shader_module,
                    entry_point: Some("fs_main"),
                    targets: &[
                        // Albedo
                        Some(ColorTargetState {
                            format: TextureFormat::Rgba8UnormSrgb,
                            blend: Some(BlendState {
                                color: BlendComponent {
                                    src_factor: BlendFactor::SrcAlpha,
                                    dst_factor: BlendFactor::OneMinusSrcAlpha,
                                    operation: BlendOperation::Add,
                                },
                                alpha: BlendComponent::OVER,
                            }),
                            write_mask: ColorWrites::ALL,
                        }),
                        // Normals
                        Some(ColorTargetState {
                            format: TextureFormat::Rgba16Float,
                            blend: Some(BlendState {
                                color: BlendComponent {
                                    src_factor: BlendFactor::SrcAlpha,
                                    dst_factor: BlendFactor::OneMinusSrcAlpha,
                                    operation: BlendOperation::Add,
                                },
                                alpha: BlendComponent::OVER,
                            }),
                            write_mask: ColorWrites::ALL,
                        }),
                        // Material
                        Some(ColorTargetState {
                            format: TextureFormat::Rgba8Unorm,
                            blend: Some(BlendState {
                                color: BlendComponent {
                                    src_factor: BlendFactor::SrcAlpha,
                                    dst_factor: BlendFactor::OneMinusSrcAlpha,
                                    operation: BlendOperation::Add,
                                },
                                alpha: BlendComponent::OVER,
                            }),
                            write_mask: ColorWrites::ALL,
                        }),
                    ],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: Some(Face::Back),
                    polygon_mode: PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            });

        // TEMP
        let instance_buffer = gpu_controller.create_buffer_init(&BufferInitDescriptor {
            label: Some("Temp Instance Buffer"),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&[Instance::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0])]),
        });

        Ok(Self {
            albedo_texture,
            normal_texture,
            material_texture,
            depth_texture,
            geometry_render_pipeline,
            gpu_controller,
            instance_buffer,
        })
    }

    pub(crate) fn render<F>(
        &self,
        camera: &Camera,
        output: &Texture,
        geometry_callback: F,
    ) -> Result<()>
    where
        F: FnOnce(&mut RenderPass),
    {
        // Get the output texture for the renderer
        let view = output.create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .gpu_controller
            .create_command_encoder("Defered Render 3D Encoder");

        // Create views for G-Buffer
        let albedo_view = self
            .albedo_texture
            .create_view(&TextureViewDescriptor::default());
        let normal_view = self
            .normal_texture
            .create_view(&TextureViewDescriptor::default());
        let material_view = self
            .material_texture
            .create_view(&TextureViewDescriptor::default());

        // Run any needed gpu updates here

        // Geometry Pass
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Defered 3D Geometry Pass"),
                color_attachments: &[
                    Some(RenderPassColorAttachment {
                        view: &albedo_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::BLACK),
                            store: StoreOp::Store,
                        },
                    }),
                    Some(RenderPassColorAttachment {
                        view: &normal_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::BLACK),
                            store: StoreOp::Store,
                        },
                    }),
                    Some(RenderPassColorAttachment {
                        view: &material_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::BLACK),
                            store: StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self
                        .depth_texture
                        .create_view(&TextureViewDescriptor::default()),
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.geometry_render_pipeline);

            // Set bind groups here
            render_pass.set_bind_group(CAMERA_BIND_GROUP, camera.bind_group(), &[]);

            // temp
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            // Run render callback
            geometry_callback(&mut render_pass);
        }

        // Lighting Pass
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Defered 3D Lighting Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Set the lighing bind group here
        }

        self.gpu_controller.submit(encoder);

        Ok(())
    }
}
