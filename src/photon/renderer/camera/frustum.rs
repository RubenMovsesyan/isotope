use cgmath::{InnerSpace, Point3, Vector3, Zero};
use log::debug;

use super::PhotonCamera;

type Point = Vector3<f32>;
type Norm = Vector3<f32>;

#[derive(Debug)]
struct Plane {
    normal: Vector3<f32>,
    distance: f32,
}

impl Plane {
    fn get_signed_distance(&self, point: &Vector3<f32>) -> f32 {
        self.normal.dot(*point) - self.distance
    }
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            normal: Vector3::zero(),
            distance: f32::default(),
        }
    }
}

impl From<(Point, Norm)> for Plane {
    fn from(value: (Point, Norm)) -> Self {
        let normal = value.1.normalize();

        Self {
            normal,
            distance: normal.dot(value.0),
        }
    }
}

#[derive(Debug, Default)]
pub struct Frustum {
    top_face: Plane,
    bottom_face: Plane,

    left_face: Plane,
    right_face: Plane,

    near_face: Plane,
    far_face: Plane,
}

impl Frustum {
    pub(crate) fn new(camera: &PhotonCamera) -> Self {
        let half_v_side = camera.zfar * f32::tan(camera.fovy * 0.5);
        let half_h_side = half_v_side * camera.aspect;

        let front = camera.target.normalize();
        let front_mult_far = camera.zfar * front;
        let right = front.cross(camera.up);
        let position = camera.eye.to_homogeneous().truncate();

        let top_face = Plane::from((
            position,
            right.cross(front_mult_far - camera.up * half_v_side),
        ));

        let bottom_face = Plane::from((
            position,
            (front_mult_far + camera.up * half_v_side).cross(right),
        ));

        let left_face = Plane::from((
            position,
            camera.up.cross(front_mult_far + right * half_h_side),
        ));

        let right_face = Plane::from((
            position,
            (front_mult_far - right * half_h_side).cross(camera.up),
        ));

        let near_face = Plane::from((position + camera.znear * front, front));
        let far_face = Plane::from((position + front_mult_far, -front));

        Self {
            top_face,
            bottom_face,
            left_face,
            right_face,
            near_face,
            far_face,
        }
    }

    pub(crate) fn update(
        &mut self,
        eye: Point3<f32>,
        target: Vector3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) {
        let half_v_side = zfar * f32::tan(fovy * 0.5);
        let half_h_side = half_v_side * aspect;

        let front = target.normalize();
        let front_mult_far = zfar * front;
        let right = front.cross(up);
        let position = eye.to_homogeneous().truncate();

        self.top_face = Plane::from((position, right.cross(front_mult_far - up * half_v_side)));

        self.bottom_face =
            Plane::from((position, (front_mult_far + up * half_v_side).cross(right)));

        self.left_face = Plane::from((position, up.cross(front_mult_far + right * half_h_side)));

        self.right_face = Plane::from((position, (front_mult_far - right * half_h_side).cross(up)));

        self.near_face = Plane::from((position + znear * front, front));
        self.far_face = Plane::from((position + front_mult_far, -front));
    }

    pub(crate) fn contains(&self, radius: f32, center: impl Into<Vector3<f32>>) -> bool {
        let center = center.into();

        let is_on_or_forward_plane =
            |plane: &Plane| -> bool { plane.get_signed_distance(&center) > -radius };

        is_on_or_forward_plane(&self.top_face)
            && is_on_or_forward_plane(&self.bottom_face)
            && is_on_or_forward_plane(&self.left_face)
            && is_on_or_forward_plane(&self.right_face)
            && is_on_or_forward_plane(&self.near_face)
            && is_on_or_forward_plane(&self.far_face)
    }
}
