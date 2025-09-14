use cgmath::{Quaternion, Vector3};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Transform {
    pub(crate) position: [f32; 3],
    pub(crate) rotation: [f32; 4],
}

impl Transform {
    pub fn new<V, Q>(position: V, rotation: Q) -> Self
    where
        V: Into<[f32; 3]>,
        Q: Into<[f32; 4]>,
    {
        Self {
            position: position.into(),
            rotation: rotation.into(),
        }
    }

    pub fn position<F, R>(&mut self, callback: F) -> R
    where
        F: FnOnce(&mut Vector3<f32>) -> R,
    {
        // Safety: Vector3<f32> and [f32; 3] have an identical memory layout
        let position_ref = unsafe { &mut *(self.position.as_mut_ptr() as *mut Vector3<f32>) };
        callback(position_ref)
    }

    pub fn rotation<F, R>(&mut self, callback: F) -> R
    where
        F: FnOnce(&mut Quaternion<f32>) -> R,
    {
        // Safety: Quaternion<f32> and [f32; 4] have an identical memory layout
        let rotation_ref = unsafe { &mut *(self.rotation.as_mut_ptr() as *mut Quaternion<f32>) };
        callback(rotation_ref)
    }

    pub fn position_and_rotation<F, R>(&mut self, callback: F) -> R
    where
        F: FnOnce(&mut Vector3<f32>, &mut Quaternion<f32>) -> R,
    {
        // Safety: Vector3<f32> and [f32; 3] have an identical memory layout
        let position_ref = unsafe { &mut *(self.position.as_mut_ptr() as *mut Vector3<f32>) };
        // Safety: Quaternion<f32> and [f32; 4] have an identical memory layout
        let rotation_ref = unsafe { &mut *(self.rotation.as_mut_ptr() as *mut Quaternion<f32>) };
        callback(position_ref, rotation_ref)
    }

    pub fn get_position<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&Vector3<f32>) -> R,
    {
        // Safety: Vector3<f32> and [f32; 3] have an identical memory layout
        let position_ref = unsafe { &*(self.position.as_ptr() as *const Vector3<f32>) };
        callback(position_ref)
    }

    pub fn get_rotation<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&Quaternion<f32>) -> R,
    {
        // Safety: Quaternion<f32> and [f32; 4] have an identical memory layout
        let rotation_ref = unsafe { &*(self.rotation.as_ptr() as *const Quaternion<f32>) };
        callback(rotation_ref)
    }

    pub fn get_position_and_rotation<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&Vector3<f32>, &Quaternion<f32>) -> R,
    {
        // Safety: Vector3<f32> and [f32; 3] have an identical memory layout
        let position_ref = unsafe { &*(self.position.as_ptr() as *const Vector3<f32>) };
        // Safety: Quaternion<f32> and [f32; 4] have an identical memory layout
        let rotation_ref = unsafe { &*(self.rotation.as_ptr() as *const Quaternion<f32>) };
        callback(position_ref, rotation_ref)
    }
}
