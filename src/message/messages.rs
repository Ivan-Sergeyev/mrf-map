#![allow(dead_code)]

use log::debug;
use petgraph::{
    graph::{EdgeReference, NodeIndex},
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};

use crate::cfn::{relaxation::Relaxation, solution::Solution};

use super::{
    message_general::{GeneralAlignment, GeneralMessage},
    message_trait::Message,
};

// Stores messages and facilitates computations on groups of messages, including reparametrizations
pub struct Messages {
    messages: Vec<GeneralMessage>, // todo: use MessageType
}

impl Messages {
    // Creates new zero messages for every edge in a given relaxation
    pub fn new(relaxation: &Relaxation) -> Self {
        let mut messages = Vec::with_capacity(relaxation.edge_count());
        for edge in relaxation.edge_references() {
            messages.push(
                relaxation
                    .cfn()
                    .new_zero_message(relaxation.factor_origin(edge.target())),
            );
        }
        Messages { messages }
    }

    // Creates a new reparametrization and initializes it with data from a given factor
    fn init_reparam(&self, relaxation: &Relaxation, factor: NodeIndex<usize>) -> GeneralMessage {
        relaxation
            .cfn()
            .new_message_clone(relaxation.factor_origin(factor))
    }

    // Adds messages along all incoming edges to a given reparametrization
    fn add_all_incoming_messages(
        &self,
        relaxation: &Relaxation,
        reparam: &mut GeneralMessage,
        factor: NodeIndex<usize>,
    ) {
        for in_edge in relaxation.edges_directed(factor, Incoming) {
            reparam.add_assign_incoming(&self.messages[in_edge.id().index()]);
        }
    }

    // Subtracts messages along all incoming edges to a given reparametrization
    fn sub_all_outgoing_messages(
        &self,
        relaxation: &Relaxation,
        reparam: &mut GeneralMessage,
        factor: NodeIndex<usize>,
    ) {
        for out_edge in relaxation.edges_directed(factor, Outgoing) {
            reparam.sub_assign_outgoing(&self.messages[out_edge.id().index()], out_edge.weight());
        }
    }

    // Subtracts messages along all outgoing edges excep the given one to a given reparametrization
    fn sub_all_other_outgoing_messages(
        &self,
        relaxation: &Relaxation,
        reparam: &mut GeneralMessage,
        factor: NodeIndex<usize>,
        edge: EdgeReference<'_, GeneralAlignment, usize>,
    ) {
        for out_edge in relaxation
            .edges_directed(factor, Outgoing)
            .filter(|out_edge| out_edge.id().index() != edge.id().index())
        {
            reparam.sub_assign_outgoing(&self.messages[out_edge.id().index()], out_edge.weight());
        }
    }

    // Alternative implementation of subtract_all_other_outgoing_messages()
    // - removed nested if inside for loop, replaced with compensating addition after the loop
    // - may be faster due to avoiding if-jumps inside for-loop and vectorization of message addition
    // todo: bench performance
    fn subtract_all_other_outgoing_messages_alt(
        &self,
        relaxation: &Relaxation,
        reparam: &mut GeneralMessage,
        factor: NodeIndex<usize>,
        edge: EdgeReference<'_, GeneralAlignment, usize>,
    ) {
        self.sub_all_outgoing_messages(relaxation, reparam, factor);
        reparam.add_assign_outgoing(&self.messages[edge.id().index()], edge.weight());
    }

    // Updates the message corresponding to a given edge by computing the minimum from equation (17) in the SRMP paper
    // over a given reparametrization, then renormalizes the message so that its smallest entry becomes 0
    fn update_and_normalize(
        &mut self,
        reparam: &GeneralMessage,
        edge: EdgeReference<'_, GeneralAlignment, usize>,
    ) -> f64 {
        let delta =
            self.messages[edge.id().index()].update_with_minimization(&reparam, edge.weight());
        self.messages[edge.id().index()].add_assign_scalar(-delta);
        delta
    }

