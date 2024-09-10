#![allow(dead_code)]

use std::{cmp::max, time::Instant};

use bitvec::prelude::LocalBits;
use bitvec::vec::BitVec;
use petgraph::{
    graph::NodeIndex,
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};

use crate::cfn::cost_function_network::*;

use super::{
    factor_types::{Factor, FactorType},
    relaxation::{ConstructRelaxation, MinimalEdges, RelaxationGraph},
};

pub struct SolverOptions {
    max_iterations: usize,
    time_max: std::time::Duration,
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

struct SRMPEdgeTypesWeights {
    edge_incoming_types: [BitVec<usize, LocalBits>; 2], // is_forward, is_backward
    edge_weights: [Vec<f64>; 2],                        // weight_forward, weight_backward
}

struct SRMP<CFN>
where
    CFN: CostFunctionNetwork + ConstructRelaxation<MinimalEdges>,
{
    cfn: CFN,
    relaxation_graph: RelaxationGraph,
    options: SolverOptions,
    factor_sequence: Vec<NodeIndex<usize>>,
    // factor_sequence_indices: Vec<Option<usize>>,
    edge_directions_weights: SRMPEdgeTypesWeights,
    messages: Vec<FactorType>,
    lower_bound: f64,
}

fn srmp_count_edges_compute_weights(
    relaxation_graph: &RelaxationGraph,
    factor_sequence: &Vec<NodeIndex<usize>>,
    is_in_factor_sequence: &BitVec,
) -> SRMPEdgeTypesWeights {
    // Initialize results storage
    let mut edge_types_weights = SRMPEdgeTypesWeights {
        edge_incoming_types: [
            BitVec::repeat(false, relaxation_graph.edge_count()),
            BitVec::repeat(false, relaxation_graph.edge_count()),
        ],
        edge_weights: [
            vec![0.; relaxation_graph.edge_count()],
            vec![0.; relaxation_graph.edge_count()],
        ],
    };

    // Create forward and backward iterators over factor sequence
    let pass_iters: [Box<dyn Iterator<Item = _>>; 2] = [
        Box::new(factor_sequence.iter()),
        Box::new(factor_sequence.iter().rev()),
    ];

    // Perform forward pass and backward pass
    for (pass, pass_iter) in pass_iters.into_iter().enumerate() {
        // Keep track of which nodes were processed
        let mut is_processed =
            BitVec::<usize, LocalBits>::repeat(false, relaxation_graph.node_count());

        // Consider factors in sequence in given order
        for &beta in pass_iter {
            // Count outgoing edges
            let mut outgoing_before: usize = 0;
            for gamma in relaxation_graph.neighbors_directed(beta, Outgoing) {
                outgoing_before +=
                    (is_in_factor_sequence[gamma.index()] && is_processed[gamma.index()]) as usize;
            }

            // Count incoming edges
            let mut incoming_before: usize = 0;
            let mut incoming_not_before: usize = 0;
            for alpha in relaxation_graph.neighbors_directed(beta, Incoming) {
                incoming_before += is_processed[alpha.index()] as usize;
                incoming_not_before += !is_processed[alpha.index()] as usize;
            }

            // Set weights
            for alpha_beta in relaxation_graph.edges_directed(beta, Incoming) {
                if is_processed[alpha_beta.source().index()] {
                    edge_types_weights.edge_incoming_types[pass]
                        .set(alpha_beta.source().index(), true);
                    edge_types_weights.edge_weights[pass][alpha_beta.id().index()] =
                        1. / (outgoing_before + max(incoming_before, incoming_not_before)) as f64;
                } else {
                    // Mark alpha as processed
                    is_processed.set(alpha_beta.source().index(), true);

                    // // Assignments not needed, left as a comment for clarity
                    // edge_types_weights.edge_incoming_types[pass].set(alpha_beta.source().index(), false);
                    // edge_types_weights.edge_weights[pass][alpha_beta.id().index()] = 0.;
                }
            }

            // Mark beta as processed
            is_processed.set(beta.index(), true);
        }
    }

    edge_types_weights
}

impl SRMP<GeneralCFN> {
    fn srmp_main_iteration(&mut self) {
        // Create forward and backward iterators over factor sequence
        let pass_iters: [Box<dyn Iterator<Item = _>>; 2] = [
            Box::new(self.factor_sequence.iter()),
            Box::new(self.factor_sequence.iter().rev()),
        ];

        // Perform forward pass and backward pass
        for (pass, pass_iter) in pass_iters.into_iter().enumerate() {
            // Consider factors in sequence in given order
            for &beta in pass_iter {
                // Update incoming messages that go "opposite of iteration order"
                for alpha_beta in
                    self.relaxation_graph
                        .edges_directed(beta, Incoming)
                        .filter(|alpha_beta| {
                            self.edge_directions_weights.edge_incoming_types[1 - pass]
                                [alpha_beta.id().index()]
                        })
                {
                    // Send message along (alpha, beta)
                    // - ie update message_ab[xb] = min(theta_hat_a[xa] + ... - ...) over xa ~ xb
                }

                // Compute theta_beta
                // - Initialize to copy of beta (or zero if beta is an empty unary factor)
                let mut theta_beta = self
                    .cfn
                    .get_factor_copy(self.relaxation_graph.node_weight(beta).unwrap());

                // - Subtract all outgoing messages
                for gamma in self.relaxation_graph.edges_directed(beta, Outgoing) {

                }

                // - Add all incoming messages
                for alpha in self.relaxation_graph.edges_directed(beta, Incoming) {

                }
                // - multiply by "iteration order" weight of beta (which is the same for all "iteration order" incoming edges)

                // Update incoming messages that go "in iteration order"
                for alpha_beta in
                    self.relaxation_graph
                        .edges_directed(beta, Incoming)
                        .filter(|alpha_beta| {
                            self.edge_directions_weights.edge_incoming_types[pass]
                                [alpha_beta.id().index()]
                        })
                {
                    // subtract theta_beta times "iteration order" weight of alpha_beta from message_ab
                }
            }
        }

        // updating LB:
        // in the backward pass,
        //
    }
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
        let mut is_in_factor_sequence = BitVec::repeat(false, relaxation_graph.node_count());
        // let mut factor_sequence_indices = vec![None; relaxation_graph.node_count()];
        for node_index in relaxation_graph.node_indices() {
            if relaxation_graph
                .neighbors_directed(node_index, Incoming)
                .next()
                .is_some()
            {
                is_in_factor_sequence.set(node_index.index(), true);
                // factor_sequence_indices[node_index.index()] = Some(factor_sequence.len());
                factor_sequence.push(node_index);
            }
        }

