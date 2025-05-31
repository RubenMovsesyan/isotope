use std::{borrow::Cow, sync::Arc};

use anyhow::{Result, anyhow};
use wgpu::{
    BindGroup, BindGroupLayout, BlendState, ColorTargetState, ColorWrites, CompareFunction,
    DepthBiasState, DepthStencilState, Face, FragmentState, FrontFace, MultisampleState,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
    ShaderSource, StencilState, SurfaceConfiguration, VertexBufferLayout, VertexState,
};

use crate::{gpu_utils::GpuController, photon::renderer::texture::PHOTON_TEXTURE_DEPTH_FORMAT};

// Macro to build bind groups and layouts from buffers
macro_rules! bind_group_builder {
    ($device:expr, $label:literal, $( ($binding:literal, $visibility:expr, $buffer:expr, $type:expr) ),*) => {{
        let mut layout_entries = Vec::new();
        let mut bind_group_entries = Vec::new();

        $(
            layout_entries.push(BindGroupLayoutEntry {
                binding: $binding,
                visibility: ShaderStages::$visibility,
                ty: BindingType::Buffer {
                    ty: $type,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });

            bind_group_entries.push(BindGroupEntry {
                binding: $binding,
                resource: $buffer.as_entire_binding(),
            });
        )*

        let bind_group_layout = $device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(&format!("{} Layout", $label)),
            entries: &layout_entries,
        });

        let bind_group = $device.create_bind_group(&BindGroupDescriptor {
            label: Some($label),
            layout: &bind_group_layout,
            entries: &bind_group_entries,
        });

        (bind_group_layout, bind_group)
    }};
}

#[derive(Debug, Default)]
pub struct PhotonRenderDescriptorBuilder<'a> {
    vertex_shader: Option<Cow<'a, str>>,
    fragment_shader: Option<Cow<'a, str>>,
    label: Option<String>,
    vertex_buffers: Vec<VertexBufferLayout<'static>>,
    bind_group_layouts: Vec<BindGroupLayout>,
    bind_groups: Vec<BindGroup>,
    vertex_pipeline_compilation_options: Option<PipelineCompilationOptions<'a>>,
    fragment_pipeline_compilation_options: Option<PipelineCompilationOptions<'a>>,
    polygon_mode: Option<PolygonMode>,
}

impl<'a> PhotonRenderDescriptorBuilder<'a> {
    /// Add a vertex shader to the render descriptor
    pub fn with_vertex_shader(&mut self, vertex_shader_code: &'a str) -> &mut Self {
        self.vertex_shader = Some(Cow::Borrowed(vertex_shader_code));

        self
    }

    /// Add a fragment shader to the render descriptor
    pub fn with_fragment_shader(&mut self, fragment_shader_code: &'a str) -> &mut Self {
        self.fragment_shader = Some(Cow::Borrowed(fragment_shader_code));

        self
    }

    /// Give the render pipeline a name
    pub fn with_label(&mut self, label: String) -> &mut Self {
        self.label = Some(label);

        self
    }

