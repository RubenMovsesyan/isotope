use std::time::Instant;

use cgmath::{ElementWise, InnerSpace, Vector2, Vector3, Zero};
use log::debug;

use crate::boson::collider::Collision;

use super::Solver;

const VELOCITY_THRESHOLD: f32 = -0.01;
const FRICTION_THRESHOLD: f32 = 0.0001;
const PENETRATION_FACTOR: f32 = 0.1;
// const REST_THRESHOLD: f32 = 0.1;

#[derive(Debug)]
pub struct ImpulseSolver;

impl Solver for ImpulseSolver {
    fn solve(&self, collisions: &mut [Collision], _delta_t: &Instant) {
        for collision in collisions.iter_mut() {
            // for convenience
            let points = &collision.points;

            // Impulse
            let mut impulse: Vector3<f32> = Vector3::zero();
            let mut friction: Vector3<f32> = Vector3::zero();

            let mut a_torque: Vector3<f32> = Vector3::zero();
            let mut b_torque: Vector3<f32> = Vector3::zero();

            collision.object_a.access(|a| {
                collision.object_b.access(|b| {
                    // Solving for impulse

                    let r_a = points.a_deep - a.get_pos();
                    let r_b = points.b_deep - b.get_pos();
                    let angular_a = a.get_angular_vel().cross(r_a);
                    let angular_b = b.get_angular_vel().cross(r_b);

                    // let r_velocity = (b.get_vel() + angular_b) - (a.get_vel() + angular_a);
                    let r_velocity = b.get_vel() - a.get_vel();
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

                        // Calculate impulse magnitude
                        let impulse_magnitude =
                            -(1.0 + elasticity) * modified_speed / (a_inv_mass + b_inv_mass);

                        impulse = impulse_magnitude * points.normal;

                        // Solving for friction
                        let mut tangent = r_velocity - velocity_along_normal * points.normal;

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

                            friction = -impulse_magnitude * tangent * mu;
                        }
                    }

                    // Calculate torques
                    a_torque = a.get_inv_inertia().mul_element_wise(r_a.cross(friction));
                    b_torque = b.get_inv_inertia().mul_element_wise(r_b.cross(-friction));
                });
            });

            // Apply modifications
            collision.object_a.modify(|a| {
                let a_inv_mass = a.get_inv_mass();
                a.vel(|velocity| {
                    *velocity -= impulse * a_inv_mass;
                    *velocity -= friction * a_inv_mass;
                });

                // a.angular_vel(|angular_velocity| {
                //     *angular_velocity -= a_torque;
                // });
            });

            collision.object_b.modify(|b| {
                let b_inv_mass = b.get_inv_mass();
                b.vel(|velocity| {
                    *velocity += impulse * b_inv_mass;
                    *velocity += friction * b_inv_mass;
                });

                // b.angular_vel(|angular_velocity| {
                //     *angular_velocity += b_torque;
                // });
            });
        }
    }
}
