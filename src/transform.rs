use cgmath::{One, Quaternion, Vector3, Zero};

#[derive(Debug)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub orientation: Quaternion<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::zero(),
            orientation: Quaternion::one(),
        }
    }
}