    /// Set the vertex buffer layouts
    pub fn with_vertex_buffer_layouts(
        &mut self,
        vertex_buffer_layouts: &[VertexBufferLayout<'static>],
    ) -> &mut Self {
        self.vertex_buffers = Vec::from(vertex_buffer_layouts);

        self
    }

    /// Add Bind Group Layouts
    pub fn with_bind_group_layouts(&mut self, bind_group_layouts: &[BindGroupLayout]) -> &mut Self {
        self.bind_group_layouts = Vec::from(bind_group_layouts);

        self
    }

    /// Add Bind Group Layouts
    pub fn add_bind_group_layout(&mut self, bind_group_layout: BindGroupLayout) -> &mut Self {
        self.bind_group_layouts.push(bind_group_layout);

        self
    }

    /// Add Bind Group
    pub fn add_bind_group(&mut self, bind_group: BindGroup) -> &mut Self {
        self.bind_groups.push(bind_group);

        self
    }

    /// Add Bind Group Layout and Bind Group
    pub fn add_bind_group_with_layout(
        &mut self,
        bind_group_with_layout: (BindGroupLayout, BindGroup),
    ) -> &mut Self {
        self.bind_group_layouts.push(bind_group_with_layout.0);
        self.bind_groups.push(bind_group_with_layout.1);

        self
    }

    /// Set the vertex pipeline compilation options
    pub fn with_vertex_pipeline_compilation_options(
        &mut self,
        vertex_pipeline_compilation_options: PipelineCompilationOptions<'a>,
    ) -> &mut Self {
        self.vertex_pipeline_compilation_options = Some(vertex_pipeline_compilation_options);

        self
    }

    /// Set the fragment pipeline compilation options
    pub fn with_fragment_pipeline_compilation_options(
        &mut self,
        fragment_pipeline_compilation_options: PipelineCompilationOptions<'a>,
    ) -> &mut Self {
        self.fragment_pipeline_compilation_options = Some(fragment_pipeline_compilation_options);

        self
    }

    /// Set the polygon mode
    pub fn with_polygon_mode(&mut self, polygon_mode: PolygonMode) -> &mut Self {
        self.polygon_mode = Some(polygon_mode);

        self
    }

    /// Build the render descriptor from the builder
    pub fn build(
        &mut self,
        gpu_controller: Arc<GpuController>,
        surface_configuration: &SurfaceConfiguration,
    ) -> Result<PhotonRenderDescriptor> {
        // Helper function to copy the label from the builder
        let get_label_with = |connecting_label: &str| -> String {
            let label = match self.label.as_ref() {
                Some(label) => label.clone() + " ",
                None => "".to_string(),
            };

            label + connecting_label
        };

        // Create the vertex shader module
        let vertex_shader = {
            let shader_code = self
                .vertex_shader
                .take()
                .ok_or(anyhow!("Vertex Shader Not Initialized"))?;

            gpu_controller
                .device
                .create_shader_module(ShaderModuleDescriptor {
                    label: Some(&get_label_with("Vertex Shader")),
                    source: ShaderSource::Wgsl(shader_code),
                })
        };

        // Create the fragment shader module
        let fragment_shader = {
            let shader_code = self
                .fragment_shader
                .take()
                .ok_or(anyhow!("Fragment Shader Not Initialized"))?;

            gpu_controller
                .device
                .create_shader_module(ShaderModuleDescriptor {
                    label: Some(&get_label_with("Fragment Shader")),
                    source: ShaderSource::Wgsl(shader_code),
                })
        };

        // Create the render pipeline
        let render_pipeline = {
            let layouts: Vec<&BindGroupLayout> = self.bind_group_layouts.iter().collect::<Vec<_>>();

            let layout = gpu_controller
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some(&get_label_with("Render Pipeline Layout")),
                    bind_group_layouts: layouts.as_slice(),
                    push_constant_ranges: &[],
                });

            gpu_controller
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some(&get_label_with("Render Pipeline")),
                    layout: Some(&layout),
                    vertex: VertexState {
                        module: &vertex_shader,
                        entry_point: Some("main"),
                        buffers: &self.vertex_buffers,
                        compilation_options: match self.vertex_pipeline_compilation_options.take() {
                            Some(compilation_options) => compilation_options,
                            None => PipelineCompilationOptions::default(),
                        },
                    },
                    fragment: Some(FragmentState {
                        module: &fragment_shader,
                        entry_point: Some("main"),
                        targets: &[Some(ColorTargetState {
                            format: surface_configuration.format,
                            blend: Some(BlendState::REPLACE),
                            write_mask: ColorWrites::ALL,
                        })],
                        compilation_options: match self.fragment_pipeline_compilation_options.take()
                        {
                            Some(compilation_options) => compilation_options,
                            None => PipelineCompilationOptions::default(),
                        },
                    }),
                    primitive: PrimitiveState {
                        topology: PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: FrontFace::Ccw,
                        cull_mode: Some(Face::Back),
                        polygon_mode: match self.polygon_mode.take() {
                            Some(mode) => mode,
                            None => PolygonMode::Fill,
                        },
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: Some(DepthStencilState {
                        format: PHOTON_TEXTURE_DEPTH_FORMAT,
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
                    multiview: None,
                    cache: None,
                })
        };

        Ok(PhotonRenderDescriptor {
            render_pipeline,
            gpu_controller,
        })
    }
}

#[derive(Debug)]
pub struct PhotonRenderDescriptor {
    render_pipeline: RenderPipeline,

    gpu_controller: Arc<GpuController>,
}
