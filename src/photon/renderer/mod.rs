use std::sync::Arc;

use photon_layouts::PhotonLayoutsManager;
use wgpu::{RenderPipeline, SurfaceConfiguration, include_wgsl};

use crate::{GpuController, construct_render_pipeline};

pub mod photon_layouts;
mod render_macros;
pub mod texture;

#[derive(Debug)]
pub struct PhotonRenderer {
    gpu_controller: Arc<GpuController>,
    layouts: PhotonLayoutsManager,
    render_pipeline: RenderPipeline,
}

impl PhotonRenderer {
    pub fn new(
        gpu_controller: Arc<GpuController>,
        surface_configuration: &SurfaceConfiguration,
    ) -> Self {
        let vertex_shader = gpu_controller
            .device
            .create_shader_module(include_wgsl!("shaders/vert.wgsl"));
        let fragment_shader = gpu_controller
            .device
            .create_shader_module(include_wgsl!("shaders/frag.wgsl"));

        let layouts = PhotonLayoutsManager::new(&gpu_controller);

        let render_pipeline = construct_render_pipeline!(
            &gpu_controller.device,
            surface_configuration,
            vertex_shader,
            fragment_shader,
            String::from("Photon"),
            &layouts.texture_layout
        );

        Self {
            gpu_controller,
            layouts,
            render_pipeline,
        }
    }
}
