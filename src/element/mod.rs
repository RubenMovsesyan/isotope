use wgpu::RenderPass;

pub mod buffered;
pub mod material;
pub mod mesh;
pub mod model;
pub mod model_vertex;

pub(crate) trait Element {
    // fn vertex_buffer(&self) -> &Buffer;
    // fn index_buffer(&self) -> &Buffer;
    // fn instance_buffer(&self) -> Option<&Buffer>;
    fn render(&self, render_pass: &RenderPass);
}
