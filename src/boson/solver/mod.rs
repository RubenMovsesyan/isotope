use std::{fmt::Debug, time::Instant};

use super::Collision;

pub mod basic_impulse_solver;
pub mod position_solver;
pub mod rotational_impulse_solver;

pub trait Solver: Debug + Send + Sync + 'static {
    fn solve(&self, collisions: &mut [Collision], delta_t: &Instant);
}
