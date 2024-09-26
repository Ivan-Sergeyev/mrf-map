#![allow(dead_code)]

use std::{cmp::max, time::Instant};

use bitvec::prelude::LocalBits;
use bitvec::vec::BitVec;
use log::info;
use petgraph::{
    graph::NodeIndex,
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};

use crate::{
    cfn::relaxation::Relaxation,
    message::{message_trait::Message, messages::Messages},
};

use super::{
    solution::Solution,
    solver::{Solver, SolverOptions},
};

type PassIterator<'a> = Box<dyn Iterator<Item = &'a NodeIndex<usize>> + 'a>;

// Stores the sequence of factors considered in the SRMP algorithm
struct FactorSequence {
    sequence: Vec<NodeIndex<usize>>, // contains node indices in the relaxation grpah
}

impl FactorSequence {
    // Creates a factor sequence for the given relaxation
    // (i.e., all unary factors and all factors with at least one incoming edge)
    fn new(relaxation: &Relaxation) -> Self {
        FactorSequence {
            sequence: relaxation
                .node_indices()
                .filter(|node_index| {
                    relaxation.is_unary_factor(*node_index)
                        || relaxation.has_edges(*node_index, Incoming)
                })
                .collect(),
        }
    }

    // Sorts the factor sequence
    fn sort(mut self) -> Self {
        // todo: add options for different sorting criteria
        self.sequence.sort_unstable();
        self
    }
}

// Stores the attributes used in the computations in the forward and backward passes
#[derive(Debug)]
pub struct NodeEdgeAttrs {
    edge_is_forward: BitVec<usize, LocalBits>, // is_fw from cpp // todo: better desc
    edge_is_backward: BitVec<usize, LocalBits>, // is_bw from cpp // todo: better desc
    edge_is_update_lb: BitVec<usize, LocalBits>, // if the lower bound is updated via the edge in the backward pass
    node_is_update_lb: BitVec<usize, LocalBits>, // if the lower bound is updated via the node in the backward pass
    node_weight_forward: Vec<usize>,             // weight_forward from cpp // todo: better desc
    node_weight_backward: Vec<usize>,            // weight_backward from cpp // todo: better desc
    node_weight_update_lb: Vec<usize>, // weight for updating the lower bound in the backward pass
}

impl NodeEdgeAttrs {
    // Initializes all attributes to zero
    fn zero(num_nodes: usize, num_edges: usize) -> Self {
        NodeEdgeAttrs {
            edge_is_forward: BitVec::repeat(false, num_edges),
            edge_is_backward: BitVec::repeat(false, num_edges),
            edge_is_update_lb: BitVec::repeat(false, num_edges),
            node_is_update_lb: BitVec::repeat(false, num_nodes),
            node_weight_forward: vec![0; num_nodes],
            node_weight_backward: vec![0; num_nodes],
            node_weight_update_lb: vec![0; num_nodes],
        }
    }

    // Computes attributes based on the given relaxation and factor sequence
    fn new<'a>(relaxation: &Relaxation, factor_sequence: &'a FactorSequence) -> NodeEdgeAttrs {
        // Initialize node and edge attribute storage
        let mut attrs = NodeEdgeAttrs::zero(relaxation.node_count(), relaxation.edge_count());

        // Label backward edges
        let mut is_touched = BitVec::<usize, LocalBits>::repeat(false, relaxation.node_count());
        for factor in factor_sequence.sequence.iter() {
            let alpha = factor.index();
            attrs.node_is_update_lb.set(
                alpha,
                !is_touched[alpha] || relaxation.is_unary_factor(*factor),
            );
            is_touched.set(alpha, true);

            for in_edge in relaxation.edges_directed(*factor, Incoming) {
                let alpha_beta = in_edge.id().index();
                let beta = in_edge.source().index();
                attrs.edge_is_backward.set(alpha_beta, is_touched[beta]);
                attrs.edge_is_update_lb.set(alpha_beta, !is_touched[beta]);
                is_touched.set(beta, true);
            }
        }

        // Label forward edges
        let mut is_touched = BitVec::<usize, LocalBits>::repeat(false, relaxation.node_count());
        for factor in factor_sequence.sequence.iter().rev() {
            let alpha = factor.index();
            is_touched.set(alpha, true);

            for in_edge in relaxation.edges_directed(*factor, Incoming) {
                let alpha_beta = in_edge.id().index();
                let beta = in_edge.source().index();
                attrs.edge_is_forward.set(alpha_beta, is_touched[beta]);
                is_touched.set(beta, true);
            }
        }

        // Compute weights
        let mut is_touched = BitVec::<usize, LocalBits>::repeat(false, relaxation.node_count());
        for factor in factor_sequence.sequence.iter() {
            let alpha = factor.index();
            is_touched.set(alpha, true);

            // Compute number of outgoing edges in forward and backward direction
            let mut weight_out_dir = vec![0, 0]; // forward, backward
            for out_edge in relaxation.edges_directed(*factor, Outgoing) {
                let beta = out_edge.target().index();
                weight_out_dir[is_touched[beta] as usize] += 1;
            }

            // Compute number of incoming edges in forward and backward direction and total number of incoming edges
            let mut weight_in_forward = 0;
            let mut weight_in_backward = 0;
            let mut weight_in_total = 0;
            for in_edge in relaxation.edges_directed(*factor, Incoming) {
                let alpha_beta = in_edge.id().index();
                weight_in_forward += attrs.edge_is_forward[alpha_beta] as usize;
                weight_in_backward += attrs.edge_is_backward[alpha_beta] as usize;
                weight_in_total += 1;
            }

            // Compute node weight in forward direction
            attrs.node_weight_forward[alpha] =
                max(weight_in_total - weight_in_forward, weight_in_forward) + weight_out_dir[0];
            if attrs.node_weight_forward[alpha] + weight_in_forward == 0 {
                attrs.node_weight_forward[alpha] = 1;
            }

            // Compute node weight in backward direction
            attrs.node_weight_backward[alpha] =
                max(weight_in_total - weight_in_backward, weight_in_backward) + weight_out_dir[1];
            if attrs.node_weight_backward[alpha] + weight_in_backward == 0 {
                attrs.node_weight_backward[alpha] = 1;
            }

            // Compute flag and node weight for lower bound updates
            let new_is_update_lb =
                attrs.node_is_update_lb[alpha] && attrs.node_weight_backward[alpha] > 0;
            attrs.node_is_update_lb.set(alpha, new_is_update_lb);

            attrs.node_weight_update_lb[alpha] =
                attrs.node_weight_backward[alpha] - weight_in_backward;
        }

        attrs
    }
}

