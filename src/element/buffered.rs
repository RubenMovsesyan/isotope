use wgpu::VertexBufferLayout;

pub(crate) trait Buffered {
    fn desc() -> VertexBufferLayout<'static>;
}
