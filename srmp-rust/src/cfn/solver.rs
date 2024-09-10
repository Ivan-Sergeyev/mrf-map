#![allow(dead_code)]

use std::cmp::max;

use bitvec::{order::Msb0, vec::BitVec};
use petgraph::{
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};

use crate::cfn::cost_function_network::*;

use super::relaxation::{ConstructRelaxation, MinimalEdges, RelaxationGraph};

pub struct SolverOptions {
    max_iterations: usize,
    max_time: std::time::Duration,
    eps: f64,
    extract_solution_period: usize,
    print_times: bool,
}

trait Solver<CFN>
where
    CFN: CostFunctionNetwork + ConstructRelaxation<MinimalEdges>,
{
    fn init(cfn: CFN, options: SolverOptions) -> Self;
    fn solve(self) -> Self;
}

struct SRMP<CFN>
where
    CFN: CostFunctionNetwork + ConstructRelaxation<MinimalEdges>,
{
    cfn: CFN,
    relaxation: RelaxationGraph,
    options: SolverOptions,
    lower_bound: f64,
}

impl Solver<GeneralCFN> for SRMP<GeneralCFN> {
    fn init(cfn: GeneralCFN, options: SolverOptions) -> Self {
        let lower_bound = 0.;

        // Convert CFN to normal form
        // todo: cfn.to_normal_form();

        // Construct relaxation graph
        let relaxation_graph = cfn.construct_relaxation();

        // todo: prune relaxation (if there is factor aplha with a single child beta then we can reparametrize theta
        // to get min_{x_a ~ x_b} theta_a(x_a) = 0 for all x_b and then remove alpha; this won't affect the relaxation)

        // Find all factors with at least one incoming edge
        let mut factor_sequence = Vec::new();
        let mut factor_sequence_indices = vec![None; relaxation_graph.node_count()];
        for node_index in relaxation_graph.node_indices() {
            if relaxation_graph
                .neighbors_directed(node_index, Incoming)
                .next()
                .is_some()
            {
                factor_sequence_indices[node_index.index()] = Some(factor_sequence.len());
                factor_sequence.push(node_index);
            }
        }

        // Order factor sequence
        // todo: different ordering procedures
        factor_sequence.sort();

        // Count edges of each category and compute SRMP weights
        let mut is_processed;

        // - Forward pass
        let mut is_forward = BitVec::<u64, Msb0>::repeat(false, relaxation_graph.edge_count());
        let mut weight_forward = vec![0.; relaxation_graph.edge_count()];
        is_processed = BitVec::<u64, Msb0>::repeat(false, relaxation_graph.node_count());

        for &beta_index in factor_sequence.iter() {
            // Count outgoing edges
            let mut outgoing_before: usize = 0;
            for gamma in relaxation_graph.neighbors_directed(beta_index, Outgoing) {
                if let Some(_) = factor_sequence_indices[gamma.index()] {
                    if is_processed[gamma.index()] {
                        outgoing_before += 1;
                    }
                }
            }

            // Count incoming edges
            let mut incoming_before: usize = 0;
            let mut incoming_not_before: usize = 0;
            for alpha in relaxation_graph.neighbors_directed(beta_index, Incoming) {
                if is_processed[alpha.index()] {
                    incoming_before += 1;
                } else {
                    incoming_not_before += 1;
                }
            }

            // Set weights
            for alpha_beta in relaxation_graph.edges_directed(beta_index, Incoming) {
                let edge_index = alpha_beta.id().index();
                let alpha = alpha_beta.id().index();
                if is_processed[alpha] {
                    is_forward.set(edge_index, true);
                    let denominator = outgoing_before + max(incoming_before, incoming_not_before);
                    weight_forward[edge_index] = 1. / denominator as f64;
                } else {
                    // These are not needed, but left for clarity:
                    // is_forward.set(edge_index, false);
                    // weight_forward[edge_index] = 0.;
                    is_processed.set(alpha, true);
                }
            }

            // Mark beta as processed
            is_processed.set(beta_index.index(), true);
        }

        // - Backward counting pass
        let mut is_backward = BitVec::<u64, Msb0>::repeat(false, relaxation_graph.edge_count());
        let mut weight_backward = vec![0.; relaxation_graph.edge_count()];
        is_processed = BitVec::<u64, Msb0>::repeat(false, relaxation_graph.node_count());

        for &beta_index in factor_sequence.iter().rev() {
            // Count outgoing edges
            let mut outgoing_before: usize = 0;
            for gamma in relaxation_graph.neighbors_directed(beta_index, Outgoing) {
                if let Some(_) = factor_sequence_indices[gamma.index()] {
                    if is_processed[gamma.index()] {
                        outgoing_before += 1;
                    }
                }
            }

            // Count incoming edges
            let mut incoming_before: usize = 0;
            let mut incoming_not_before: usize = 0;
            for alpha in relaxation_graph.neighbors_directed(beta_index, Incoming) {
                if is_processed[alpha.index()] {
                    incoming_before += 1;
                } else {
                    incoming_not_before += 1;
                }
            }

            // Set weights
            for alpha_beta in relaxation_graph.edges_directed(beta_index, Incoming) {
                let edge_index = alpha_beta.id().index();
                let alpha = alpha_beta.id().index();
                if is_processed[alpha] {
                    is_backward.set(edge_index, true);
                    let denominator = outgoing_before + max(incoming_before, incoming_not_before);
                    weight_backward[edge_index] = 1. / denominator as f64;
                } else {
                    // These are not needed, but left for clarity:
                    // is_backward.set(edge_index, false);
                    // weight_backward[edge_index] = 0.;
                    is_processed.set(alpha, true);
                }
            }

            // Mark beta as processed
            is_processed.set(beta_index.index(), true);
        }

        SRMP {
            cfn: cfn,
            relaxation: relaxation_graph,
            options: options,
            lower_bound: lower_bound,
        }
    }

    fn solve(self) -> Self {
        todo!()

        // summary:
        // loop until stopping criterion
        // do forward pass and backward pass
        // update messages, recompute potentials, s
    }
}
