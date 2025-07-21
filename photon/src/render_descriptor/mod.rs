use std::sync::Arc;

use gpu_controller::GpuController;
use vertex::Vertex;
use wgpu::{BindGroup, Buffer, VertexBufferLayout};

mod mesh;
pub mod vertex;

pub trait Buffered {
    fn desc() -> VertexBufferLayout<'static>;
}

pub struct RenderDescriptor {}

pub struct Material;
