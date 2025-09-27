use cgmath::{Quaternion, Vector3};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Transform3D {
    pub(crate) position: [f32; 3],
    _padding: f32,
    pub(crate) rotation: [f32; 4],
}

impl Default for Transform3D {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            _padding: 0.0,
            rotation: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

impl Transform3D {
    /// Creates a new Transform3D with the given position and rotation.
    ///
    /// # Arguments
    /// * `position` - The 3D position, convertible to [f32; 3]
    /// * `rotation` - The rotation quaternion, convertible to [f32; 4]
    pub fn new<V, Q>(position: V, rotation: Q) -> Self
    where
        V: Into<[f32; 3]>,
        Q: Into<[f32; 4]>,
    {
        Self {
            position: position.into(),
            _padding: 0.0,
            rotation: rotation.into(),
        }
    }

    /// Provides mutable access to the position as a Vector3<f32> through a callback.
    ///
    /// # Arguments
    /// * `callback` - Function that receives a mutable reference to the position vector
    ///
    /// # Returns
    /// The return value of the callback function
    pub fn position<F, R>(&mut self, callback: F) -> R
    where
        F: FnOnce(&mut Vector3<f32>) -> R,
    {
        // Safety: Vector3<f32> and [f32; 3] have an identical memory layout
        let position_ref = unsafe { &mut *(self.position.as_mut_ptr() as *mut Vector3<f32>) };
        callback(position_ref)
    }

    /// Provides mutable access to the rotation as a Quaternion<f32> through a callback.
    ///
    /// # Arguments
    /// * `callback` - Function that receives a mutable reference to the rotation quaternion
    ///
    /// # Returns
    /// The return value of the callback function
    pub fn rotation<F, R>(&mut self, callback: F) -> R
    where
        F: FnOnce(&mut Quaternion<f32>) -> R,
    {
        // Safety: Quaternion<f32> and [f32; 4] have an identical memory layout
        let rotation_ref = unsafe { &mut *(self.rotation.as_mut_ptr() as *mut Quaternion<f32>) };
        callback(rotation_ref)
    }

    /// Provides mutable access to both position and rotation through a callback.
    ///
    /// # Arguments
    /// * `callback` - Function that receives mutable references to both position and rotation
    ///
    /// # Returns
    /// The return value of the callback function
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

    /// Provides immutable access to the position as a Vector3<f32> through a callback.
    ///
    /// # Arguments
    /// * `callback` - Function that receives an immutable reference to the position vector
    ///
    /// # Returns
    /// The return value of the callback function
    pub fn get_position<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&Vector3<f32>) -> R,
    {
        // Safety: Vector3<f32> and [f32; 3] have an identical memory layout
        let position_ref = unsafe { &*(self.position.as_ptr() as *const Vector3<f32>) };
        callback(position_ref)
    }

    /// Provides immutable access to the rotation as a Quaternion<f32> through a callback.
    ///
    /// # Arguments
    /// * `callback` - Function that receives an immutable reference to the rotation quaternion
    ///
    /// # Returns
    /// The return value of the callback function
    pub fn get_rotation<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&Quaternion<f32>) -> R,
    {
        // Safety: Quaternion<f32> and [f32; 4] have an identical memory layout
        let rotation_ref = unsafe { &*(self.rotation.as_ptr() as *const Quaternion<f32>) };
        callback(rotation_ref)
    }

    /// Provides immutable access to both position and rotation through a callback.
    ///
    /// # Arguments
    /// * `callback` - Function that receives immutable references to both position and rotation
    ///
    /// # Returns
    /// The return value of the callback function
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
