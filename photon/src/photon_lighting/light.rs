type Position = [f32; 3];
type Normal = [f32; 3];
type Color = [f32; 3];

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct Light {
    pub position: Position,
    _padding: f32,
    pub normal: Normal,
    _padding_2: f32,
    pub color: Color,
    pub intensity: f32,
}

impl Light {
    pub fn new<P, N, C>(position: P, direction: N, color: C, intensity: f32) -> Self
    where
        P: Into<Position>,
        N: Into<Normal>,
        C: Into<Color>,
    {
        Self {
            position: position.into(),
            _padding: 0.0,
            normal: direction.into(),
            _padding_2: 0.0,
            color: color.into(),
            intensity,
        }
    }

    pub fn pos<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut Position),
    {
        callback(&mut self.position);
    }

    pub fn direction<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut Normal),
    {
        callback(&mut self.normal);
    }

    pub fn color<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut Color),
    {
        callback(&mut self.color);
    }

    pub fn intensity<F>(&mut self, callback: F)
    where
        F: FnOnce(&mut f32),
    {
        callback(&mut self.intensity);
    }
}
