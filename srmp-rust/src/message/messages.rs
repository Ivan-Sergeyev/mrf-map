#![allow(dead_code)]

use petgraph::{
    graph::{EdgeReference, NodeIndex},
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};

use crate::{
    cfn::{solution::Solution, relaxation::Relaxation},
    CostFunctionNetwork,
};

use super::{
    message_general::{GeneralMessage, GeneralOutgoingAlignment},
    message_trait::Message,
};

pub struct Messages {
    messages: Vec<GeneralMessage>,
}

impl Messages {
    pub fn new(relaxation: &Relaxation) -> Self {
        let mut messages = Vec::with_capacity(relaxation.graph().edge_count());
        for edge in relaxation.graph().edge_references() {
            messages.push(
                relaxation
                    .cfn()
                    .new_zero_message(relaxation.graph().node_weight(edge.target()).unwrap())
                    .into(),
            );
        }
        Messages { messages }
    }

    fn init_reparametrization(
        &self,
        relaxation: &Relaxation,
        factor_node: NodeIndex<usize>,
    ) -> GeneralMessage {
        relaxation
            .cfn()
            .factor_clone_for_message_passing(relaxation.factor_origin(factor_node))
            .into()
    }

    fn add_all_incoming_messages(
        &self,
        relaxation: &Relaxation,
        reparametrization: &mut GeneralMessage,
        factor_node: NodeIndex<usize>,
    ) {
        for gamma_alpha in relaxation.edges_directed(factor_node, Incoming) {
            reparametrization.add_assign_incoming(&self.messages[gamma_alpha.id().index()]);
        }
    }

    fn sub_all_outgoing_messages(
        &self,
        relaxation: &Relaxation,
        reparametrization: &mut GeneralMessage,
        factor_node: NodeIndex<usize>,
    ) {
        for out_edge in relaxation.edges_directed(factor_node, Outgoing) {
            reparametrization
                .sub_assign_outgoing(&self.messages[out_edge.id().index()], out_edge.weight());
        }
    }

    fn sub_all_other_outgoing_messages(
        &self,
        relaxation: &Relaxation,
        reparametrization: &mut GeneralMessage,
        factor_node: NodeIndex<usize>,
        edge: EdgeReference<'_, GeneralOutgoingAlignment, usize>,
    ) {
        for out_edge in relaxation
            .edges_directed(factor_node, Outgoing)
            .filter(|out_edge| out_edge.id().index() != edge.id().index())
        {
            reparametrization
                .sub_assign_outgoing(&self.messages[out_edge.id().index()], out_edge.weight());
        }
    }

    fn subtract_all_other_outgoing_messages_alt(
        &self,
        relaxation: &Relaxation,
        reparametrization: &mut GeneralMessage,
        factor_node: NodeIndex<usize>,
        edge: EdgeReference<'_, GeneralOutgoingAlignment, usize>,
    ) {
        // Alternative implementation of subtract_all_other_outgoing_messages()
        // - removed nested if inside for loop, replaced with compensating addition after the loop
        // - may be faster due to avoiding if-jumps inside for-loop and vectorization of message addition
        self.sub_all_outgoing_messages(relaxation, reparametrization, factor_node);
        reparametrization.add_assign_outgoing(&self.messages[edge.id().index()], edge.weight());
    }

    fn update_and_normalize(
        &mut self,
        reparametrization: &GeneralMessage,
        edge: EdgeReference<'_, GeneralOutgoingAlignment, usize>,
    ) -> f64 {
        let delta = self.messages[edge.id().index()]
            .update_with_minimization(&reparametrization, edge.weight());
        self.messages[edge.id().index()].add_assign_scalar(-delta);
        delta
    }

    pub fn send(
        &mut self,
        relaxation: &Relaxation,
        edge: EdgeReference<'_, GeneralOutgoingAlignment, usize>,
    ) -> f64 {
        // Equation (17) in the SRMP paper
        let alpha = edge.source();
        let mut theta_alpha = self.init_reparametrization(relaxation, alpha);
        self.add_all_incoming_messages(relaxation, &mut theta_alpha, alpha);
        self.sub_all_other_outgoing_messages(relaxation, &mut theta_alpha, alpha, edge);
        self.update_and_normalize(&theta_alpha, edge)
    }

    pub fn compute_reparametrization(
        &mut self,
        relaxation: &Relaxation,
        factor_node: NodeIndex<usize>,
    ) -> GeneralMessage {
        // Line 5 in SRMP pseudocode in the paper
        let mut theta = self.init_reparametrization(relaxation, factor_node);
        self.add_all_incoming_messages(relaxation, &mut theta, factor_node);
        self.sub_all_outgoing_messages(relaxation, &mut theta, factor_node);
        theta
    }

    pub fn sub_assign_reparametrization(
        &mut self,
        reparametrization: &GeneralMessage,
        edge: EdgeReference<'_, GeneralOutgoingAlignment, usize>,
    ) {
        self.messages[edge.id().index()].sub_assign_incoming(reparametrization);
    }

    pub fn send_srmp_initial(
        &mut self,
        relaxation: &Relaxation,
        factor_node: NodeIndex<usize>,
    ) -> f64 {
        // Computes initial reparametrization in SRMP for a given factor `alpha`
        let mut theta = self.init_reparametrization(relaxation, factor_node);
        self.add_all_incoming_messages(relaxation, &mut theta, factor_node);
        *theta.min()
    }

    pub fn send_restricted(
        &self,
        relaxation: &Relaxation,
        edge: EdgeReference<'_, GeneralOutgoingAlignment, usize>,
        solution: &Solution,
    ) -> GeneralMessage {
        // Similar to equation (17) in the SRMP paper, but minimization is performed only over labelings consistent with current labeling
        let alpha = edge.source();
        let mut theta_alpha = self.init_reparametrization(relaxation, alpha);
        self.add_all_incoming_messages(relaxation, &mut theta_alpha, alpha);
        self.sub_all_other_outgoing_messages(relaxation, &mut theta_alpha, alpha, edge);
        theta_alpha.restricted_min(
            relaxation.cfn(),
            solution,
            relaxation.factor_origin(alpha),
            relaxation.factor_origin(edge.target()),
        )
    }

    pub fn compute_restricted_reparametrization(
        &self,
        relaxation: &Relaxation,
        factor_node: NodeIndex<usize>,
        solution: &Solution,
    ) -> GeneralMessage {
        // Computes "restricted" reparametrization of factor `beta` by sending "restricted" messages (maintains consistency with current labeling)
        let mut theta_beta = self.init_reparametrization(relaxation, factor_node);
        self.sub_all_outgoing_messages(relaxation, &mut theta_beta, factor_node);
        for alpha_beta in relaxation.edges_directed(factor_node, Incoming) {
            let alpha = relaxation.factor_origin(alpha_beta.source());
            let num_labeled = solution.num_labeled(relaxation.cfn(), alpha);
            if num_labeled > 0 && num_labeled < relaxation.cfn().arity(alpha) {
                let restrected_message = self.send_restricted(relaxation, alpha_beta, solution);
                theta_beta.add_assign_incoming(&restrected_message);
            } else {
                theta_beta.add_assign_incoming(&self.messages[alpha_beta.id().index()]);
            }
        }
        theta_beta
    }
}
