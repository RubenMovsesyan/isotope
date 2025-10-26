use boson::{BosonBody, BosonObject, RigidBody};
use cgmath::Vector3;

use crate::Transform3D;

pub trait BosonCompat {
    fn write_transform(&self, transform: &Transform3D);
    fn read_position<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&Vector3<f64>) -> R;
}

impl BosonCompat for BosonObject {
    fn write_transform(&self, transform: &Transform3D) {
        self.modify_body(|body| match body {
            // BosonBody::PointMass(point_mass) => transform.get_position(|pos| {
            //     point_mass.position.x = pos.x as f64;
            //     point_mass.position.y = pos.y as f64;
            //     point_mass.position.z = pos.z as f64;
            // }),
            _ => {}
        });
    }

    fn read_position<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&Vector3<f64>) -> R,
    {
        self.read_body(|body| match body {
            // BosonBody::PointMass(point_mass) => callback(&point_mass.position),
            BosonBody::RigidBody(rigid_body) => callback(&rigid_body.position),
            _ => {
                todo!()
            }
        })
    }
}
