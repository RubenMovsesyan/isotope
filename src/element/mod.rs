use std::fmt::Debug;

use wgpu::RenderPass;

pub mod buffered;
pub mod material;
pub mod mesh;
pub mod model;
pub mod model_vertex;

pub trait Element: Debug + Send + Sync {
    fn render(&self, render_pass: &mut RenderPass);
}
