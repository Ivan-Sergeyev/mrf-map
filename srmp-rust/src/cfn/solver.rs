use crate::cfn::cost_function_network::*;

pub struct SolverOptions {
    max_iterations: usize,
    max_time: std::time::Duration,
    eps: f64,
    extract_solution_period: usize,
    print_times: bool,
    verbose: bool,
}

trait Solver<CFN> where CFN: CostFunctionNetwork {
    fn init(options: SolverOptions, cfn: CFN) -> Self;
    fn solve(self) -> f64;
    fn get_solution(&self) -> &Vec<f64>;
}

struct SRMP<CFN> where CFN: CostFunctionNetwork {
    options: SolverOptions,
    cfn: CFN,
    solution: Vec<f64>,
    lower_bound: f64,

}

impl<CFN> Solver<CFN> for SRMP<CFN> where CFN: CostFunctionNetwork {
    fn init(options: SolverOptions, cfn: CFN) -> Self {
        let num_variables = cfn.num_variables();
        todo!();
        SRMP {
            options: options,
            cfn: cfn,
            solution: vec![0.; num_variables],
            lower_bound: 0.,
        }

        // summary:
        // lb_init = 0;
        // count number of (non-removed) nodes and (non-removed and "first_in" (?)) factors, set some tmp variables to 0
        // -- for some factors (if "factor->first_out" is null (?)), call SEND_MPLP_MESSAGES(factor) and update LB, but don't update solution
        // sort pointers to nodes and factors (see "order.cpp")
        // determine maximum factor arity and maximum domain size (among nodes and factors selected above)
        // determine which edges are forward and backward, and for which to compute the bound (aka I^+_\beta and I^-_\beta from paper)
        // compute weights omega_\alpha \beta
    }

    fn solve(self) -> f64 {
        todo!()

        // summary:
        // loop until stopping criterion
        // do forward pass and backward pass
        // update messages, recompute potentials, s
    }

    fn get_solution(&self) -> &Vec<f64> {
        &self.solution
    }
}
