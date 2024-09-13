#![allow(dead_code)]

use std::{cmp::max, time::Instant};

use bitvec::prelude::LocalBits;
use bitvec::vec::BitVec;
use log::{debug, info, log_enabled, Level};
use petgraph::{
    graph::NodeIndex,
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};

use crate::{
    cfn::{cost_function_network::*, relaxation::Relaxation},
    message::{message_trait::Message, messages::Messages},
};

use super::{
    solution::Solution,
    solver::{Solver, SolverOptions},
};

type PassIterator<'a> = Box<dyn Iterator<Item = &'a NodeIndex<usize>> + 'a>;

struct FactorSequence {
    sequence: Vec<NodeIndex<usize>>,
}

impl FactorSequence {
    fn new(relaxation: &Relaxation) -> Self {
        FactorSequence {
            sequence: relaxation
                .node_indices()
                .filter(|node_index| relaxation.has_edges(*node_index, Incoming))
                .collect(),
        }
    }

    fn pass_iter(&self, is_forward_pass: bool) -> PassIterator {
        match is_forward_pass {
            true => Box::new(self.sequence.iter()),
            false => Box::new(self.sequence.iter().rev()),
        }
    }

    fn new_pass_iters(&self) -> [PassIterator; 2] {
        [self.pass_iter(true), self.pass_iter(false)]
    }

    fn order(&mut self) {
        self.sequence.sort_unstable();
    }
}

pub struct NodeEdgeAttrs {
    edge_type: [BitVec<usize, LocalBits>; 2], // is_forward, is_backward
    edge_bound: BitVec<usize, LocalBits>, // if lower bound is updated using edge in backward pass
    node_bound: BitVec<usize, LocalBits>, // if lower bound is updated using node in backward pass
    node_weight: [Vec<f64>; 2],           // weight_forward, weight_backward
    node_weight_lb: Vec<i64>,             // for updating lower bound in backward pass
}

impl NodeEdgeAttrs {
    fn new_zero(num_nodes: usize, num_edges: usize) -> Self {
        NodeEdgeAttrs {
            edge_type: [
                BitVec::repeat(false, num_edges),
                BitVec::repeat(false, num_edges),
            ],
            edge_bound: BitVec::repeat(false, num_edges),
            node_bound: BitVec::repeat(false, num_nodes),
            node_weight: [vec![0.; num_nodes], vec![0.; num_nodes]],
            node_weight_lb: vec![0; num_nodes],
        }
    }

    fn new<'a>(relaxation: &Relaxation, factor_sequence: &'a FactorSequence) -> NodeEdgeAttrs {
        // Initialize node and edge attribute storage
        let mut node_edge_attrs =
            NodeEdgeAttrs::new_zero(relaxation.node_count(), relaxation.edge_count());

        // Save which factors are in `factor_sequence`
        let mut is_in_factor_sequence: BitVec<usize, LocalBits> =
            BitVec::repeat(false, relaxation.node_count());
        for node_index in &factor_sequence.sequence {
            is_in_factor_sequence.set(node_index.index(), true);
        }

        // Perform forward and backward passes
        for (pass, pass_iter) in factor_sequence.new_pass_iters().into_iter().enumerate() {
            // Keep track of which nodes were processed
            let mut is_touched = BitVec::<usize, LocalBits>::repeat(false, relaxation.node_count());

            // Consider factors in sequence in given order
            for &beta in pass_iter {
                // Count outgoing edges
                let mut outgoing_before: usize = 0;
                for gamma in relaxation.neighbors(beta, Outgoing) {
                    outgoing_before += (is_in_factor_sequence[gamma.index()]
                        && is_touched[gamma.index()])
                        as usize;
                }

                // Count incoming edges and determine which factors will update lower bound after message passing
                node_edge_attrs.node_bound.set(
                    beta.index(),
                    (pass == 0) && (!is_touched[beta.index()] || relaxation.is_unary_factor(beta)),
                );
                let mut incoming_total: usize = 0;
                let mut incoming_before: usize = 0;
                let mut incoming_not_before: usize = 0;
                for alpha in relaxation.neighbors(beta, Incoming) {
                    let is_alpha_processed = is_touched[alpha.index()];
                    node_edge_attrs
                        .edge_bound
                        .set(alpha.index(), (pass == 0) && !is_alpha_processed);
                    incoming_before += is_alpha_processed as usize;
                    incoming_not_before += !is_alpha_processed as usize;
                    incoming_total += 1;
                }

                // Set weights
                debug!(
                    "Edge counts: out before {}, in before {}, in not before {}, in total {}",
                    outgoing_before, incoming_before, incoming_not_before, incoming_total
                );
                let w = outgoing_before + max(incoming_before, incoming_not_before);
                node_edge_attrs.node_weight[pass][beta.index()] = 1. / w as f64;
                node_edge_attrs.node_weight_lb[beta.index()] = (pass == 1) as i64
                    * (i64::try_from(w).expect("Could not convert to signed int")
                        - incoming_total as i64);
                for alpha_beta in relaxation.edges_directed(beta, Incoming) {
                    node_edge_attrs.edge_type[pass].set(
                        alpha_beta.id().index(),
                        is_touched[alpha_beta.source().index()],
                    );
                    is_touched.set(alpha_beta.source().index(), true);
                }

                // Mark beta as processed
                is_touched.set(beta.index(), true);
            }
        }

        node_edge_attrs
    }
}

