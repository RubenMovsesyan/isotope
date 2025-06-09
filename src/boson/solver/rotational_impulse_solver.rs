use cgmath::{InnerSpace, SquareMatrix, Vector2, Vector3, Zero};
use log::*;
use std::time::Instant;

use crate::boson::collider::Collision;

use super::Solver;

const VELOCITY_THRESHOLD: f32 = -0.01;
const FRICTION_THRESHOLD: f32 = 0.0001;
const PENETRATION_FACTOR: f32 = 0.1;

#[derive(Debug)]
pub struct RotationalImpulseSolver;

impl Solver for RotationalImpulseSolver {
    fn solve(&self, collisions: &mut [Collision], _delta_t: &Instant) {
        for collision in collisions.iter_mut() {
            // for convenience
            let points = &collision.points;

            // Impulse
            let mut impulse: Vector3<f32> = Vector3::zero();
            let mut friction: Vector3<f32> = Vector3::zero();
            let mut ra: Vector3<f32> = Vector3::zero();
            let mut rb: Vector3<f32> = Vector3::zero();

            let mut angular_vel_a_change: Vector3<f32> = Vector3::zero();
            let mut angular_vel_b_change: Vector3<f32> = Vector3::zero();

            // let mut angular_friction_a: Vector3<f32> = Vector3::zero();
            // let mut angular_friction_b: Vector3<f32> = Vector3::zero();

            match collision.object_a.access(|a| {
                collision.object_b.access(|b| {
                    // Solving for impulse
                    let angular_vel_a = a.get_angular_vel();
                    let angular_vel_b = b.get_angular_vel();

                    ra = points.contact_point - a.get_pos();
                    rb = points.contact_point - b.get_pos();

                    let r_velocity = (b.get_vel() + angular_vel_b) - (a.get_vel() + angular_vel_a);
                    let velocity_along_normal = r_velocity.dot(points.normal);

                    if velocity_along_normal > VELOCITY_THRESHOLD {
                        // For convenience
                        let a_inv_mass = a.get_inv_mass();
                        let b_inv_mass = b.get_inv_mass();

                        if a_inv_mass == 0.0 && b_inv_mass == 0.0 {
                            return;
                        }

                        let elasticity = a.get_restitution() * b.get_restitution();
                        let penetration_speed = points.depth * PENETRATION_FACTOR;
                        let modified_speed = velocity_along_normal - penetration_speed;

                        let a_inv_inertia = a
                            .get_inertia()
                            .invert()
                            .expect("Failed to invert the matrix");

                        let b_inv_inertia = b
                            .get_inertia()
                            .invert()
                            .expect("Failed to invert the matrix");

                        let ra_cross_n = ra.cross(points.normal);
                        let rb_cross_n = rb.cross(points.normal);

                        let denom = a_inv_mass
                            + b_inv_mass
                            + ((a_inv_inertia * ra_cross_n).cross(ra)
                                + (b_inv_inertia * rb_cross_n).cross(rb))
                            .dot(points.normal);

                        // Calculate impulse magnitude
                        let impulse_magnitude = -(1.0 + elasticity) * modified_speed / denom;

                        impulse = impulse_magnitude * points.normal;

                        // Solving for friction
                        let mut tangent = r_velocity - velocity_along_normal * points.normal;

                        angular_vel_a_change = impulse_magnitude * (a_inv_inertia * ra_cross_n);
                        angular_vel_b_change = impulse_magnitude * (b_inv_inertia * rb_cross_n);

                        if tangent.magnitude() > FRICTION_THRESHOLD {
                            tangent = tangent.normalize();

                            let friction_velocity = r_velocity.dot(tangent);

                            // Calcuate the friction coefficient (using static or dynamic friction)
                            let mu: f32 = if friction_velocity.abs() < 0.01 {
                                Vector2::new(a.get_static_friction(), b.get_static_friction())
                                    .magnitude()
                            } else {
                                Vector2::new(a.get_dynamic_friction(), b.get_dynamic_friction())
                                    .magnitude()
                            };

                            friction = impulse_magnitude * tangent * mu;
                            // let torque_mag_a = ra.magnitude() * friction.magnitude();
                            // let torque_mag_b = rb.magnitude() * friction.magnitude();
                            // angular_vel_a_change = torque_mag_a * tangent.normalize();
                            // angular_vel_b_change = torque_mag_b * tangent.normalize();

                            // angular_friction_a = a_inv_inertia * ra.cross(friction);
                            // angular_friction_b = b_inv_inertia * rb.cross(friction);
                        }
                    }
                })
            }) {
                Err(err) => {
                    error!("Failed to access boson objects due to: {}", err);
                }
                _ => {}
            }

            // Apply modifications
            match collision.object_a.modify(|a| {
                let a_inv_mass = a.get_inv_mass();
                a.vel(|velocity| {
                    *velocity -= impulse * a_inv_mass;
                    *velocity -= friction * a_inv_mass;
                });

                a.angular_vel(|angular_velocity| {
                    *angular_velocity -= angular_vel_a_change;
                });
            }) {
                Err(err) => {
                    error!("Failed to modify boson objects due to: {}", err);
                }
                _ => {}
            }

            match collision.object_b.modify(|b| {
                let b_inv_mass = b.get_inv_mass();
                b.vel(|velocity| {
                    *velocity += impulse * b_inv_mass;
                    *velocity += friction * b_inv_mass;
                });

                b.angular_vel(|angular_velocity| {
                    *angular_velocity -= angular_vel_b_change;
                });
            }) {
                Err(err) => {
                    error!("Failed to modify boson objects due to: {}", err);
                }
                _ => {}
            }
        }
    }
}
