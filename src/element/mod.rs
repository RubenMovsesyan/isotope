use wgpu::Buffer;

pub mod model_vertex;
pub mod vertex;

pub(crate) trait Element {
    fn vertex_buffer(&self) -> &Buffer;
    fn index_buffer(&self) -> &Buffer;
    fn instance_buffer(&self) -> Option<&Buffer>;
}