    // Updates the message corresponding to a given edge by sending messages,
    // i.e., performs a computation from equation (17) in the SRMP paper
    pub fn send(
        &mut self,
        relaxation: &Relaxation,
        edge: EdgeReference<'_, GeneralAlignment, usize>,
    ) -> f64 {
        debug!(
            "In send() for edge {} from {} to {}",
            edge.id().index(),
            edge.source().index(),
            edge.target().index()
        );

        let alpha = edge.source();
        let mut reparam_alpha = self.init_reparam(relaxation, alpha);
        self.add_all_incoming_messages(relaxation, &mut reparam_alpha, alpha);
        self.sub_all_other_outgoing_messages(relaxation, &mut reparam_alpha, alpha, edge);
        self.update_and_normalize(&reparam_alpha, edge)
    }

    // Computes a reparametrization for a given factor by sending messages to and from it,
    // i.e., performs a computation from line 5 in the SRMP paper
    pub fn compute_reparam(
        &mut self,
        relaxation: &Relaxation,
        factor: NodeIndex<usize>,
    ) -> GeneralMessage {
        debug!("In compute_reparam() for factor {}", factor.index());

        let mut reparam = self.init_reparam(relaxation, factor);
        self.add_all_incoming_messages(relaxation, &mut reparam, factor);
        self.sub_all_outgoing_messages(relaxation, &mut reparam, factor);
        reparam
    }

    // Subtracts a given reparametrization from the message corresponding to a given edge
    pub fn sub_assign_reparam(
        &mut self,
        reparam: &GeneralMessage,
        edge: EdgeReference<'_, GeneralAlignment, usize>,
    ) {
        debug!(
            "In sub_assign_reparam() for edge {} from {} to {}",
            edge.id().index(),
            edge.source().index(),
            edge.target().index()
        );

        self.messages[edge.id().index()].sub_assign_incoming(reparam);
    }

    // Computes the initial reparametrization at the start of the SRMP algorithm for a given factor
    pub fn send_srmp_initial(&mut self, relaxation: &Relaxation, factor: NodeIndex<usize>) -> f64 {
        debug!("In send_srmp_initial() for factor {}", factor.index());

        let mut theta = self.init_reparam(relaxation, factor);
        self.add_all_incoming_messages(relaxation, &mut theta, factor);
        *theta.min()
    }

    // Updates the message corresponding to a given edge by sending messages "restricted" by a given solution.
    // In other words, performs a computation similar to equation (17) in the SRMP paper,
    // but minimization is performed only over labelings consistent with the given solution.
    // Refer to the "Extracting primal solution" subsection in the SRMP section for more details.
    pub fn send_restricted(
        &self,
        relaxation: &Relaxation,
        edge: EdgeReference<'_, GeneralAlignment, usize>,
        solution: &Solution,
    ) -> GeneralMessage {
        debug!(
            "In send_restricted() for edge {} from {} to {}",
            edge.id().index(),
            edge.source().index(),
            edge.target().index()
        );

        let alpha = edge.source();
        let mut reparam_alpha = self.init_reparam(relaxation, alpha);
        self.add_all_incoming_messages(relaxation, &mut reparam_alpha, alpha);
        self.sub_all_other_outgoing_messages(relaxation, &mut reparam_alpha, alpha, edge);
        reparam_alpha.restricted_min(
            relaxation.cfn(),
            solution,
            relaxation.factor_origin(alpha),
            relaxation.factor_origin(edge.target()),
        )
    }

    // Computes "restricted" reparametrization of a given factor by sending "restricted" by a given solution.
    // Refer to the "Extracting primal solution" subsection in the SRMP section for more details.
    pub fn compute_restricted_reparam(
        &self,
        relaxation: &Relaxation,
        factor: NodeIndex<usize>,
        solution: &Solution,
    ) -> GeneralMessage {
        debug!(
            "In compute_restricted_reparam() for factor {}",
            factor.index()
        );

        let mut reparam_beta = self.init_reparam(relaxation, factor);
        self.sub_all_outgoing_messages(relaxation, &mut reparam_beta, factor);
        for in_edge in relaxation.edges_directed(factor, Incoming) {
            let alpha = relaxation.factor_origin(in_edge.source());
            let num_labeled = solution.num_labeled(relaxation.cfn().factor_variables(alpha));
            if num_labeled > 0 && num_labeled < relaxation.cfn().arity(alpha) {
                let restrected_message = self.send_restricted(relaxation, in_edge, solution);
                reparam_beta.add_assign_incoming(&restrected_message);
            } else {
                reparam_beta.add_assign_incoming(&self.messages[in_edge.id().index()]);
            }
        }
        reparam_beta
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cfn::relaxation::ConstructRelaxation,
        factor_types::{factor_type::FactorType, unary_factor::UnaryFactor},
        CostFunctionNetwork,
    };

