use boson::{BosonBody, BosonObject};

use crate::Transform3D;

pub trait BosonTransform {
    fn modify_transform<F, R>(&mut self, callback: F) -> R
    where
        F: FnOnce(&mut Transform3D) -> R;

    fn get_transform<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&Transform3D) -> R;
}

impl BosonTransform for BosonObject {
    fn modify_transform<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&mut Transform3D) -> R,
    {
        self.modify_body(|boson_body| match boson_body {
            BosonBody::PointMass(point_mass) => {}
            _ => {
                todo!()
            }
        })
    }

    fn get_transform<F, R>(&self, callback: F) -> R
    where
        F: FnOnce(&Transform3D) -> R,
    {
    }
}
