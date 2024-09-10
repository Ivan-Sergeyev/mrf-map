#![allow(dead_code)]

use std::{cmp::max, time::Instant};

use bitvec::prelude::LocalBits;
use bitvec::vec::BitVec;
use petgraph::{
    graph::NodeIndex,
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};

use crate::{
    cfn::{
        cost_function_network::*,
        relaxation::{ConstructRelaxation, Messages, Relaxation},
    },
    factor_types::factor_trait::Factor,
};

use super::solver::{Solver, SolverOptions};

macro_rules! new_pass_iters {
    ($pass_iters:ident, $factor_sequence:expr) => {
        let $pass_iters: [Box<dyn Iterator<Item = _>>; 2] = [
            Box::new($factor_sequence.iter()), // forward pass iterator
            Box::new($factor_sequence.iter().rev()), // backward pass iterator
        ];
    };
}

pub struct NodeEdgeAttrs {
    edge_type: [BitVec<usize, LocalBits>; 2], // is_forward, is_backward
    edge_bound: BitVec<usize, LocalBits>,        // if lower bound should be updated using edge in backward pass
    node_bound: BitVec<usize, LocalBits>,        // if lower bound should be updated using node in backward pass
    node_weight: [Vec<f64>; 2],                  // weight_forward, weight_backward
    node_weight_lb: Vec<usize>,
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

