use cgmath::Vector3;

pub const GRAVITATIONAL_CONSTANT: f64 = 6.674e-11;

pub trait Gravitational {
    fn apply_gravity(&mut self, gravity: &Gravity, timestep: f64);
}

pub enum Gravity {
    None,
    World(Vector3<f64>),
    Point(Vector3<f64>, f64),
    WorldPoint(Vector3<f64>, Vector3<f64>, f64),
}
