#![allow(dead_code)]

use std::{
    cmp::max,
    time::{Duration, Instant},
};

use bitvec::prelude::LocalBits;
use bitvec::vec::BitVec;
use petgraph::{
    graph::NodeIndex,
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};

use crate::cfn::{
    cost_function_network::*,
    factor_types::FactorType,
    relaxation::{RelaxationGraph, RelaxationMinimalEdges, RelaxationType},
};

use super::{
    message_passing::MessagePassingMechanism,
    solver::{Solver, SolverOptions},
};

pub struct SRMPEdgeAttrs {
    is_type: [BitVec<usize, LocalBits>; 2], // is_forward, is_backward
    weight: [Vec<f64>; 2],                  // weight_forward, weight_backward
}

pub struct SRMP<'a, CFN, Relaxation>
where
    CFN: CostFunctionNetwork,
    Relaxation: RelaxationType<CFN>,
{
    message_passing_mechanism: MessagePassingMechanism<'a, CFN, Relaxation>,
    factor_sequence: Vec<NodeIndex<usize>>,
    edge_attrs: SRMPEdgeAttrs,
    lower_bound: f64,
}

fn generate_edge_attrs(
    relaxation_graph: &RelaxationGraph,
    factor_sequence: &Vec<NodeIndex<usize>>,
    is_in_factor_sequence: &BitVec,
) -> SRMPEdgeAttrs {
    // Initialize results storage
    let mut edge_attrs = SRMPEdgeAttrs {
        is_type: [
            BitVec::repeat(false, relaxation_graph.edge_count()),
            BitVec::repeat(false, relaxation_graph.edge_count()),
        ],
        weight: [
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
                    edge_attrs.is_type[pass].set(alpha_beta.source().index(), true);
                    edge_attrs.weight[pass][alpha_beta.id().index()] =
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

    edge_attrs
}

impl SRMP<'_, GeneralCFN, RelaxationMinimalEdges> {
    fn main_iteration(&mut self, compute_solution: bool) {
        // Create forward and backward iterators over factor sequence
        let pass_iters: [Box<dyn Iterator<Item = _>>; 2] = [
            Box::new(self.factor_sequence.iter()),
            Box::new(self.factor_sequence.iter().rev()),
        ];

        // Create shorthands
        let cfn = self.message_passing_mechanism.cfn;
        let relaxation_graph = self.message_passing_mechanism.relaxation.graph();

        // Perform forward pass and backward pass
        for (pass, pass_iter) in pass_iters.into_iter().enumerate() {
            // Consider factors in sequence in given order
            for &beta in pass_iter {
                // Update incoming messages that go "opposite of iteration order"
                for alpha_beta in relaxation_graph
                    .edges_directed(beta, Incoming)
                    .filter(|alpha_beta| self.edge_attrs.is_type[1 - pass][alpha_beta.id().index()])
                {
                    self.message_passing_mechanism.send_message(alpha_beta);
                }

                // Compute theta_beta
                // - Initialize to copy of beta (or zero if beta is an empty unary factor)
                let mut theta_beta = cfn
                    .factor_clone_for_message_passing(relaxation_graph.node_weight(beta).unwrap());

                // - Subtract all outgoing messages
                for gamma in relaxation_graph.edges_directed(beta, Outgoing) {
                    todo!()
                }

                // - Add all incoming messages
                for alpha in relaxation_graph.edges_directed(beta, Incoming) {
                    todo!()
                }
                // - multiply by "iteration order" weight of beta (which is the same for all "iteration order" incoming edges)

                // Update incoming messages that go "in iteration order"
                for alpha_beta in relaxation_graph
                    .edges_directed(beta, Incoming)
                    .filter(|alpha_beta| self.edge_attrs.is_type[pass][alpha_beta.id().index()])
                {
                    // subtract theta_beta times "iteration order" weight of alpha_beta from message_ab
                }
            }
        }

        todo!()
        // updating LB:
        // in the backward pass,
        //
    }
}

impl<'a> Solver<'a, GeneralCFN> for SRMP<'a, GeneralCFN, RelaxationMinimalEdges> {
    fn init(cfn: &'a GeneralCFN) -> Self {
        let lower_bound = 0.;

        // Convert CFN to normal form
        // todo: cfn.to_normal_form();

        // Construct relaxation graph
        let relaxation = RelaxationMinimalEdges::construct_relaxation(cfn);

        // todo: prune relaxation (if there is factor aplha with a single child beta then we can reparametrize theta
        // to get min_{x_a ~ x_b} theta_a(x_a) = 0 for all x_b and then remove alpha; this won't affect the relaxation)

        // Find all factors with at least one incoming edge
        let mut factor_sequence = Vec::new();
        let mut is_in_factor_sequence = BitVec::repeat(false, relaxation.graph().node_count());
        for node_index in relaxation.graph().node_indices() {
            if relaxation
                .graph()
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
        factor_sequence.sort_unstable();

        // Count edges of each category and compute SRMP weights
        let edge_directions_weights = generate_edge_attrs(
            &relaxation.graph(),
            &factor_sequence,
            &is_in_factor_sequence,
        );

        // Form and return struct
        SRMP {
            message_passing_mechanism: MessagePassingMechanism::new(cfn, relaxation),
            factor_sequence: factor_sequence,
            edge_attrs: edge_directions_weights,
            lower_bound: lower_bound,
        }
    }

    fn run(mut self, options: &SolverOptions) -> Self {
        // Assumption: options.compute_solution_period > 0

        let time_start = Instant::now();
        let mut iteration = 0;
        let mut iter_solution = options.compute_solution_period();
        let mut compute_solution = true;
        let mut previous_lower_bound;

        loop {
            previous_lower_bound = self.lower_bound;

            self.main_iteration(compute_solution);

            iteration += 1;

            //
            iter_solution -= compute_solution as usize * options.compute_solution_period();
            iter_solution += 1;
            compute_solution = iter_solution == options.compute_solution_period();

            if iteration >= options.max_iterations()
                || self.lower_bound < previous_lower_bound + options.eps()
                || Instant::now() - time_start >= options.time_max()
            {
                break;
            }
        }

        self
    }
}
