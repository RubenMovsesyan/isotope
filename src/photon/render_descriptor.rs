use std::{borrow::Cow, sync::Arc};

use log::*;
use wgpu::{
    BindGroup, BindGroupLayout, BindingType, BlendComponent, BlendFactor, BlendOperation,
    BlendState, Buffer, BufferBindingType, ColorTargetState, ColorWrites, CompareFunction,
    DepthBiasState, DepthStencilState, Face, FragmentState, FrontFace, MultisampleState,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, RenderPass, RenderPipeline, RenderPipelineDescriptor, SamplerBindingType,
    ShaderModuleDescriptor, ShaderSource, StencilState, TextureSampleType, TextureViewDimension,
    VertexBufferLayout, VertexState,
};

use crate::{gpu_utils::GpuController, photon::renderer::texture::PHOTON_TEXTURE_DEPTH_FORMAT};

#[allow(dead_code)]
pub const STORAGE_RW: BindingType = BindingType::Buffer {
    ty: BufferBindingType::Storage { read_only: false },
    has_dynamic_offset: false,
    min_binding_size: None,
};

#[allow(dead_code)]
pub const STORAGE_RO: BindingType = BindingType::Buffer {
    ty: BufferBindingType::Storage { read_only: true },
    has_dynamic_offset: false,
    min_binding_size: None,
};

#[allow(dead_code)]
pub const UNIFORM: BindingType = BindingType::Buffer {
    ty: BufferBindingType::Uniform,
    has_dynamic_offset: false,
    min_binding_size: None,
};

#[allow(dead_code)]
pub const TEXTURE: BindingType = BindingType::Texture {
    multisampled: false,
    sample_type: TextureSampleType::Float { filterable: true },
    view_dimension: TextureViewDimension::D2,
};

pub const SAMPLER: BindingType = BindingType::Sampler(SamplerBindingType::Filtering);

pub const RENDER_MINIMUM_BIND_GROUP: u32 = 2;