pub struct SRMP<'a> {
    relaxation: &'a Relaxation<'a>,
    messages: Messages,
    factor_sequence: FactorSequence,
    node_edge_attrs: NodeEdgeAttrs,
    initial_lower_bound: f64,
}

macro_rules! iter_messages {
    ($self:ident, $beta:ident, $edge_direction:expr, $pass_direction:expr, $edge:ident, $or_condition:expr) => {
        $self.relaxation.edges_directed($beta, $edge_direction)
        .filter(|$edge| {$self.node_edge_attrs.edge_type[$pass_direction][$edge.id().index()]} || $or_condition)
    };
}

impl<'a> SRMP<'a> {
    fn compute_solution(&self, solution: &mut Solution, beta: NodeIndex<usize>) {
        let beta_origin = self.relaxation.factor_origin(beta);
        if solution.is_fully_labeled(self.relaxation.cfn().factor_variables(beta_origin)) {
            return;
        }

        let theta_star = self
            .messages
            .compute_restricted_reparam(self.relaxation, beta, solution);

        // Choose a labeling with smallest `theta_star` cost
        match beta_origin {
            FactorOrigin::Variable(node_index) => {
                solution[*node_index] = Some(theta_star.index_min())
            }
            FactorOrigin::NonUnary(_hyperedge_index) => {
                theta_star.update_solution_restricted_minimum(
                    self.relaxation.cfn(),
                    beta_origin,
                    solution,
                );
            }
        }
    }

    fn one_directional_pass(
        &mut self,
        is_forward_pass: bool,
        compute_solution: &mut Option<Solution>,
    ) -> f64 {
        let pass_direction = 1 - is_forward_pass as usize;
        let mut lower_bound = self.initial_lower_bound;

        for &beta in self.factor_sequence.pass_iter(is_forward_pass) {
            // Line 4 of SRMP pseudocode: update messages along incoming edges "opposite" of pass direction
            if is_forward_pass {
                let incoming_opposite_pass =
                    iter_messages!(self, beta, Incoming, 1 - pass_direction, alpha_beta, false);
                for alpha_beta in incoming_opposite_pass {
                    self.messages.send(self.relaxation, alpha_beta);
                }
            } else {
                let incoming_opposite_pass = iter_messages!(
                    self,
                    beta,
                    Incoming,
                    1 - pass_direction,
                    alpha_beta,
                    self.node_edge_attrs.edge_bound[alpha_beta.id().index()] // also consider edges which update lower bound
                );
                for alpha_beta in incoming_opposite_pass {
                    let delta = self.messages.send(self.relaxation, alpha_beta);
                    lower_bound += (self.node_edge_attrs.edge_bound[alpha_beta.id().index()] as u8
                        as f64)
                        * delta;
                }
            }

            if let Some(labeling) = compute_solution {
                self.compute_solution(labeling, beta);
            }

            // Line 5 of SRMP pseudocode: compute reparametrization of beta
            let mut theta_beta = self.messages.compute_reparam(self.relaxation, beta);

            // Line 6 of SRMP pseudocode: update messages along incoming edges "in the same direction" as the pass
            theta_beta
                .mul_assign_scalar(self.node_edge_attrs.node_weight[pass_direction][beta.index()]);
            let incoming_same_as_pass =
                iter_messages!(self, beta, Incoming, pass_direction, alpha_beta, false);
            for alpha_beta in incoming_same_as_pass {
                self.messages.sub_assign_reparam(&theta_beta, alpha_beta);
            }

            if !is_forward_pass {
                // Take beta into account in lower bound
                let beta_weight_lb = self.node_edge_attrs.node_weight_lb[beta.index()];
                if self.node_edge_attrs.node_bound[beta.index()] && beta_weight_lb > 0 {
                    lower_bound += theta_beta.max() * beta_weight_lb as f64;
                }
            }
        }

        lower_bound
    }

