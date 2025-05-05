use wgpu::VertexBufferLayout;

pub(crate) trait Buffered {
    fn desc(&self) -> VertexBufferLayout<'static>;
}