// Macro to build bind groups and layouts from buffers
#[macro_export]
macro_rules! bind_group_builder {
    ($device:expr, $label:literal, $( ($binding:literal, $visibility:ident, $resource:expr, $type:expr) ),*) => {{
        use wgpu::{
            BindGroupLayoutEntry,
            ShaderStages,
            BindGroupEntry,
            BindGroupLayoutDescriptor,
            BindGroupDescriptor,
        };


        let mut layout_entries = Vec::new();
        let mut bind_group_entries = Vec::new();

        $(
            layout_entries.push(BindGroupLayoutEntry {
                binding: $binding,
                visibility: ShaderStages::$visibility,
                ty: $type,
                count: None,
            });

            bind_group_entries.push(BindGroupEntry {
                binding: $binding,
                resource: $resource,
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

#[macro_export]
macro_rules! bind_group_with_layout {
    ($device:expr, $label:literal, $layout:expr, $( ($binding:literal, $resource:expr) ),*) => {
        {
            use wgpu::{
                BindGroupEntry, BindGroupDescriptor
            };

            let mut bind_group_entries = Vec::new();

            $(
                bind_group_entries.push(BindGroupEntry {
                    binding: $binding,
                    resource: $resource,
                });
            )*

            let bind_group = $device.create_bind_group(&BindGroupDescriptor {
                label: Some($label),
                layout: $layout,
                entries: &bind_group_entries,
            });

            ($layout, bind_group)
        }
    };
}

#[derive(Debug, Default)]
pub struct PhotonRenderDescriptorBuilder<'a> {
    vertex_shader: Option<Cow<'a, str>>,
    fragment_shader: Option<Cow<'a, str>>,
    label: Option<String>,
    vertex_buffers: Vec<VertexBufferLayout<'static>>,
    bind_group_layouts: Option<Vec<BindGroupLayout>>,
    bind_groups: Option<Vec<BindGroup>>,
    vertex_pipeline_compilation_options: Option<PipelineCompilationOptions<'a>>,
    fragment_pipeline_compilation_options: Option<PipelineCompilationOptions<'a>>,
    polygon_mode: Option<PolygonMode>,
    render_descriptor_chains: Vec<Arc<PhotonRenderDescriptor>>,
    primitive_topology: Option<PrimitiveTopology>,
    depth_stencil_state: Option<DepthStencilState>,
}

#[allow(dead_code)]
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
    pub fn with_label<S>(&mut self, label: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.label = Some(label.as_ref().to_string());

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
        self.bind_group_layouts = Some(Vec::from(bind_group_layouts));

        self
    }

    /// Set the primitive topology type
    pub fn with_primitive_topology(&mut self, primitive_topology: PrimitiveTopology) -> &mut Self {
        self.primitive_topology = Some(primitive_topology);

        self
    }

    /// Set the depth stencil State
    pub fn with_depth_stencil_state(
        &mut self,
        depth_stencil_state: DepthStencilState,
    ) -> &mut Self {
        self.depth_stencil_state = Some(depth_stencil_state);

        self
    }

    /// Add Bind Group Layouts
    pub fn add_bind_group_layout(&mut self, bind_group_layout: BindGroupLayout) -> &mut Self {
        if let Some(bind_group_layouts) = self.bind_group_layouts.as_mut() {
            bind_group_layouts.push(bind_group_layout);
        } else {
            self.bind_group_layouts = Some(Vec::from([bind_group_layout]));
        }

        self
    }

    /// Add Bind Group
    pub fn add_bind_group(&mut self, bind_group: BindGroup) -> &mut Self {
        if let Some(bind_groups) = self.bind_groups.as_mut() {
            bind_groups.push(bind_group);
        } else {
            self.bind_groups = Some(Vec::from([bind_group]))
        }

        self
    }

    /// Set the Bind Group Layouts and Bind Groups
    pub fn with_bind_group_with_layouts(
        &mut self,
        bind_group_with_layouts: Vec<(BindGroupLayout, BindGroup)>,
    ) -> &mut Self {
        for (layout, bind_group) in bind_group_with_layouts.into_iter() {
            self.add_bind_group_with_layout((layout, bind_group));
        }

        self
    }

    /// Add Bind Group Layout and Bind Group
    pub fn add_bind_group_with_layout(
        &mut self,
        bind_group_with_layout: (BindGroupLayout, BindGroup),
    ) -> &mut Self {
        self.add_bind_group_layout(bind_group_with_layout.0)
            .add_bind_group(bind_group_with_layout.1)
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

    /// Add chains of bind groups for reuse
    pub fn add_render_chain(&mut self, chain: Arc<PhotonRenderDescriptor>) -> &mut Self {
        self.render_descriptor_chains.push(chain);

        self
    }

    // helper function to copy the label from the builder
    fn get_label_with(&self, connecting_label: &str) -> String {
        let label = match self.label.as_ref() {
            Some(label) => label.clone() + " ",
            None => "".to_string(),
        };

        label + connecting_label
    }

    /// Build the render descriptor without a render pipeline
    pub fn build_module(&mut self, gpu_controller: Arc<GpuController>) -> PhotonRenderDescriptor {
        let bind_groups = if let Some(bind_groups) = self.bind_groups.take() {
            bind_groups
        } else {
            Vec::new()
        };

        let bind_group_layouts = if let Some(bind_group_layouts) = self.bind_group_layouts.take() {
            bind_group_layouts
        } else {
            Vec::new()
        };

        debug!("Building Module: {:#?}", self.label);
        debug!("With Bind Group Layouts: {:#?}", bind_group_layouts);
        debug!("With Bind Groups: {:#?}", bind_groups);

        PhotonRenderDescriptor {
            render_module: {
                if self.render_descriptor_chains.is_empty() {
                    PhotonRenderDescriptorModule::Module
                } else {
                    PhotonRenderDescriptorModule::ChainedModule {
                        chained_render_descriptors: self
                            .render_descriptor_chains
                            .iter()
                            .map(|chain| chain.clone())
                            .collect(),
                    }
                }
            },
            bind_groups,
            bind_group_layouts,
            gpu_controller,
        }
    }

    /// Build the render descriptor from the builder
    pub fn build(&mut self, gpu_controller: Arc<GpuController>) -> PhotonRenderDescriptor {
        // Create the vertex shader module
        let mut vertex_shader = self.vertex_shader.take().and_then(|shader_code| {
            Some(
                gpu_controller
                    .device
                    .create_shader_module(ShaderModuleDescriptor {
                        label: Some(&self.get_label_with("Vertex Shader")),
                        source: ShaderSource::Wgsl(shader_code),
                    }),
            )
        });

        // Create the fragment shader module
        let fragment_shader = self.fragment_shader.take().and_then(|shader_code| {
            Some(
                gpu_controller
                    .device
                    .create_shader_module(ShaderModuleDescriptor {
                        label: Some(&self.get_label_with("Fragment Shader")),
                        source: ShaderSource::Wgsl(shader_code),
                    }),
            )
        });

        // Unwrap the bind groups if there are any
        let bind_groups = self.bind_groups.take().unwrap_or_else(|| Vec::new()); // NOTE: unwrap_or_else is lazy eval where unwrap_or is eager

        let bind_group_layouts = {
            let mut v = Vec::from([
                &gpu_controller.layouts.camera_layout,
                &gpu_controller.layouts.lights_layout,
            ]);
            for chain in self.render_descriptor_chains.iter() {
                chain.add_layouts_to_chain(&mut v);
            }

            if let Some(bind_group_layouts) = self.bind_group_layouts.as_ref() {
                for bind_group_layout in bind_group_layouts.iter() {
                    v.push(bind_group_layout);
                }
            }

            v
        };

        // Create the render pipeline
        // If the vertex shader does not exist then the render descriptor will use the default
        // render pipeline provided by GpuController
        let render_pipeline = if let Some(vertex_shader) = vertex_shader.take() {
            let layout = gpu_controller
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some(&self.get_label_with("Render Pipeline Layout")),
                    bind_group_layouts: bind_group_layouts.as_slice(),
                    push_constant_ranges: &[],
                });

            let targets = [Some(ColorTargetState {
                format: gpu_controller.surface_configuration().format,
                // blend: Some(BlendState::REPLACE),
                blend: Some(BlendState {
                    color: BlendComponent {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent::OVER,
                }),
                write_mask: ColorWrites::ALL,
            })];

            Some(
                gpu_controller
                    .device
                    .create_render_pipeline(&RenderPipelineDescriptor {
                        label: Some(&self.get_label_with("Render Pipeline")),
                        layout: Some(&layout),
                        vertex: VertexState {
                            module: &vertex_shader,
                            entry_point: Some("main"),
                            buffers: &self.vertex_buffers,
                            compilation_options: match self
                                .vertex_pipeline_compilation_options
                                .take()
                            {
                                Some(compilation_options) => compilation_options,
                                None => PipelineCompilationOptions::default(),
                            },
                        },
                        fragment: if let Some(shader) = fragment_shader.as_ref() {
                            Some(FragmentState {
                                module: shader,
                                entry_point: Some("main"),
                                targets: &targets,
                                compilation_options: match self
                                    .fragment_pipeline_compilation_options
                                    .take()
                                {
                                    Some(compilation_options) => compilation_options,
                                    None => PipelineCompilationOptions::default(),
                                },
                            })
                        } else {
                            None
                        },
                        primitive: PrimitiveState {
                            topology: if let Some(primitive_topology) = self.primitive_topology {
                                primitive_topology
                            } else {
                                PrimitiveTopology::TriangleList // Default
                            },
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
                        depth_stencil: if let Some(depth_stencil_state) =
                            self.depth_stencil_state.take()
                        {
                            Some(depth_stencil_state)
                        } else {
                            Some(DepthStencilState {
                                format: PHOTON_TEXTURE_DEPTH_FORMAT,
                                depth_write_enabled: true,
                                depth_compare: CompareFunction::Less,
                                stencil: StencilState::default(),
                                bias: DepthBiasState::default(),
                            })
                        },
                        multisample: MultisampleState {
                            count: 1,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        multiview: None,
                        cache: None,
                    }),
            )
        } else {
            None
        };

        PhotonRenderDescriptor {
            gpu_controller,
            bind_groups,
            bind_group_layouts: if let Some(bind_group_layouts) = self.bind_group_layouts.take() {
                bind_group_layouts
            } else {
                Vec::new()
            },
            render_module: {
                if self.render_descriptor_chains.is_empty() {
                    PhotonRenderDescriptorModule::Full { render_pipeline }
                } else {
                    PhotonRenderDescriptorModule::ChainedFull {
                        chained_render_descriptors: self
                            .render_descriptor_chains
                            .iter()
                            .map(|chain| chain.clone())
                            .collect(),
                        render_pipeline,
                    }
                }
            },
        }
    }
}

#[derive(Debug)]
enum PhotonRenderDescriptorModule {
    Full {
        render_pipeline: Option<RenderPipeline>,
    },
    Module,
    ChainedFull {
        chained_render_descriptors: Vec<Arc<PhotonRenderDescriptor>>,
        render_pipeline: Option<RenderPipeline>,
    },
    ChainedModule {
        chained_render_descriptors: Vec<Arc<PhotonRenderDescriptor>>,
    },
}

#[derive(Debug)]
pub struct PhotonRenderDescriptor {
    render_module: PhotonRenderDescriptorModule,
    bind_group_layouts: Vec<BindGroupLayout>,
    bind_groups: Vec<BindGroup>,

    pub(crate) gpu_controller: Arc<GpuController>,
}

impl PhotonRenderDescriptor {
    pub(crate) fn write_buffer(&self, buffer: &Buffer, data: &[u8]) {
        self.gpu_controller.queue.write_buffer(buffer, 0, data);
    }

    pub(crate) unsafe fn write_buffer_offset(&self, buffer: &Buffer, offset: u64, data: &[u8]) {
        self.gpu_controller.queue.write_buffer(buffer, offset, data);
    }

    pub(crate) fn add_layouts_to_chain<'a>(&'a self, layouts_vec: &mut Vec<&'a BindGroupLayout>) {
        use PhotonRenderDescriptorModule::*;

        match &self.render_module {
            ChainedModule {
                chained_render_descriptors,
            } => {
                for module in chained_render_descriptors.iter() {
                    module.add_layouts_to_chain(layouts_vec);
                }
            }
            _ => {}
        }

        for bind_group_layout in self.bind_group_layouts.iter() {
            layouts_vec.push(bind_group_layout);
        }
    }

    pub fn setup_render(&self, render_pass: &mut RenderPass) -> u32 {
        use PhotonRenderDescriptorModule::*;

        match &self.render_module {
            Full { render_pipeline } => {
                match render_pipeline {
                    Some(pipeline) => render_pass.set_pipeline(pipeline),
                    None => render_pass.set_pipeline(&self.gpu_controller.default_render_pipeline),
                }

                for (index, bind_group) in self.bind_groups.iter().enumerate() {
                    render_pass.set_bind_group(
                        index as u32 + RENDER_MINIMUM_BIND_GROUP,
                        bind_group,
                        &[],
                    );
                }

                0
            }
            Module => {
                for (index, bind_group) in self.bind_groups.iter().enumerate() {
                    render_pass.set_bind_group(
                        index as u32 + RENDER_MINIMUM_BIND_GROUP,
                        bind_group,
                        &[],
                    );
                }

                self.bind_groups.len() as u32
            }
            ChainedFull {
                chained_render_descriptors,
                render_pipeline,
            } => {
                let mut start_index: u32 = 0;

                match render_pipeline {
                    Some(pipeline) => render_pass.set_pipeline(pipeline),
                    None => render_pass.set_pipeline(&self.gpu_controller.default_render_pipeline),
                }

                for chain in chained_render_descriptors.iter() {
                    start_index += chain.setup_render(render_pass);
                }

                for (index, bind_group) in self.bind_groups.iter().enumerate() {
                    render_pass.set_bind_group(
                        index as u32 + start_index + RENDER_MINIMUM_BIND_GROUP,
                        bind_group,
                        &[],
                    );
                }

                start_index + self.bind_groups.len() as u32
            }
            ChainedModule {
                chained_render_descriptors,
            } => {
                let mut start_index: u32 = 0;

                for chain in chained_render_descriptors.iter() {
                    start_index += chain.setup_render(render_pass);
                }

                for (index, bind_group) in self.bind_groups.iter().enumerate() {
                    render_pass.set_bind_group(
                        index as u32 + start_index + RENDER_MINIMUM_BIND_GROUP,
                        bind_group,
                        &[],
                    );
                }

                start_index + self.bind_groups.len() as u32
            }
        }
    }
}
