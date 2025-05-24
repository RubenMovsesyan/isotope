use std::{any::Any, fmt::Debug};

use wgpu::RenderPass;

pub mod buffered;
pub mod material;
pub mod mesh;
pub mod model;
pub mod model_vertex;

pub trait Element: Debug + Send + Sync {
    // For the renderer
    #[allow(unused_variables)]
    fn render(&mut self, render_pass: &mut RenderPass) {}

    // For the logic
    fn update(&mut self) {}

    // Required for casting
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
