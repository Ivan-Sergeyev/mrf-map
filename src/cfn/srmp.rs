#![allow(dead_code)]

use std::{cmp::max, time::Instant};

use bitvec::prelude::LocalBits;
use bitvec::vec::BitVec;
use log::{debug, info};
use petgraph::{
    graph::NodeIndex,
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};

use crate::{
    cfn::{cost_function_network::*, relaxation::Relaxation},
    message::{message_trait::Message, messages::Messages},
};

use super::solver::{Solver, SolverOptions};

type PassIterator<'a> = Box<dyn Iterator<Item = &'a NodeIndex<usize>> + 'a>;

struct FactorSequence {
    sequence: Vec<NodeIndex<usize>>,
}

impl FactorSequence {
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

    fn pass_iter(&self, is_forward_pass: bool) -> PassIterator {
        match is_forward_pass {
            true => Box::new(self.sequence.iter()),
            false => Box::new(self.sequence.iter().rev()),
        }
    }

    fn new_pass_iters(&self) -> [PassIterator; 2] {
        [self.pass_iter(true), self.pass_iter(false)]
    }

    fn sort(mut self) -> Self {
        self.sequence.sort_unstable();
        self
    }
}

#[derive(Debug)]
pub struct NodeEdgeAttrs {
    edge_is_forward: BitVec<usize, LocalBits>,   // todo: desc
    edge_is_backward: BitVec<usize, LocalBits>,  // todo: desc
    edge_is_update_lb: BitVec<usize, LocalBits>, // if lower bound is updated using edge in backward pass
    node_is_update_lb: BitVec<usize, LocalBits>, // if lower bound is updated using node in backward pass
    node_weight_forward: Vec<usize>,             // todo: desc
    node_weight_backward: Vec<usize>,            // todo: desc
    node_weight_update_lb: Vec<usize>,           // weight for updating lower bound in backward pass
}

impl NodeEdgeAttrs {
    fn new_zero(num_nodes: usize, num_edges: usize) -> Self {
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

    fn new<'a>(relaxation: &Relaxation, factor_sequence: &'a FactorSequence) -> NodeEdgeAttrs {
        debug!("In NodeEdgeAttrs::new()");

        // Initialize node and edge attribute storage
        let mut attrs = NodeEdgeAttrs::new_zero(relaxation.node_count(), relaxation.edge_count());

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
            let mut weight_out_dir = vec![0, 0];
            for out_edge in relaxation.edges_directed(*factor, Outgoing) {
                let beta = out_edge.target().index();
                weight_out_dir[is_touched[beta] as usize] += 1;
            }

            // Compute number of incoming edges in forward and backward direction and total number of incoming edges
            let mut weight_in_dir = vec![0, 0];
            let mut weight_in_total = 0;
            for in_edge in relaxation.edges_directed(*factor, Incoming) {
                let alpha_beta = in_edge.id().index();
                weight_in_dir[0] += attrs.edge_is_forward[alpha_beta] as usize;
                weight_in_dir[1] += attrs.edge_is_backward[alpha_beta] as usize;
                weight_in_total += 1;
            }

            // Compute node weights in forward and backward direction, and node weights used in lower bound updates
            attrs.node_weight_forward[alpha] =
                max(weight_in_total - weight_in_dir[0], weight_in_dir[0]) + weight_out_dir[0];
            if attrs.node_weight_forward[alpha] + weight_in_dir[0] == 0 {
                attrs.node_weight_forward[alpha] = 1;
            }

            attrs.node_weight_backward[alpha] =
                max(weight_in_total - weight_in_dir[1], weight_in_dir[1]) + weight_out_dir[1];
            if attrs.node_weight_backward[alpha] + weight_in_dir[1] == 0 {
                attrs.node_weight_backward[alpha] = 1;
            }

            let new_is_update_lb =
                attrs.node_is_update_lb[alpha] && attrs.node_weight_backward[alpha] > 0;
            attrs.node_is_update_lb.set(alpha, new_is_update_lb);

            attrs.node_weight_update_lb[alpha] =
                attrs.node_weight_backward[alpha] - weight_in_dir[1];
        }

