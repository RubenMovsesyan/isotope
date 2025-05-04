use wgpu::VertexBufferLayout;

pub(crate) trait Vertex {
    fn desc(&self) -> VertexBufferLayout<'static>;
}
