use cgmath::{InnerSpace, Point3, Quaternion, Rad, Rotation, Rotation3, Vector3, Zero};

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
        let front = target.normalize();
        let up_norm = up.normalize();
        let right = front.cross(up_norm).normalize();

        let half_fov_rad = fovy.to_radians() * 0.5;
        let half_fov_h_rad = (half_fov_rad.tan() * aspect).atan() * 1.5; // Temp solution for now because there is something wrong with horizontal frustum math

        let top_normal = {
            let top_edge_direction = {
                let rotation = Quaternion::from_axis_angle(right, Rad(half_fov_rad));
                rotation.rotate_vector(front)
            };

            top_edge_direction.cross(right).normalize()
        };

        let bottom_normal = {
            let bottom_edge_direction = {
                let rotation = Quaternion::from_axis_angle(right, Rad(-half_fov_rad));
                rotation.rotate_vector(front)
            };

            bottom_edge_direction.cross(-right).normalize()
        };

        let left_normal = {
            let left_edge_direction = {
                let rotation = Quaternion::from_axis_angle(up_norm, Rad(half_fov_h_rad));
                rotation.rotate_vector(front)
            };

            left_edge_direction.cross(up_norm).normalize()
        };

        let right_normal = {
            let right_edge_direction = {
                let rotation = Quaternion::from_axis_angle(up_norm, Rad(-half_fov_h_rad));
                rotation.rotate_vector(front)
            };

            right_edge_direction.cross(-up_norm).normalize()
        };

        self.top_face = Plane::from((position, top_normal));
        self.bottom_face = Plane::from((position, bottom_normal));
        self.left_face = Plane::from((position, left_normal));
        self.right_face = Plane::from((position, right_normal));

        self.near_face = Plane::from((position + znear * front, front));
        self.far_face = Plane::from((position + zfar * front, -front));
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
