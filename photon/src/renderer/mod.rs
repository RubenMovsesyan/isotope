use std::sync::Arc;

use anyhow::Result;
use defered_renderer::DeferedRenderer3D;
use gpu_controller::{GpuController, RenderPass, Texture};

use crate::{Light, camera::Camera};

pub mod defered_renderer;

const CAMERA_BIND_GROUP: u32 = 0;
const LIGHTS_BIND_GROUP: u32 = 1;
pub const MATERIALS_BIND_GROUP: u32 = 1;

pub enum Renderer {
    Defered3D(DeferedRenderer3D),
}

impl Renderer {
    pub fn new_defered_3d(gpu_controller: Arc<GpuController>) -> Result<Self> {
        Ok(Self::Defered3D(DeferedRenderer3D::new(gpu_controller)?))
    }

    pub fn update_lights(&mut self, lights: &[Light]) {
        match self {
            Self::Defered3D(renderer) => {
                renderer.lights_manager.update_lights(lights);
            }
        }
    }

    pub fn render<G>(&self, camera: &Camera, output: &Texture, geometry_callback: G)
    where
        G: FnOnce(&mut RenderPass),
    {
        match self {
            Self::Defered3D(renderer) => _ = renderer.render(camera, output, geometry_callback),
        }
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        match self {
            Self::Defered3D(renderer) => renderer.resize(new_size),
        }
    }
}
