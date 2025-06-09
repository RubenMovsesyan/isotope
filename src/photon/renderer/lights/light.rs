use cgmath::{Point3, Vector3};

pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Into<[f32; 3]> for Color {
    fn into(self) -> [f32; 3] {
        [self.r, self.b, self.g]
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct Light {
    pub position: [f32; 3],
    _padding: f32,
    pub normal: [f32; 3],
    _padding_2: f32,
    pub color: [f32; 3],
    pub intensity: f32,
}

impl Light {
    pub fn new(
        position: Point3<f32>,
        direction: Vector3<f32>,
        color: Color,
        intensity: f32,
    ) -> Self {
        Self {
            position: position.into(),
            _padding: 0.0,
            normal: direction.into(),
            _padding_2: 0.0,
            color: color.into(),
            intensity,
        }
    }

    pub fn x<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut f32),
    {
        callback(&mut self.position[0]);
    }

    pub fn y<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut f32),
    {
        callback(&mut self.position[1]);
    }

    pub fn z<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut f32),
    {
        callback(&mut self.position[2]);
    }

    pub fn pos<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut f32, &mut f32, &mut f32),
    {
        let (x, yz) = self.position.split_at_mut(1);
        let (y, z) = yz.split_at_mut(1);
        callback(&mut x[0], &mut y[0], &mut z[0]);
    }
}
