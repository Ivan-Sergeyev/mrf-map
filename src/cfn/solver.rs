#![allow(dead_code)]

use std::time::Duration;

use crate::cfn::relaxation::Relaxation;

// Stores options to a cost function network solver
pub struct SolverOptions {
    max_iterations: usize, // maximum number of iterations
    time_max: Duration,    // maximum allowed time limit
    eps: f64,              // precision for tracking lower bound improvement
    compute_solution_period: usize, // number of iterations between solution recomputations
                           // if compute_solution_period = 0, the solution is never computed
}

impl SolverOptions {
    // Returns default options
    pub fn default() -> Self {
        SolverOptions {
            max_iterations: 10000,
            time_max: Duration::new(20 * 60, 0), // 20 minutes
            eps: 1e-8,
            compute_solution_period: 1,
        }
    }

    // Sets the maximum number of iterations
    pub fn set_max_iterations(&mut self, value: usize) -> &mut Self {
        self.max_iterations = value;
        self
    }

    // Sets the time limit
    pub fn set_time_max(&mut self, value: Duration) -> &mut Self {
        self.time_max = value;
        self
    }

    // Sets the precision for tracking lower bound improvement
    pub fn set_eps(&mut self, value: f64) -> &mut Self {
        self.eps = value;
        self
    }

    // Sets the number of iterations between solution recomputations
    pub fn set_compute_solution_period(&mut self, value: usize) -> &mut Self {
        self.compute_solution_period = value;
        self
    }

    // Returns the maximum number of iterations
    pub fn max_iterations(&self) -> usize {
        self.max_iterations
    }

    // Returns the time limit
    pub fn time_max(&self) -> Duration {
        self.time_max
    }

    // Returns the precision for tracking lower bound improvement
    pub fn eps(&self) -> f64 {
        self.eps
    }

    // Returns the number of iterations between solution recomputations
    pub fn compute_solution_period(&self) -> usize {
        self.compute_solution_period
    }
}

// Interface for cost function network solvers
pub trait Solver<'a> {
    // Initializes the solver with the given relaxation
    fn init(relaxation: &'a Relaxation) -> Self;

    // Executes the solver with the given options
    fn run(self, options: &SolverOptions) -> Self;
}
