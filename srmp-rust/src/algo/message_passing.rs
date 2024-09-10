#![allow(dead_code)]

use log::debug;
use petgraph::{
    graph::EdgeReference,
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};

use crate::{
    cfn::{
        factor_types::{factor_trait::Factor, FactorType},
        relaxation::{
            IndexAlignmentTable, RelaxationMinimalEdges, RelaxationType,
        },
    },
    CostFunctionNetwork, GeneralCFN,
};

pub struct MessagePassingMechanism<'a, CFN, Relaxation>
where
    CFN: CostFunctionNetwork,
    Relaxation: RelaxationType<CFN>,
{
    pub cfn: &'a CFN,
    pub relaxation: Relaxation, // todo: make a reference
    pub messages: Vec<FactorType>,
}

impl<'a> MessagePassingMechanism<'a, GeneralCFN, RelaxationMinimalEdges> {
    pub fn new(cfn: &'a GeneralCFN, relaxation: RelaxationMinimalEdges) -> Self {
        // Create initial (zero) messages
        let mut messages = Vec::with_capacity(relaxation.graph().edge_count());
        for edge in relaxation.graph().edge_references() {
            messages
                .push(cfn.new_zero_message(relaxation.graph().node_weight(edge.target()).unwrap()));
        }

        MessagePassingMechanism {
            cfn: cfn,
            relaxation: relaxation,
            messages: messages,
        }
    }

    fn add_incoming_message(
        &self,
        theta: &mut FactorType,
        edge: EdgeReference<'_, IndexAlignmentTable, usize>,
    ) {
        // Assumptions:
        // - `edge` is the index of an edge in `self.relaxation`
        // - `theta` is a reparametrization of `edge.id().target()`
        Factor::add_assign(theta, &self.messages[edge.id().index()]);
    }

    fn subtract_outgoing_message(
        &self,
        theta: &mut FactorType,
        edge: EdgeReference<'_, IndexAlignmentTable, usize>,
    ) {
        // Assumptions:
        // - `edge` is the index of an edge in `self.relaxation`
        // - `theta` is a reparametrization of `edge.id().target()`
        let index_alignment_table = self.relaxation.index_alignment_table(edge.id());
        for (b, b_index) in index_alignment_table.first_block().iter().enumerate() {
            for c_index in index_alignment_table.second_block().iter() {
                theta[*b_index + *c_index] -= self.messages[edge.id().index()][b];
            }
        }
    }

    fn add_outgoing_message(
        &self,
        theta: &mut FactorType,
        edge: EdgeReference<'_, IndexAlignmentTable, usize>,
    ) {
        // Assumptions:
        // - `edge` is the index of an edge in `self.relaxation`
        // - `theta` is a reparametrization of `edge.id().target()`
        let index_alignment_table = self.relaxation.index_alignment_table(edge.id());
        for (b, b_index) in index_alignment_table.first_block().iter().enumerate() {
            for c_index in index_alignment_table.second_block().iter() {
                theta[*b_index + *c_index] += self.messages[edge.id().index()][b];
            }
        }
    }

    fn update_message_with_min(
        &mut self,
        theta: &mut FactorType,
        edge: EdgeReference<'_, IndexAlignmentTable, usize>,
    ) -> f64 {
        // Assumptions:
        // - `edge` is the index of an edge in `self.relaxation`
        // - `theta` is a reparametrization of `edge.id().target()`
        let index_alignment_table = self.relaxation.index_alignment_table(edge.id());
        for (b, b_index) in index_alignment_table.first_block().iter().enumerate() {
            let mut v_min = theta[*b_index];
            for c_index in index_alignment_table.second_block().iter() {
                v_min = v_min.min(theta[*b_index + *c_index]);
            }
            self.messages[edge.id().index()][b] = v_min;
        }
        theta.min()
    }

    fn renormalize_message(&mut self, edge: EdgeReference<'_, IndexAlignmentTable, usize>, delta: f64) {
        self.messages[edge.id().index()].add_assign_number(-delta);
    }

    pub fn send_message(&mut self, alpha_beta: EdgeReference<'_, IndexAlignmentTable, usize>) -> f64 {
        // Assumption:
        // - `alpha_beta` is the index of an edge in `self.relaxation`

        // Initialize current reparametrization
        let alpha = alpha_beta.source();
        let alpha_origin = self.relaxation.graph().node_weight(alpha).unwrap();
        let mut theta_alpha = self.cfn.factor_clone_for_message_passing(alpha_origin);

        // Add incoming messages
        for gamma_alpha in self.relaxation.graph().edges_directed(alpha, Incoming) {
            self.add_incoming_message(&mut theta_alpha, gamma_alpha);
        }

        // Subtract outgoing messages
        if false {
            // Alternative implementation, may be faster
            for gamma_alpha in self.relaxation.graph().edges_directed(alpha, Outgoing) {
                self.subtract_outgoing_message(&mut theta_alpha, gamma_alpha);
            }
            self.add_outgoing_message(&mut theta_alpha, alpha_beta);
        } else {
            // Original implementation
            for gamma_alpha in self.relaxation.graph().edges_directed(alpha, Outgoing) {
                if gamma_alpha.id().index() != alpha_beta.id().index() {
                    self.subtract_outgoing_message(&mut theta_alpha, gamma_alpha);
                }
            }
        }

        // Update and renormalize message
        let delta = self.update_message_with_min(&mut theta_alpha, alpha_beta);
        self.renormalize_message(alpha_beta, delta);
        delta
    }

    // todo:
    // - compute restricted minimum
    // - send restricted message
    // - send MPLP messages
}
