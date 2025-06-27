use std::sync::Arc;

use wgpu::RenderPipeline;

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
    ChangedModule {
        chained_render_descriptors: Vec<Arc<PhotonRenderDescriptor>>,
    },
}

#[derive(Debug)]
pub struct PhotonRenderDescriptor {}
