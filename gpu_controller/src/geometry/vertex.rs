use std::mem;

use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use crate::Buffered;

use super::{NormalVec, Position, UvCoord};

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Position,
    pub uv_coord: UvCoord,
    pub normal_vec: NormalVec,
}

impl Vertex {
    pub fn new<PN, UV, NV>(position: PN, uv_coord: UV, normal_vec: NV) -> Self
    where
        PN: Into<Position>,
        UV: Into<UvCoord>,
        NV: Into<NormalVec>,
    {
        Self {
            position: position.into(),
            uv_coord: uv_coord.into(),
            normal_vec: normal_vec.into(),
        }
    }
}

impl Buffered for Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                // Position
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                // UV Coordinates
                VertexAttribute {
                    offset: mem::size_of::<Position>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                // Normal Vector
                VertexAttribute {
                    offset: (mem::size_of::<Position>() + mem::size_of::<UvCoord>())
                        as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}
