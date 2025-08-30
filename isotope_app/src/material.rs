#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Material {
    _padding: [u32; 2],
    pub ambient_color: [f32; 3],
    pub diffuse_color: [f32; 3],
    pub specular_color: [f32; 3],
    pub specular_focus: f32,
    pub optical_density: f32,
    pub dissolve: f32,
    pub illum: u32,
}

// ERROR color as default
impl Default for Material {
    fn default() -> Self {
        Self {
            _padding: [0; 2],
            ambient_color: [1.0, 0.0, 1.0],
            diffuse_color: [1.0, 0.0, 1.0],
            specular_color: [1.0, 0.0, 1.0],
            specular_focus: 100.0,
            optical_density: 0.0,
            dissolve: 0.0,
            illum: 0,
        }
    }
}
