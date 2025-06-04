use std::time::Instant;

use cgmath::{Vector3, Zero};

use crate::boson::collider::Collision;

use super::Solver;

const PERCENT: f32 = 0.8;
const SLOP: f32 = 0.01;

#[derive(Debug)]
pub struct PositionSolver;

impl Solver for PositionSolver {
    fn solve(&self, collisions: &mut [Collision], _delta_t: &Instant) {
        for collision in collisions.iter_mut() {
            // For convinience
            let points = &collision.points;

            let mut delta_a: Vector3<f32> = Vector3::zero();
            let mut delta_b: Vector3<f32> = Vector3::zero();

            collision.object_a.access(|a| {
                collision.object_b.access(|b| {
                    let a_inv_mass = a.get_inv_mass();
                    let b_inv_mass = b.get_inv_mass();

                    let correction = points.normal * PERCENT * f32::max(points.depth - SLOP, 0.0)
                        / (a_inv_mass + b_inv_mass);

                    delta_a = a_inv_mass * correction;
                    delta_b = b_inv_mass * correction;
                });
            });

            collision.object_a.modify(|a| {
                a.pos(|position| {
                    *position += delta_a;
                });
            });

            collision.object_b.modify(|b| {
                b.pos(|position| {
                    *position -= delta_b;
                });
            });
        }
    }
}
