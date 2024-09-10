#![allow(dead_code)]

use std::time::Duration;

use crate::CostFunctionNetwork;

pub struct SolverOptions {
    max_iterations: usize,
    time_max: Duration,
    eps: f64,
    compute_solution_period: usize, // compute_solution_period = 0 means "never"
    print_times: bool,              // todo: move to logging
}

impl SolverOptions {
    pub fn default() -> Self {
        SolverOptions {
            max_iterations: 10000,
            time_max: Duration::new(20 * 60, 0), // 20 minutes
            eps: 1e-8,
            compute_solution_period: 10,
            print_times: true,
        }
    }

    pub fn max_iterations(&self) -> usize {
        self.max_iterations
    }

    pub fn time_max(&self) -> Duration {
        self.time_max
    }

    pub fn eps(&self) -> f64 {
        self.eps
    }

    pub fn compute_solution_period(&self) -> usize {
        self.compute_solution_period
    }

    pub fn print_times(&self) -> bool {
        self.print_times
    }
}

pub trait Solver<'a, CFN>
where
    CFN: CostFunctionNetwork,
{
    fn init(cfn: &'a CFN) -> Self;
    fn run(self, options: &SolverOptions) -> Self;
}
