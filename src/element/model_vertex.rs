use super::buffered::Buffered;
use cgmath::{InnerSpace, Vector3};
use std::mem;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

pub(crate) type VertexPosition = [f32; 3];
pub(crate) type VertexUvCoord = [f32; 2];
pub(crate) type VertexNormalVec = [f32; 3];

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ModelVertex {
    pub(crate) position: VertexPosition,
    pub(crate) uv_coords: VertexUvCoord,
    pub(crate) normal_vec: VertexNormalVec,
}

#[allow(dead_code)]
impl ModelVertex {
    pub(crate) fn new<PN, UV, NV>(position: PN, uv_coords: UV, normal_vec: NV) -> Self
    where
        PN: Into<VertexPosition>,
        UV: Into<VertexUvCoord>,
        NV: Into<VertexNormalVec>,
    {
        Self {
            position: position.into(),
            uv_coords: uv_coords.into(),
            normal_vec: normal_vec.into(),
        }
    }

    #[allow(dead_code)]
    pub(crate) const fn new_const(
        position: VertexPosition,
        uv_coords: VertexUvCoord,
        normal_vec: VertexNormalVec,
    ) -> Self {
        Self {
            position,
            uv_coords,
            normal_vec,
        }
    }
}

impl From<(f32, f32, f32)> for ModelVertex {
    fn from(value: (f32, f32, f32)) -> Self {
        let normal: Vector3<f32> = Vector3 {
            x: value.0,
            y: value.1,
            z: value.2,
        }
        .normalize();

        Self {
            position: [value.0, value.1, value.2],
            normal_vec: normal.into(),
            uv_coords: [0.0, 0.0],
        }
    }
}

impl Buffered for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                // Position
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                // UV coordinates
                VertexAttribute {
                    offset: mem::size_of::<VertexPosition>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                // Normal Vector
                VertexAttribute {
                    offset: (mem::size_of::<VertexPosition>() + mem::size_of::<VertexUvCoord>())
                        as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelNormalVertex {
    pub(crate) position: [f32; 3],
}

impl Buffered for ModelNormalVertex {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: mem::size_of::<ModelNormalVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                // Position
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}
