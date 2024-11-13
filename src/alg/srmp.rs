#![allow(dead_code)]

use std::{cmp::max, time::Instant};

use bitvec::{order::LocalBits, vec::BitVec};
use log::{debug, info};
use petgraph::{
    graph::{EdgeReference, NodeIndex},
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};

use crate::{
    cfn::{factor_sequence::FactorSequence, relaxation::Relaxation, solution::Solution},
    messages::{
        message_nd::{AlignmentIndexing, MessageND},
        message_trait::Message,
    },
    CostFunctionNetwork,
};

use super::solver::{Solver, SolverOptions};

type PassIterator<'a> = Box<dyn Iterator<Item = &'a NodeIndex<usize>> + 'a>;

// Stores the attributes used in the computations in the forward and backward passes
#[derive(Debug)]
pub struct NodeEdgeAttrs {
    edge_is_forward: BitVec<usize, LocalBits>, // is_fw from cpp // todo: better desc
    edge_is_backward: BitVec<usize, LocalBits>, // is_bw from cpp // todo: better desc
    edge_is_update_lb: BitVec<usize, LocalBits>, // if the lower bound is updated via the edge in the backward pass
    node_is_update_lb: BitVec<usize, LocalBits>, // if the lower bound is updated via the node in the backward pass
    node_omega_forward: Vec<f64>, // the scaling factor for the reparametrization update in the forward pass
    node_omega_backward: Vec<f64>, // the scaling factor for the reparametrization update in the backward pass
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
            node_omega_forward: vec![0.; num_nodes],
            node_omega_backward: vec![0.; num_nodes],
            node_weight_update_lb: vec![0; num_nodes],
        }
    }

    // Computes attributes based on the given relaxation and factor sequence
    fn new<'a>(relaxation: &Relaxation, factor_sequence: &'a FactorSequence) -> NodeEdgeAttrs {
        // Initialize node and edge attribute storage
        let mut attrs = NodeEdgeAttrs::zero(relaxation.node_count(), relaxation.edge_count());

        // Label backward edges
        let mut is_touched = BitVec::<usize, LocalBits>::repeat(false, relaxation.node_count());
        for factor in factor_sequence.iter() {
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
        for factor in factor_sequence.iter().rev() {
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
        for factor in factor_sequence.iter() {
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
            let mut alpha_weight_forward =
                max(weight_in_total - weight_in_forward, weight_in_forward) + weight_out_dir[0];
            if alpha_weight_forward + weight_in_forward == 0 {
                alpha_weight_forward = 1;
            }

            // Compute node weight in backward direction
            let mut alpha_weight_backward =
                max(weight_in_total - weight_in_backward, weight_in_backward) + weight_out_dir[1];
            if alpha_weight_backward + weight_in_backward == 0 {
                alpha_weight_backward = 1;
            }

            // Compute scaling factors for reparametrization updates
            attrs.node_omega_forward[alpha] = 1. / alpha_weight_forward as f64;
            attrs.node_omega_backward[alpha] = 1. / alpha_weight_backward as f64;

            // Compute flag and node weight for lower bound updates
            let new_is_update_lb = attrs.node_is_update_lb[alpha] && alpha_weight_backward > 0;
            attrs.node_is_update_lb.set(alpha, new_is_update_lb);
            attrs.node_weight_update_lb[alpha] = alpha_weight_backward - weight_in_backward;
        }

        attrs
    }
}

// Stores messages and facilitates computations on groups of messages, including reparametrizations
pub struct SRMPMessages<'a> {
    cfn: &'a CostFunctionNetwork,
    relaxation: &'a Relaxation<'a>,
    alignment_indexing: Vec<AlignmentIndexing>, // todo: make generic
    messages: Vec<MessageND>,                   // todo: make generic
}

impl<'a> SRMPMessages<'a> {
    // Creates new zero messages for every edge in a given relaxation
    fn new(cfn: &'a CostFunctionNetwork, relaxation: &'a Relaxation) -> Self {
        let mut messages = Vec::with_capacity(relaxation.edge_count());
        let mut alignment_indexing = Vec::with_capacity(relaxation.edge_count());
        for edge in relaxation.edge_references() {
            let alpha = relaxation.factor_origin(edge.source());
            let beta = relaxation.factor_origin(edge.target());
            messages.push(MessageND::zero(&cfn, beta));
            alignment_indexing.push(AlignmentIndexing::new(&cfn, &alpha, &beta));
        }

        SRMPMessages {
            cfn,
            relaxation,
            alignment_indexing,
            messages,
        }
    }