    fn forward_pass(&mut self, compute_solution: &mut Option<Solution>) {
        self.one_directional_pass(true, compute_solution);
    }

    fn backward_pass(&mut self, compute_solution: &mut Option<Solution>) -> f64 {
        self.one_directional_pass(false, compute_solution)
    }

    fn init_solution(&mut self, compute_solution: bool) -> Option<Solution> {
        match compute_solution {
            true => Some(Solution::new(self.relaxation.cfn().num_variables())),
            false => None,
        }
    }
}

impl<'a> Solver<'a> for SRMP<'a> {
    fn init(relaxation: &'a Relaxation) -> Self {
        // Initialize messages
        let mut messages = Messages::new(&relaxation);

        // Find all factors with at least one incoming edge
        let mut factor_sequence = FactorSequence::new(&relaxation);

        // Order factor sequence
        // todo: different ordering procedures
        factor_sequence.order();

        // Count edges of each category and compute SRMP weights
        let node_edge_attrs = NodeEdgeAttrs::new(&relaxation, &factor_sequence);

        // Compute initial lower bound
        let mut initial_lower_bound = 0.;
        for node_index in relaxation
            .node_indices()
            .filter(|node_index| !relaxation.has_edges(*node_index, Outgoing))
        {
            initial_lower_bound += messages.send_srmp_initial(relaxation, node_index);
        }

        // Form and return SRMP struct
        SRMP {
            relaxation,
            messages,
            factor_sequence,
            node_edge_attrs,
            initial_lower_bound,
        }
    }

    fn run(mut self, options: &SolverOptions) -> Self {
        let time_start = Instant::now();
        let mut iteration = 0;
        let mut iter_solution = options.compute_solution_period();
        let mut compute_solution = true;
        let mut current_lower_bound;

        let mut best_solution = None;
        let mut best_cost = 0.;
        let mut forward_cost = 0.;
        let mut backward_cost = 0.;

        loop {
            // Forward pass
            let previous_lower_bound = self.initial_lower_bound;
            let mut forward_solution = self.init_solution(compute_solution);
            self.forward_pass(&mut forward_solution);
            if let Some(solution) = forward_solution {
                info!("Forward cost: {}. Forward solution {:#?}.", forward_cost, solution);
                forward_cost = self.relaxation.cfn().cost(&solution);
                if best_solution.is_none() || best_cost > forward_cost {
                    best_cost = forward_cost;
                    best_solution = Some(solution);
                }
            }

            // Backward pass
            let mut backward_solution = self.init_solution(compute_solution);
            current_lower_bound = self.backward_pass(&mut backward_solution);
            if let Some(solution) = backward_solution {
                backward_cost = self.relaxation.cfn().cost(&solution);
                info!("Backward cost: {}. Backward solution {:#?}.", backward_cost, solution);
                if best_solution.is_none() || best_cost > backward_cost {
                    best_cost = forward_cost;
                    best_solution = Some(solution);
                }
            }

            // Logging
            let elapsed_time = time_start.elapsed();
            if compute_solution {
                info!("Iteration {}. Elapsed time {:?}", iteration, elapsed_time);
                // todo: info!("Iteration {}. Elapsed time {:?}. Forward cost {}. Forward solution {:#?}. Baclward cost: {}. Backward solution {:#?}.", iteration, elapsed_time, forward_cost, forward_solution.unwrap(), backward_cost, backward_solution.unwrap());
            } else {
                info!("Iteration {}. Elapsed time {:?}", iteration, elapsed_time);
            }

            // Advance to next iteration
            iteration += 1;
            iter_solution -= compute_solution as usize * options.compute_solution_period();
            iter_solution += 1;
            compute_solution = (iter_solution == options.compute_solution_period())
                || (iteration + 1 == options.max_iterations());

            // Break if stopping a condition is satisfied
            if iteration >= options.max_iterations()
                || current_lower_bound < previous_lower_bound + options.eps()
                || elapsed_time >= options.time_max()
            {
                break;
            }
        }

        self
    }
}