    fn new<'a>(
        relaxation: &Relaxation,
        factor_sequence: &'a Vec<NodeIndex<usize>>,
        is_in_factor_sequence: &BitVec,
    ) -> NodeEdgeAttrs {
        // Initialize node and edge attribute storage
        let mut node_edge_attrs =
            NodeEdgeAttrs::new_zero(relaxation.node_count(), relaxation.edge_count());

        // Create forward and backward iterators over factor sequence
        new_pass_iters!(pass_iters, factor_sequence);

        // Perform forward pass and backward pass
        for (pass, pass_iter) in pass_iters.into_iter().enumerate() {
            // Keep track of which nodes were processed
            let mut is_touched =
                BitVec::<usize, LocalBits>::repeat(false, relaxation.node_count());

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
                    (pass == 0)
                        && (!is_touched[beta.index()] || relaxation.is_unary_factor(beta)),
                );
                let mut incoming_total: usize = 0;
                let mut incoming_before: usize = 0;
                let mut incoming_not_before: usize = 0;
                for alpha in relaxation.graph().neighbors_directed(beta, Incoming) {
                    let is_alpha_processed = is_touched[alpha.index()];
                    node_edge_attrs
                        .edge_bound
                        .set(alpha.index(), (pass == 0) && !is_alpha_processed);
                    incoming_before += is_alpha_processed as usize;
                    incoming_not_before += !is_alpha_processed as usize;
                    incoming_total += 1;
                }

                // Set weights
                let w = outgoing_before + max(incoming_before, incoming_not_before);
                node_edge_attrs.node_weight[pass][beta.index()] = 1. / w as f64;
                node_edge_attrs.node_weight_lb[beta.index()] = (pass == 1) as usize * (w - incoming_total);
                for alpha_beta in relaxation.graph().edges_directed(beta, Incoming) {
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
    cfn: &'a GeneralCFN,
    relaxation: Relaxation,
    messages: Messages,
    factor_sequence: Vec<NodeIndex<usize>>,
    node_edge_attrs: NodeEdgeAttrs,
    initial_lower_bound: f64,
}

macro_rules! iter_messages {
    ($self:ident, $beta:ident, $edge_direction:expr, $pass_direction:expr, $edge:ident, $or_condition:expr) => {
        $self.relaxation.graph().edges_directed($beta, $edge_direction)
        .filter(|$edge| {$self.node_edge_attrs.edge_type[$pass_direction][$edge.id().index()]} || $or_condition)
    };
}

impl<'a> SRMP<'a> {
    fn new_factor_sequence(
        relaxation: &Relaxation,
    ) -> (Vec<NodeIndex<usize>>, BitVec<usize, LocalBits>) {
        let mut factor_sequence = Vec::new();
        let mut is_in_factor_sequence = BitVec::repeat(false, relaxation.node_count());
        for node_index in relaxation
            .graph()
            .node_indices()
            .filter(|node_index| relaxation.has_edges(*node_index, Incoming))
        {
            // If there are incoming edges, then add this factor to the factor sequence
            factor_sequence.push(node_index);
            is_in_factor_sequence.set(node_index.index(), true);
        }
        (factor_sequence, is_in_factor_sequence)
    }

    fn forward_pass(&mut self) {
        let pass_direction = 0;

        for &beta in self.factor_sequence.iter() {
            // Update messages along I_\beta^+ (line 4 of SRMP pseudocode in the paper)
            let incoming_opposite_pass =
                iter_messages!(self, beta, Incoming, 1 - pass_direction, alpha_beta, false);
            for alpha_beta in incoming_opposite_pass {
                self.messages
                    .send_message(self.relaxation.graph(), self.cfn, alpha_beta);
            }

            // Compute reparametrization of beta (line 5 of SRMP pseudocode)
            let mut theta_beta =
                self.messages
                    .compute_reparametrization(self.relaxation.graph(), self.cfn, beta);

            // Update messages along I_\beta^- (line 6 of SRMP pseudocode)
            theta_beta.mul_assign(self.node_edge_attrs.node_weight[0][beta.index()]);
            let incoming_same_as_pass =
                iter_messages!(self, beta, Incoming, pass_direction, alpha_beta, false);
            for alpha_beta in incoming_same_as_pass {
                self.messages
                    .sub_assign_reparametrization(&theta_beta, alpha_beta);
            }
        }
    }

    fn backward_pass(&mut self) -> f64 {
        let mut lower_bound = self.initial_lower_bound;
        let pass_direction = 1;

        for &beta in self.factor_sequence.iter().rev() {
            // Update messages along I_\beta^- (line 4 of SRMP pseudocode)
            let incoming_opposite = iter_messages!(
                self,
                beta,
                Incoming,
                1 - pass_direction,
                alpha_beta,
                self.node_edge_attrs.edge_bound[alpha_beta.id().index()]
            );
            for alpha_beta in incoming_opposite {
                let delta =
                    self.messages
                        .send_message(self.relaxation.graph(), self.cfn, alpha_beta);
                lower_bound += (self.node_edge_attrs.edge_bound[alpha_beta.id().index()] as u8
                    as f64)
                    * delta;
            }

            // Compute reparametrization of beta (line 5 of SRMP pseudocode)
            let mut theta_beta =
                self.messages
                    .compute_reparametrization(self.relaxation.graph(), self.cfn, beta);

            // Update messages along I_\beta^+ (line 6 of SRMP pseudocode in the paper)
            theta_beta.mul_assign(self.node_edge_attrs.node_weight[pass_direction][beta.index()]);
            let incoming_same =
                iter_messages!(self, beta, Incoming, pass_direction, alpha_beta, false);
            for alpha_beta in incoming_same {
                self.messages
                    .sub_assign_reparametrization(&theta_beta, alpha_beta);
            }

            // Take beta into account in lower bound
            let beta_weight_lb = self.node_edge_attrs.node_weight_lb[beta.index()];
            if self.node_edge_attrs.node_bound[beta.index()] && beta_weight_lb > 0 {
                lower_bound += theta_beta.max() * beta_weight_lb as f64;
            }
        }

        lower_bound
    }
}

impl<'a> Solver<'a> for SRMP<'a> {
    fn init(cfn: &'a GeneralCFN) -> Self {
        // Construct relaxation graph
        let relaxation = Relaxation::new(cfn);

        // todo: prune relaxation?
        // - suppose there is factor aplha with a single child beta
        // - then we can reparametrize theta to get min_{x_a ~ x_b} theta_a(x_a) = 0 for all x_b and then remove alpha
        // - this won't affect the relaxation (claim from the SRMP paper)

        // Find all factors with at least one incoming edge
        let (mut factor_sequence, is_in_factor_sequence) = SRMP::new_factor_sequence(&relaxation);

        // Order factor sequence
        // todo: different ordering procedures
        factor_sequence.sort_unstable();

        // Count edges of each category and compute SRMP weights
        let node_edge_attrs =
            NodeEdgeAttrs::new(&relaxation, &factor_sequence, &is_in_factor_sequence);

        // Init zero messages
        let mut messages = Messages::new(&cfn, &relaxation.graph());

        // Compute initial lower bound
        let mut initial_lower_bound = 0.;
        for node_index in relaxation
            .graph()
            .node_indices()
            .filter(|node_index| !relaxation.has_edges(*node_index, Outgoing))
        {
            initial_lower_bound +=
                messages.send_srmp_init_message(relaxation.graph(), cfn, node_index);
        }

        // Form and return SRMP struct
        SRMP {
            cfn,
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

        loop {
            let previous_lower_bound = self.initial_lower_bound;
            if compute_solution {
                todo!();  // critical todo: compute solution
                self.forward_pass();
                current_lower_bound = self.backward_pass();
            } else {
                self.forward_pass();
                current_lower_bound = self.backward_pass();
            }

            iteration += 1;

            iter_solution -= compute_solution as usize * options.compute_solution_period();
            iter_solution += 1;
            compute_solution = (iter_solution == options.compute_solution_period())
                || (iteration + 1 == options.max_iterations());

            let elapsed_time = time_start.elapsed();
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
