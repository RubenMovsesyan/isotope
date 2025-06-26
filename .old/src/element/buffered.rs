use wgpu::VertexBufferLayout;

pub trait Buffered {
    fn desc() -> VertexBufferLayout<'static>;
}
