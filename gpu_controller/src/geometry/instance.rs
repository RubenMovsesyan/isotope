use cgmath::{Quaternion, Vector3};
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

    pub fn pos<F, R>(&mut self, callback: F) -> R
    where
        F: FnOnce(&mut Vector3<f32>) -> R,
    {
        let pos_ref = unsafe { &mut *(self.position.as_mut_ptr() as *mut Vector3<f32>) };
        callback(pos_ref)
    }

    pub fn orient<F, R>(&mut self, callback: F) -> R
    where
        F: FnOnce(&mut Quaternion<f32>) -> R,
    {
        let rot_ref = unsafe { &mut *(self.orientation.as_mut_ptr() as *mut Quaternion<f32>) };
        callback(rot_ref)
    }

    pub fn scale<F, R>(&mut self, callback: F) -> R
    where
        F: FnOnce(&mut Scale) -> R,
    {
        // let scale_ref = unsafe { &mut *(self.scale.as_mut_ptr() as *mut Matrix4<f32>) };
        callback(&mut self.scale)
    }

    pub fn transform<F, R>(&mut self, callback: F) -> R
    where
        F: FnOnce(&mut Vector3<f32>, &mut Quaternion<f32>, &mut Scale) -> R,
    {
        let pos_ref = unsafe { &mut *(self.position.as_mut_ptr() as *mut Vector3<f32>) };
        let rot_ref = unsafe { &mut *(self.orientation.as_mut_ptr() as *mut Quaternion<f32>) };
        callback(pos_ref, rot_ref, &mut self.scale)
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