    use super::*;

    fn construct_cfn_example_1() -> CostFunctionNetwork {
        let mut cfn = CostFunctionNetwork::from_domain_sizes(&vec![3, 4, 5], false, 3);
        cfn.add_unary_factor(
            0,
            UnaryFactor {
                function_table: vec![1., 2., 3.],
            },
        );
        cfn.add_unary_factor(
            2,
            UnaryFactor {
                function_table: vec![11., 12., 13., 14., 15.],
            },
        );
        cfn.add_non_unary_factor(vec![0, 1], FactorType::General(vec![4.; 3 * 4].into()));
        cfn.add_non_unary_factor(vec![0, 2], FactorType::General(vec![5.; 3 * 5].into()));
        cfn.add_non_unary_factor(vec![1, 2], FactorType::General(vec![6.; 4 * 5].into()));
        cfn.add_non_unary_factor(
            vec![0, 1, 2],
            FactorType::General(vec![7.; 3 * 4 * 5].into()),
        );
        cfn
    }

    #[test]
    fn new() {
        let cfn = construct_cfn_example_1();
        let relaxation = Relaxation::new(&cfn);
        let messages = Messages::new(&relaxation);

        for (index, edge) in relaxation.edge_references().enumerate() {
            let message_vec: Vec<f64> = messages.messages[index]
                .iter()
                .map(|value| *value)
                .collect();

            let factor_origin = relaxation.factor_origin(edge.target());
            let max_function_table_size = relaxation.cfn().max_function_table_size(factor_origin);

            assert_eq!(message_vec, vec![0.; max_function_table_size]);
        }
    }

    #[test]
    fn init_reparametrization() {
        let cfn = construct_cfn_example_1();
        let relaxation = Relaxation::new(&cfn);
        let messages = Messages::new(&relaxation);

        for factor in relaxation.node_indices() {
            let reparam = messages.init_reparam(&relaxation, factor);
            let reparam_vec: Vec<f64> = reparam.iter().map(|val| *val).collect();

            let factor_origin = relaxation.factor_origin(factor);
            let max_function_table_size = relaxation.cfn().max_function_table_size(factor_origin);
            let factor_type = relaxation.cfn().get_factor(factor_origin);
            let factor_vec: Vec<f64> = match factor_type {
                Some(factor_type) => (0..max_function_table_size)
                    .map(|index| factor_type[index])
                    .collect(),
                None => vec![0.; max_function_table_size],
            };

            assert_eq!(reparam_vec, factor_vec);
        }
    }

    #[test]
    fn add_all_incoming_messages() {
        let cfn = construct_cfn_example_1();
        let relaxation = Relaxation::new(&cfn);
        let mut messages = Messages::new(&relaxation);
        for message in messages.messages.iter_mut() {
            message.add_assign_scalar(1.);
        }

        for factor in relaxation.node_indices() {
            let mut reparam = messages.init_reparam(&relaxation, factor);
            let before_vec: Vec<f64> = reparam.iter().map(|value| *value).collect();
            messages.add_all_incoming_messages(&relaxation, &mut reparam, factor);

            let diff: Vec<f64> = reparam
                .iter()
                .zip(before_vec.iter())
                .map(|(after, before)| after - before)
                .collect();

            let expected_value = relaxation.edges_directed(factor, Incoming).count() as f64;
            let expected_size = cfn.max_function_table_size(relaxation.factor_origin(factor));
            assert_eq!(diff, vec![expected_value; expected_size]);
        }
    }

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
