#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ModelVertex {
    position: [f32; 3],
    uv_coords: [f32; 2],
    normal_vec: [f32; 3],
}

impl ModelVertex {
    pub(crate) fn new<PN, UV, NV>(position: PN, uv_coords: UV, normal_vec: NV) -> Self
    where
        PN: Into<[f32; 3]>,
        UV: Into<[f32; 2]>,
        NV: Into<[f32; 3]>,
    {
        Self {
            position: position.into(),
            uv_coords: uv_coords.into(),
            normal_vec: normal_vec.into(),
        }
    }
}
