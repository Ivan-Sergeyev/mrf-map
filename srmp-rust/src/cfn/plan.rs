#![allow(dead_code)]

// translation from SRMP.h:
// * Energy -> CostFunctionNetwork
// * Node -> Variable
// * Factor -> CostFunction

trait CostFunctionNetwork {
    fn new(max_variables: usize) -> Self;

    fn add_variable(self, domain_size: usize, costs: Vec<f64>) -> usize;
    fn add_unary_cost(self, var: usize, costs: Vec<f64>) -> usize;
    fn add_pairwise_cost(self, var: usize, costs: Vec<f64>) -> usize;
    fn add_cost(self, arity: usize, variables: Vec<usize>, costs: Vec<f64>) -> usize;

    fn num_variables(&self) -> usize;
    fn domain_size(&self, var: usize) -> usize;
}

// struct CFN1 {} // CFN2...
// impl CostFunctionNetwork for CFN1 {} // for CFN2...
// pub use cfn::CFN1 as MyCFN;

// with pub trait CostFunctionNetwork:
// Box<dyn CostFunctionNetwork> cfn;

trait VariableOrdering {
    // fn generate_ordering(&self) -> impl Iter;  // note: original implementation saves ordering in `nodes[k].solution`
}
// // simpler:
// enum VariableOrderings {
//     Original,
//     Reverse,
//     MinimalCostFirst,
// }
// fn generate_ordering(&self, ordering: VariableOrderings) -> impl Iterator; // outer or member function

pub struct SolverOptions {
    max_iterations: usize,
    max_time: std::time::Duration,
    eps: f64,
    extract_solution_period: usize,
    print_times: bool,
    verbose: bool,
}

// Vec<Vec<f64>>
// Vec<f64> + manual indexing
// Vec + override indexing

// nicer: builder pattern
// can also add generic type (based on algorithm)

// note: logging can be implemented via [log](https://docs.rs/log/latest/log/) and [env_logger](https://docs.rs/env_logger/latest/env_logger/)
// [example 1](https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/log.html#log-a-debug-message-to-the-console)
// [example 2](https://samkeen.dev/logging-in-rust-a-beginners-guide)

trait Solver<T>
where
    T: CostFunctionNetwork,
{
    fn new(options: SolverOptions) -> Self;
    // fn consume_cfn(self, cfn: T) -> Self;
    fn solve(self) -> f64;
    fn get_solution(&self) -> Vec<f64>;
}

// struct Solver1 {}
// impl<T> Solver<T> for Solver1 where T: CostFunctionNetwork {}

// struct Solver1<T> where T: CostFunctionNetwork { internal_cfn: T }
// impl<T> Solver<T> for Solver1<T> where T: CostFunctionNetwork {}

// solvers to implement: SRMP, MPLP, MPLP_BW, CMP
// // how to implement:
// enum SolverOptionTypes {SRMP, MPLP, MPLP_BW, CMP, TRWS(f64)}
// another idea: impl Solver<Solver1> for CFN where CFN: CostFunctionNetwork { ... }

trait ConstructRelaxation<RelaxationMethod> {
    fn construct_relaxation(self)
    where
        Self: CostFunctionNetwork;
    fn save_uai(&self); // todo: args; maybe move to CostFunctionNetwork
    fn print_stats(&self);
}

// // relaxations: SetMinimalEdges, SetFullEdges (3 methods), SefFullDualEdges (relies on VariableOrdering), Manual
// // how to implement:
// struct SetMinimalEdges { ... }
// impl ConstructRelaxation<SetMinimalEdges> for CFN { ... }  // note: specific implementation of CostFunctionNetwork

struct CostFunction {
    arity: usize,
    // todo: other fields?
}

// // todo: other structs?
// struct Variable;
// struct NonSingletonCostFunction;
// struct Edge;

trait CostFunctionType {
    // // todo: function signatures?
    // fn init_cost_function();
    // fn get_cost();
    // fn compute_partial_reparametrization();
    // fn init_edge();
    // fn send_message();  // several variants: SendMessage, SendRestrictedMessage, SendMPLPMessages (this probably belongs only in MPLP algorithm)
    // fn prep_cost_function();
}
