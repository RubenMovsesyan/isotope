use std::f32::consts::PI;

use cgmath::{Deg, InnerSpace, Point3, Quaternion, Rad, Rotation, Rotation3, Vector3, Zero};
use log::debug;

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
        let position = eye.to_homogeneous().truncate();
        debug!("Position: {:#?}", position);
        let front = target.normalize();
        debug!("Front: {:#?}", front);
        let up_norm = up.normalize();
        debug!("Up: {:#?}", up);
        debug!("afznzf: {} {} {} {}", aspect, fovy, znear, zfar);
        let right = front.cross(up_norm).normalize();

        let half_fov_rad = fovy.to_radians() * 0.5;
        let half_fov_h_rad = (half_fov_rad.tan() * aspect).atan();

        // let vert_rotation = Quaternion::from_axis_angle(right, Rad(half_fov_rad));
        // let hori_rotation = Quaternion::from_axis_angle(up, Rad(half_fov_h_rad));

        let top_normal = {
            let first_rotation = Quaternion::from_axis_angle(right, Rad(half_fov_rad));
            let top_direction = first_rotation.rotate_vector(front);
            let second_rotation = Quaternion::from_axis_angle(right, Rad(-PI / 2.0));
            second_rotation.rotate_vector(top_direction).normalize()
        };
        debug!("Top Normal: {:#?}", top_normal);

        let bottom_normal = {
            let first_rotation = Quaternion::from_axis_angle(right, Rad(-half_fov_rad));
            let botton_direction = first_rotation.rotate_vector(front);
            let second_rotation = Quaternion::from_axis_angle(right, Rad(PI / 2.0));
            second_rotation.rotate_vector(botton_direction).normalize()
        };
        debug!("Bottom Normal: {:#?}", bottom_normal);

        let left_normal = {
            // let rotation = Quaternion::from_axis_angle(up_norm, Rad(half_fov_h_rad));
            // rotation.rotate_vector(right).normalize()

            let left_edge_direction = {
                let rotation = Quaternion::from_axis_angle(up_norm, Rad(half_fov_h_rad));
                rotation.rotate_vector(front).normalize()
            };
            // The left plane normal points inward (perpendicular to left edge, pointing right)
            left_edge_direction.cross(up_norm).normalize()
        };

        let right_normal = {
            // let rotation = Quaternion::from_axis_angle(up_norm, Rad(-half_fov_h_rad));
            // rotation.rotate_vector(-right).normalize()

            let right_edge_direction = {
                let rotation = Quaternion::from_axis_angle(up_norm, Rad(-half_fov_h_rad));
                rotation.rotate_vector(front).normalize()
            };
            // The right plane normal points inward (perpendicular to right edge, pointing left)
            up_norm.cross(right_edge_direction).normalize()
        };

        // debug!("Up Normal: {:#?}", up_norm);
        // debug!("Front Normal: {:#?}", front);
        // debug!("Right Normal: {:#?}", right);

        // debug!("Top Normal: {:#?}", top_normal);
        // debug!("Bot Normal: {:#?}", bottom_normal);
        // debug!("Lef Normal: {:#?}", top_normal);
        // debug!("Rig Normal: {:#?}", top_normal);

        self.top_face = Plane::from((position, top_normal));
        self.bottom_face = Plane::from((position, bottom_normal));
        self.left_face = Plane::from((position, left_normal));
        self.right_face = Plane::from((position, right_normal));

        self.near_face = Plane::from((position + znear * front, front));
        self.far_face = Plane::from((position + zfar * front, -front));

        // self.top_face = Plane::from((position, -vert_rotation.rotate_vector(up)));
        // self.bottom_face = Plane::from((position, (-1.0 * vert_rotation).rotate_vector(up)));

        // self.left_face = Plane::from((position, -(-1.0 * hori_rotation).rotate_vector(right)));
        // self.right_face = Plane::from((position, hori_rotation.rotate_vector(right)));

        // let half_v_side = zfar * f32::tan(fovy.to_radians() * 0.5);
        // let half_h_side = half_v_side * aspect;

        // let front = target.normalize();
        // let front_mult_far = zfar * front;
        // let right = front.cross(up);
        // let position = eye.to_homogeneous().truncate();

        // self.top_face = Plane::from((position, right.cross(front_mult_far - up * half_v_side)));

        // self.bottom_face =
        //     Plane::from((position, (front_mult_far + up * half_v_side).cross(right)));

        // self.left_face = Plane::from((position, up.cross(front_mult_far + right * half_h_side)));

        // self.right_face = Plane::from((position, (front_mult_far - right * half_h_side).cross(up)));

        // self.near_face = Plane::from((position + znear * front, front));
        // self.far_face = Plane::from((position + front_mult_far, -front));
    }

    pub(crate) fn contains(&self, radius: f32, center: impl Into<Vector3<f32>>) -> bool {
        let center = center.into();

        let is_on_or_forward_plane = |plane: &Plane| -> bool {
            // debug!("Distance: {}", plane.get_signed_distance(&center));
            // debug!("-Radius: {}", -radius);
            plane.get_signed_distance(&center) > -radius
        };

        is_on_or_forward_plane(&self.top_face) && is_on_or_forward_plane(&self.bottom_face)
        // && is_on_or_forward_plane(&self.left_face)
        // && is_on_or_forward_plane(&self.right_face)
        // && is_on_or_forward_plane(&self.near_face)
        // && is_on_or_forward_plane(&self.far_face)
    }
}