// Stores information for the SRMP algorithm
pub struct SRMP<'a> {
    relaxation: &'a Relaxation<'a>,  // the relaxation graph
    messages: Messages,              // the messages sent along the edges of the relaxation graph
    factor_sequence: FactorSequence, // the sequence of factors considered in the forward and backward passes
    node_edge_attrs: NodeEdgeAttrs, // the attributes used in the computations in the forward and backward passes
    initial_lower_bound: f64,       // the initial lower bound
}

impl<'a> SRMP<'a> {
    // If compute_solution == true, initializes an empty solution
    // If compute_solution == false, returns None
    fn init_solution(&mut self, compute_solution: bool) -> Option<Solution> {
        match compute_solution {
            true => Some(Solution::new(self.relaxation.cfn())),
            false => None,
        }
    }

    // Extends a partial solution using the given factor
    fn compute_solution(&self, solution: &mut Solution, beta: NodeIndex<usize>) {
        let beta_origin = self.relaxation.factor_origin(beta);

        if solution.is_fully_labeled(self.relaxation.cfn().factor_variables(beta_origin)) {
            return;
        }

        let restricted_reparam =
            self.messages
                .compute_restricted_reparam(self.relaxation, beta, solution);

        restricted_reparam.update_solution_restricted_min(
            self.relaxation.cfn(),
            beta_origin,
            solution,
        );
    }

    // Performs the forward pass
    fn forward_pass(&mut self, solution: &mut Option<Solution>) {
        for factor in self.factor_sequence.sequence.iter() {
            // Line 4 of SRMP pseudocode: send messages along incoming "backward" edges
            for in_edge in self
                .relaxation
                .edges_directed(*factor, Incoming)
                .filter(|in_edge| self.node_edge_attrs.edge_is_backward[in_edge.id().index()])
            {
                self.messages.send(self.relaxation, in_edge);
            }

            // Compute solution if necessary
            if let Some(labeling) = solution {
                self.compute_solution(labeling, *factor);
            }

            // Line 5 of SRMP pseudocode: compute reparametrization
            let mut reparam = self.messages.compute_reparam(self.relaxation, *factor);

            // Line 6 of SRMP pseudocode: update messages along incoming "forward" edges
            reparam.mul_assign_scalar(
                1. / self.node_edge_attrs.node_weight_forward[factor.index()] as f64,
            );
            for in_edge in self
                .relaxation
                .edges_directed(*factor, Incoming)
                .filter(|in_edge| self.node_edge_attrs.edge_is_forward[in_edge.id().index()])
            {
                self.messages.sub_assign_reparam(&reparam, in_edge);
            }
        }
    }