        // Order factor sequence
        // todo: different ordering procedures
        factor_sequence.sort();

        // Count edges of each category and compute SRMP weights
        let edge_directions_weights = srmp_count_edges_compute_weights(
            &relaxation_graph,
            &factor_sequence,
            &is_in_factor_sequence,
        );

        // Create initial (zero) messages
        let mut messages = Vec::with_capacity(relaxation_graph.edge_count());
        for edge in relaxation_graph.edge_references() {
            messages.push(cfn.new_message(relaxation_graph.node_weight(edge.target()).unwrap()));
        }

        // Form and return struct
        SRMP {
            cfn: cfn,
            relaxation_graph: relaxation_graph,
            options: options,
            factor_sequence: factor_sequence,
            // is_: factor_sequence_indices,
            edge_directions_weights: edge_directions_weights,
            messages: messages,
            lower_bound: lower_bound,
        }
    }

    fn solve(self) -> Self {
        let time_start = Instant::now();
        let mut iteration = 0;
        let mut previous_lower_bound;
        // let mut reparametrization_checkpoint;

        loop {
            previous_lower_bound = self.lower_bound;
            // todo: periodically compute current solution

            // todo: keep track of "checkpoint" reparametrization

            iteration += 1;
            if iteration >= self.options.max_iterations
                || self.lower_bound < previous_lower_bound + self.options.eps
                || Instant::now() - time_start >= self.options.time_max
            {
                break;
            }
        }

        self
    }
}
