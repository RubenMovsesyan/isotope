use cgmath::Vector3;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Gravity {
    None,
    World(Vector3<f32>),
    Point(Vector3<f32>, f32),
    WorldPoint(Vector3<f32>, Vector3<f32>, f32),
}