        attrs
    }
}

pub struct SRMP<'a> {
    relaxation: &'a Relaxation<'a>,
    messages: Messages,
    factor_sequence: FactorSequence,
    node_edge_attrs: NodeEdgeAttrs,
    initial_lower_bound: f64,
}

impl<'a> SRMP<'a> {
    fn compute_solution(&self, solution: &mut Solution, beta: NodeIndex<usize>) {
        debug!(
            "In compute_solution() with solution {} beta {}",
            solution,
            beta.index()
        );

        let beta_origin = self.relaxation.factor_origin(beta);
        if solution.is_fully_labeled(self.relaxation.cfn().factor_variables(beta_origin)) {
            return;
        }

        let theta_star = self
            .messages
            .compute_restricted_reparam(self.relaxation, beta, solution);
        debug!("theta_star: {:?}", theta_star);

        // Choose a labeling with the smallest `theta_star` cost
        match beta_origin {
            FactorOrigin::Variable(variable_index) => {
                solution[*variable_index] = Some(theta_star.index_min())
            }
            FactorOrigin::NonUnaryFactor(_factor_index) => {
                theta_star.update_solution_restricted_minimum(
                    self.relaxation.cfn(),
                    beta_origin,
                    solution,
                );
            }
        }
    }

    fn forward_pass(&mut self, solution: &mut Option<Solution>) {
        debug!("In forward_pass() with solution {:?}", solution);

        for factor in self.factor_sequence.sequence.iter() {
            debug!("Considering factor {}", factor.index());

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

    fn backward_pass(&mut self, solution: &mut Option<Solution>) -> f64 {
        debug!("In backward_pass() with solution {:?}", solution);

        let mut lower_bound = self.initial_lower_bound;

        for factor in self.factor_sequence.sequence.iter().rev() {
            debug!("beta {}", factor.index());

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
                debug!(
                    "delta {} lower bound {} edge bound {}",
                    delta,
                    lower_bound,
                    self.node_edge_attrs.edge_is_update_lb[in_edge.id().index()]
                );
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

        debug!("lower bound {}", lower_bound);
        lower_bound
    }

    fn init_solution(&mut self, compute_solution: bool) -> Option<Solution> {
        match compute_solution {
            true => Some(Solution::new(self.relaxation.cfn())),
            false => None,
        }
    }
}

impl<'a> Solver<'a> for SRMP<'a> {
    fn init(relaxation: &'a Relaxation) -> Self {
        // Compute initial lower bound

        // ( initial lower bound )
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
        let mut compute_solution = true;
        let mut current_lower_bound = 0.;

        let mut best_solution = None;
        let mut best_cost = 0.;
        let mut forward_cost;
        let mut backward_cost;

        loop {
            let previous_lower_bound = current_lower_bound;
            debug!("Previous lower bound {}", previous_lower_bound);

            // Forward pass
            let mut forward_solution = self.init_solution(compute_solution);
            self.forward_pass(&mut forward_solution);
            if let Some(solution) = forward_solution {
                forward_cost = self.relaxation.cfn().cost(&solution);
                info!(
                    "Iteration {}. Elapsed time {:?}. Forward cost: {}. Forward solution {:#?}.",
                    iteration,
                    time_start.elapsed(),
                    forward_cost,
                    solution
                );
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
                info!(
                    "Iteration {}. Elapsed time {:?}. Backward cost: {}. Backward solution {:#?}.",
                    iteration,
                    time_start.elapsed(),
                    backward_cost,
                    solution
                );
                if best_solution.is_none() || best_cost > backward_cost {
                    best_cost = backward_cost;
                    best_solution = Some(solution);
                }
            }

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

            // Break if stopping a condition is satisfied
            if iteration >= options.max_iterations() {
                info!("Maximum number of iterations reached. Interrupting.");
                break;
            }

            if elapsed_time >= options.time_max() {
                info!("Time limit reached. Interrupting.");
                break;
            }

            if iteration > 1 && current_lower_bound < previous_lower_bound + options.eps() {
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
