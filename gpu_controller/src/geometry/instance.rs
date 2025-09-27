use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use crate::Buffered;

use super::{Orientation, Position, Scale};

#[repr(C)]
#[derive(Debug, bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub struct Instance {
    position: Position,
    _padding: f32,
    orientation: Orientation,
    scale: Scale,
}

impl Instance {
    pub fn new<PN, OR, SC>(position: PN, orientation: OR, scale: SC) -> Self
    where
        PN: Into<Position>,
        OR: Into<Orientation>,
        SC: Into<Scale>,
    {
        Self {
            position: position.into(),
            _padding: 0.0,
            orientation: orientation.into(),
            scale: scale.into(),
        }
    }
}

impl Buffered for Instance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                // Position
                VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: VertexFormat::Float32x3,
                },
                // Rotation
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 4,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}