    // Creates a new reparametrization and initializes it with data from a given factor
    fn init_reparam(&self, factor: NodeIndex<usize>) -> MessageND {
        MessageND::clone_factor(self.cfn, self.relaxation.factor_origin(factor))
        // todo: make generic
    }

    // Adds messages along all incoming edges to a given reparametrization
    fn add_all_incoming_messages(&self, reparam: &mut MessageND, factor: NodeIndex<usize>) {
        for in_edge in self.relaxation.edges_directed(factor, Incoming) {
            reparam.add_assign_incoming(&self.messages[in_edge.id().index()]);
        }
    }

    // Subtracts messages along all incoming edges to a given reparametrization
    fn sub_all_outgoing_messages(&self, reparam: &mut MessageND, factor: NodeIndex<usize>) {
        for out_edge in self.relaxation.edges_directed(factor, Outgoing) {
            reparam.sub_assign_outgoing(
                &self.messages[out_edge.id().index()],
                &self.alignment_indexing[out_edge.id().index()],
            );
        }
    }

    // Subtracts messages along all outgoing edges excep the given one to a given reparametrization
    fn sub_all_other_outgoing_messages(
        &self,
        reparam: &mut MessageND,
        factor: NodeIndex<usize>,
        edge: EdgeReference<'_, (), usize>,
    ) {
        if true {
            for out_edge in self
                .relaxation
                .edges_directed(factor, Outgoing)
                .filter(|out_edge| out_edge.id().index() != edge.id().index())
            {
                reparam.sub_assign_outgoing(
                    &self.messages[out_edge.id().index()],
                    &self.alignment_indexing[out_edge.id().index()],
                );
            }
        } else {
            // Alternative implementation of subtract_all_other_outgoing_messages()
            // - removed nested if inside for loop, replaced with compensating addition after the loop
            // - may be faster due to avoiding if-jumps inside for-loop and vectorization of message addition
            // todo: bench performance
            self.sub_all_outgoing_messages(reparam, factor);
            reparam.add_assign_outgoing(
                &self.messages[edge.id().index()],
                &self.alignment_indexing[edge.id().index()],
            );
        }
    }

    // Updates the message corresponding to a given edge by computing the minimum from equation (17) in the SRMP paper
    // over a given reparametrization, then renormalizes the message so that its smallest entry becomes 0
    fn update_and_normalize(
        &mut self,
        reparam: &MessageND,
        edge: EdgeReference<'_, (), usize>,
    ) -> f64 {
        let delta = self.messages[edge.id().index()]
            .set_to_reparam_min(&reparam, &self.alignment_indexing[edge.id().index()]);
        self.messages[edge.id().index()].add_assign_scalar(-delta);
        delta
    }

    // Updates the message corresponding to a given edge by sending messages,
    // i.e., performs a computation from equation (17) in the SRMP paper
    fn send(&mut self, edge: EdgeReference<'_, (), usize>) -> f64 {
        debug!(
            "In send() for edge {} from {} to {}",
            edge.id().index(),
            edge.source().index(),
            edge.target().index()
        );

        let alpha = edge.source();
        let mut reparam_alpha = self.init_reparam(alpha);
        self.add_all_incoming_messages(&mut reparam_alpha, alpha);
        self.sub_all_other_outgoing_messages(&mut reparam_alpha, alpha, edge);
        self.update_and_normalize(&reparam_alpha, edge)
    }

    // Computes a reparametrization for a given factor by sending messages to and from it,
    // i.e., performs a computation from line 5 in the SRMP paper
    fn compute_reparam(&mut self, factor: NodeIndex<usize>) -> MessageND {
        debug!("In compute_reparam() for factor {}", factor.index());

        let mut reparam = self.init_reparam(factor);
        self.add_all_incoming_messages(&mut reparam, factor);
        self.sub_all_outgoing_messages(&mut reparam, factor);
        reparam
    }

