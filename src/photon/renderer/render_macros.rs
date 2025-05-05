#[macro_export]
macro_rules! construct_render_pipeline {
    ($device:expr, $config:expr, $vertex_shader:expr, $fragment_shader:expr, $name:expr, $( $layout:expr ),* ) => {
        {
            use wgpu::{
                PipelineLayoutDescriptor, RenderPipelineDescriptor, VertexState, PipelineCompilationOptions, FragmentState, ColorTargetState,
                BlendState, ColorWrites, PrimitiveState, PrimitiveTopology, FrontFace, Face, PolygonMode, DepthStencilState,
                CompareFunction, StencilState, DepthBiasState, MultisampleState,
            };

            use crate::photon::renderer::texture::PHOTON_TEXTURE_DEPTH_FORMAT;

            let mut layouts = Vec::new();

            $(
                layouts.push($layout);
            )*


            let layout = $device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some(&($name.clone() + "Render Pipeline Layout")),
                bind_group_layouts: layouts.as_slice(),
                push_constant_ranges: &[],
            });

            let render_pipeline = $device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(&($name.clone() + "Render Pipeline")),
                layout: Some(&layout),
                vertex: VertexState {
                    module: &$vertex_shader,
                    entry_point: Some("main"),
                    buffers: &[],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment: Some(FragmentState {
                    module: &$fragment_shader,
                    entry_point: Some("main"),
                    targets: &[Some(ColorTargetState {
                        format: $config.format,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: Some(Face::Back),
                    // Change this to make a wireframe
                    polygon_mode: PolygonMode::Fill,
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
            });

            render_pipeline
        }
    };
}