    // Performs the backward pass
    fn backward_pass(&mut self, solution: &mut Option<Solution>) -> f64 {
        let mut lower_bound = self.initial_lower_bound;

        for factor in self.factor_sequence.sequence.iter().rev() {
            // Line 4 of SRMP pseudocode: send messages along incoming "forward" edges
            // (as well as edges that update the lower bound)
            for in_edge in self
                .relaxation
                .edges_directed(*factor, Incoming)
                .filter(|in_edge| {
                    self.node_edge_attrs.edge_is_forward[in_edge.id().index()] // todo: precompute
                    || self.node_edge_attrs.edge_is_update_lb[in_edge.id().index()]
                })
            {
                let delta = self.messages.send(self.relaxation, in_edge);
                if self.node_edge_attrs.edge_is_update_lb[in_edge.id().index()] {
                    lower_bound += delta;
                }
            }

            // Compute solution if necessary
            if let Some(labeling) = solution {
                self.compute_solution(labeling, *factor);
            }

            // Line 5 of SRMP pseudocode: compute reparametrization
            let mut reparam = self.messages.compute_reparam(self.relaxation, *factor);

            // Line 6 of SRMP pseudocode: update messages along incoming "backward" edges
            reparam.mul_assign_scalar(
                1. / self.node_edge_attrs.node_weight_backward[factor.index()] as f64,
            );
            for in_edge in self
                .relaxation
                .edges_directed(*factor, Incoming)
                .filter(|in_edge| self.node_edge_attrs.edge_is_backward[in_edge.id().index()])
            {
                self.messages.sub_assign_reparam(&reparam, in_edge);
            }

            // Update lower bound if necessary
            if self.node_edge_attrs.node_is_update_lb[factor.index()] {
                lower_bound += reparam.min()
                    * self.node_edge_attrs.node_weight_update_lb[factor.index()] as f64;
            }
        }

        lower_bound
    }
}

impl<'a> Solver<'a> for SRMP<'a> {
    fn init(relaxation: &'a Relaxation) -> Self {
        // Compute initial lower bound
        let mut initial_lower_bound = 0.;
        let mut messages = Messages::new(&relaxation);
        for node_index in relaxation.node_indices().filter(|node_index| {
            !relaxation.is_unary_factor(*node_index) && // question: why are these factors used for initial lower bound calculation?
            !relaxation.has_edges(*node_index, Incoming) &&
            !relaxation.has_edges(*node_index, Outgoing)
        }) {
            initial_lower_bound += messages.send_srmp_initial(relaxation, node_index);
        }

        // Find and sort all factors with at least one incoming edge
        // todo: different ordering procedures
        let factor_sequence = FactorSequence::new(&relaxation).sort();

        // Count edges of each category and compute SRMP weights
        let node_edge_attrs = NodeEdgeAttrs::new(&relaxation, &factor_sequence);

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
        let mut compute_solution = options.compute_solution_period() > 0;
        let mut current_lower_bound = 0.;

        let mut best_solution = None;
        let mut best_cost = 0.;
        let mut forward_cost;
        let mut backward_cost;

        loop {
            let previous_lower_bound = current_lower_bound;

            // Perform the forward pass
            let mut forward_solution = self.init_solution(compute_solution);
            self.forward_pass(&mut forward_solution);

            if let Some(solution) = forward_solution {
                // Log the forward solution
                forward_cost = solution.cost(self.relaxation.cfn());
                info!(
                    "Iteration {}. Elapsed time {:?}. Forward cost: {}. Forward solution {:#?}.",
                    iteration,
                    time_start.elapsed(),
                    forward_cost,
                    solution
                );

                // Update the best solution
                if best_solution.is_none() || best_cost > forward_cost {
                    best_cost = forward_cost;
                    best_solution = Some(solution);
                }
            }

            // Perform the backward pass
            let mut backward_solution = self.init_solution(compute_solution);
            current_lower_bound = self.backward_pass(&mut backward_solution);

            if let Some(solution) = backward_solution {
                // Log the backward solution
                backward_cost = solution.cost(self.relaxation.cfn());
                info!(
                    "Iteration {}. Elapsed time {:?}. Backward cost: {}. Backward solution {:#?}.",
                    iteration,
                    time_start.elapsed(),
                    backward_cost,
                    solution
                );

                // Update the best solution
                if best_solution.is_none() || best_cost > backward_cost {
                    best_cost = backward_cost;
                    best_solution = Some(solution);
                }
            }

            // Log the current status
            let elapsed_time = time_start.elapsed();
            info!(
                "Iteration {}. Elapsed time {:?}. Current lower bound {}.",
                iteration, elapsed_time, current_lower_bound
            );

            // Advance to next iteration
            iteration += 1;
            iter_solution -= compute_solution as usize * options.compute_solution_period();
            iter_solution += 1;
            compute_solution = (iter_solution == options.compute_solution_period())
                || (iteration + 1 == options.max_iterations());

            // Break if a stopping condition is satisfied
            if iteration >= options.max_iterations() {
                info!("Maximum number of iterations reached. Interrupting.");
                break;
            } else if elapsed_time >= options.time_max() {
                info!("Time limit reached. Interrupting.");
                break;
            } else if iteration > 1 && current_lower_bound < previous_lower_bound + options.eps() {
                info!("Lower bound increased less than by epsilon. Interrupting.");
                break;
            }
        }

        info!(
            "SRMP finished. Elapsed time {:?}. Best cost {}. Best solution {:?}.",
            time_start.elapsed(),
            best_cost,
            best_solution
        );

        self
    }
}