    // Subtracts a given reparametrization from the message corresponding to a given edge
    fn sub_assign_reparam(&mut self, reparam: &MessageND, edge: EdgeReference<'_, (), usize>) {
        debug!(
            "In sub_assign_reparam() for edge {} from {} to {}",
            edge.id().index(),
            edge.source().index(),
            edge.target().index()
        );

        self.messages[edge.id().index()].sub_assign_incoming(reparam);
    }

    // Computes the initial reparametrization at the start of the SRMP algorithm for a given factor
    fn send_srmp_initial(&mut self, factor: NodeIndex<usize>) -> f64 {
        debug!("In send_srmp_initial() for factor {}", factor.index());

        let mut theta = self.init_reparam(factor);
        self.add_all_incoming_messages(&mut theta, factor);
        *theta.min()
    }

    // Updates the message corresponding to a given edge by sending messages "restricted" by a given solution.
    // In other words, performs a computation similar to equation (17) in the SRMP paper,
    // but minimization is performed only over labelings consistent with the given solution.
    // Refer to the "Extracting primal solution" subsection in the SRMP section for more details.
    fn send_restricted(
        &self,
        edge: EdgeReference<'_, (), usize>,
        solution: &Solution,
    ) -> MessageND {
        debug!(
            "In send_restricted() for edge {} from {} to {}",
            edge.id().index(),
            edge.source().index(),
            edge.target().index()
        );

        let alpha = edge.source();
        let mut reparam_alpha = self.init_reparam(alpha);
        self.add_all_incoming_messages(&mut reparam_alpha, alpha);
        self.sub_all_other_outgoing_messages(&mut reparam_alpha, alpha, edge);
        debug!(
            "reparam_alpha before taking restricted min: {:?} alpha {} beta {}",
            reparam_alpha,
            alpha.index(),
            edge.target().index()
        );
        let restricted_min = reparam_alpha.restricted_min(
            self.cfn,
            solution,
            self.relaxation.factor_origin(alpha),
            self.relaxation.factor_origin(edge.target()),
        );
        debug!(
            "reparam_alpha after taking restricted min: {:?}",
            restricted_min
        );
        restricted_min
    }

    // Computes "restricted" reparametrization of a given factor by sending messages "restricted" by a given solution.
    // Refer to the "Extracting primal solution" subsection in the SRMP section for more details.
    fn compute_restricted_reparam(
        &self,
        factor: NodeIndex<usize>,
        solution: &Solution,
    ) -> MessageND {
        debug!(
            "In compute_restricted_reparam() for factor {}",
            factor.index()
        );

        let mut reparam_beta = self.init_reparam(factor);
        self.sub_all_outgoing_messages(&mut reparam_beta, factor);
        for in_edge in self.relaxation.edges_directed(factor, Incoming) {
            let alpha = self.relaxation.factor_origin(in_edge.source());
            let num_labeled = solution.num_labeled(&self.cfn.factor_variables(alpha));
            if num_labeled > 0 && num_labeled < self.cfn.arity(alpha) {
                let restrected_message = self.send_restricted(in_edge, solution);
                reparam_beta.add_assign_incoming(&restrected_message);
            } else {
                reparam_beta.add_assign_incoming(&self.messages[in_edge.id().index()]);
            }
        }
        reparam_beta
    }

    fn get_initial_lower_bound(&mut self) -> f64 {
        let mut initial_lower_bound = 0.;
        for node_index in self.relaxation.node_indices().filter(|node_index| {
            !self.relaxation.is_unary_factor(*node_index) && // question: why are these factors used for initial lower bound calculation?
            !self.relaxation.has_edges(*node_index, Incoming) &&
            !self.relaxation.has_edges(*node_index, Outgoing)
        }) {
            initial_lower_bound += self.send_srmp_initial(node_index);
        }
        initial_lower_bound
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cfn::relaxation::ConstructRelaxation,
        factors::{factor_trait::Factor, factor_type::FactorType, function_table::FunctionTable},
        CostFunctionNetwork,
    };

    use super::*;

    fn construct_cfn_example_1() -> CostFunctionNetwork {
        let mut cfn = CostFunctionNetwork::from_domain_sizes(&vec![3, 4, 5], false, 3);
        cfn.add_factor(FactorType::FunctionTable(FunctionTable::new(
            &cfn,
            vec![0],
            vec![1., 2., 3.],
        )));
        cfn.add_factor(FactorType::FunctionTable(FunctionTable::new(
            &cfn,
            vec![2],
            vec![11., 12., 13., 14., 15.],
        )));
        cfn.add_factor(FactorType::FunctionTable(FunctionTable::new(
            &cfn,
            vec![0, 1],
            vec![4.; 3 * 4],
        )));
        cfn.add_factor(FactorType::FunctionTable(FunctionTable::new(
            &cfn,
            vec![0, 2],
            vec![5.; 3 * 5],
        )));
        cfn.add_factor(FactorType::FunctionTable(FunctionTable::new(
            &cfn,
            vec![1, 2],
            vec![6.; 4 * 5],
        )));
        cfn.add_factor(FactorType::FunctionTable(FunctionTable::new(
            &cfn,
            vec![0, 1, 2],
            vec![7.; 3 * 4 * 5],
        )));
        cfn
    }

    #[test]
    fn new() {
        let cfn = construct_cfn_example_1();
        let relaxation = Relaxation::new(&cfn);
        let messages = SRMPMessages::new(&cfn, &relaxation);

        for (index, edge) in relaxation.edge_references().enumerate() {
            let message_vec: Vec<f64> = messages.messages[index]
                .iter()
                .map(|value| *value)
                .collect();

            let factor_origin = relaxation.factor_origin(edge.target());
            let max_function_table_size = cfn.function_table_len(factor_origin);

            assert_eq!(message_vec, vec![0.; max_function_table_size]);
        }
    }

    #[test]
    fn init_reparametrization() {
        let cfn = construct_cfn_example_1();
        let relaxation = Relaxation::new(&cfn);
        let messages = SRMPMessages::new(&cfn, &relaxation);

        for factor in relaxation.node_indices() {
            let reparam = messages.init_reparam(factor);
            let reparam_vec: Vec<f64> = reparam.iter().map(|val| *val).collect();

            let factor_origin = relaxation.factor_origin(factor);
            let max_function_table_size = cfn.function_table_len(factor_origin);
            let factor_type = cfn.get_factor(factor_origin);
            let factor_vec: Vec<f64> = match factor_type {
                Some(factor_type) => factor_type.clone_function_table(),
                None => vec![0.; max_function_table_size],
            };

            assert_eq!(reparam_vec, factor_vec);
        }
    }

    #[test]
    fn add_all_incoming_messages() {
        let cfn = construct_cfn_example_1();
        let relaxation = Relaxation::new(&cfn);
        let mut messages = SRMPMessages::new(&cfn, &relaxation);

        for message in messages.messages.iter_mut() {
            message.add_assign_scalar(1.);
        }

        for factor in relaxation.node_indices() {
            let mut reparam = messages.init_reparam(factor);
            let before_vec: Vec<f64> = reparam.iter().map(|value| *value).collect();
            messages.add_all_incoming_messages(&mut reparam, factor);

            let diff: Vec<f64> = reparam
                .iter()
                .zip(before_vec.iter())
                .map(|(after, before)| after - before)
                .collect();

            let expected_value = relaxation.edges_directed(factor, Incoming).count() as f64;
            let expected_size = cfn.function_table_len(relaxation.factor_origin(factor));
            assert_eq!(diff, vec![expected_value; expected_size]);
        }
    }

    // todo: add tests for remaining functions, use the stub below

    // #[test]
    // fn sub_all_outgoing_messages() {
    //     todo!();
    // }

    // #[test]
    // fn sub_all_other_outgoing_messages() {
    //     todo!();
    // }

    // #[test]
    // fn subtract_all_other_outgoing_messages_alt() {
    //     todo!();
    // }

    // #[test]
    // fn update_and_normalize() {
    //     todo!();
    // }

    // #[test]
    // fn send() {
    //     todo!();
    // }

    // #[test]
    // fn compute_reparam() {
    //     todo!();
    // }

    // #[test]
    // fn sub_assign_reparam() {
    //     todo!();
    // }

    // #[test]
    // fn send_srmp_initial() {
    //     todo!();
    // }

    // #[test]
    // fn send_restricted() {
    //     todo!();
    // }

    // #[test]
    // fn compute_restricted_reparam() {
    //     todo!();
    // }
}

// Stores information for the SRMP algorithm
pub struct SRMP<'a> {
    cfn: &'a CostFunctionNetwork,    // the cost function network
    relaxation: &'a Relaxation<'a>,  // the relaxation graph
    node_edge_attrs: NodeEdgeAttrs, // the attributes used in the computations in the forward and backward passes
    messages: SRMPMessages<'a>,     // the messages sent along the edges of the relaxation graph
    factor_sequence: FactorSequence, // the sequence of factors considered in the forward and backward passes
    initial_lower_bound: f64,        // the initial lower bound
}

impl<'a> SRMP<'a> {
    // If compute_solution == true, initializes an empty solution
    // If compute_solution == false, returns None
    fn init_solution(&mut self, compute_solution: bool) -> Option<Solution> {
        match compute_solution {
            true => Some(Solution::new(self.cfn)),
            false => None,
        }
    }

    // Extends a partial solution using the given factor
    fn compute_solution(&self, solution: &mut Solution, beta: NodeIndex<usize>) {
        let beta_origin = self.relaxation.factor_origin(beta);

        if solution.is_fully_labeled(&self.cfn.factor_variables(beta_origin)) {
            return;
        }

        let restricted_reparam = self.messages.compute_restricted_reparam(beta, solution);

        restricted_reparam.update_solution_restricted_min(self.cfn, beta_origin, solution);
    }

    // Performs the forward pass
    fn forward_pass(&mut self, solution: &mut Option<Solution>) {
        for factor in self.factor_sequence.iter() {
            // Line 4 of SRMP pseudocode: send messages along incoming "backward" edges
            for in_edge in self
                .relaxation
                .edges_directed(*factor, Incoming)
                .filter(|in_edge| self.node_edge_attrs.edge_is_backward[in_edge.id().index()])
            {
                self.messages.send(in_edge);
            }

            // Compute solution if necessary
            if let Some(labeling) = solution {
                self.compute_solution(labeling, *factor);
            }

            // Line 5 of SRMP pseudocode: compute reparametrization
            let mut reparam = self.messages.compute_reparam(*factor);

            // Line 6 of SRMP pseudocode: update messages along incoming "forward" edges
            reparam.mul_assign_scalar(self.node_edge_attrs.node_omega_forward[factor.index()]);
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

        for factor in self.factor_sequence.iter().rev() {
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
                let delta = self.messages.send(in_edge);
                if self.node_edge_attrs.edge_is_update_lb[in_edge.id().index()] {
                    lower_bound += delta;
                }
            }

            // Compute solution if necessary
            if let Some(labeling) = solution {
                self.compute_solution(labeling, *factor);
            }

            // Line 5 of SRMP pseudocode: compute reparametrization
            let mut reparam = self.messages.compute_reparam(*factor);

            // Line 6 of SRMP pseudocode: update messages along incoming "backward" edges
            reparam.mul_assign_scalar(self.node_edge_attrs.node_omega_backward[factor.index()]);
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
    fn init(cfn: &'a CostFunctionNetwork, relaxation: &'a Relaxation) -> Self {
        // Find and sort all factors with at least one incoming edge
        // todo: different ordering procedures
        let factor_sequence = FactorSequence::new(&relaxation).sort();

        // Count edges of each category and compute SRMP weights
        let node_edge_attrs = NodeEdgeAttrs::new(&relaxation, &factor_sequence);

        // Initialize messages
        let mut messages = SRMPMessages::new(cfn, &relaxation);

        // Compute initial lower bound
        let initial_lower_bound = messages.get_initial_lower_bound();

        // Form and return SRMP struct
        SRMP {
            cfn,
            relaxation,
            node_edge_attrs,
            messages,
            factor_sequence,
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
                forward_cost = solution.cost(self.cfn);
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
                backward_cost = solution.cost(self.cfn);
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
