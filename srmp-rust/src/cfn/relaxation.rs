#![allow(dead_code)]

use petgraph::graph::{DiGraph, EdgeIndex, EdgeReference, Neighbors, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction::{self, Incoming, Outgoing};

use crate::data_structures::hypergraph::Hypergraph;
use crate::factor_types::factor_trait::Factor;
use crate::factor_types::factor_type::FactorType;
use crate::message_passing::mp_factor_type::MessageData;
use crate::message_passing::mp_trait::MessagePassing;
use crate::message_passing::mp_unary_factor::UnaryMessageData;
use crate::{CostFunctionNetwork, FactorOrigin, GeneralCFN};

pub type RelaxationGraph = DiGraph<FactorOrigin, MessageData, usize>;

pub struct Relaxation {
    graph: RelaxationGraph,
}

impl Relaxation {
    pub fn graph(&self) -> &RelaxationGraph {
        &self.graph
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn factor_origin(&self, node: NodeIndex<usize>) -> &FactorOrigin {
        self.graph.node_weight(node).unwrap()
    }

    pub fn is_unary_factor(&self, node: NodeIndex<usize>) -> bool {
        match self.factor_origin(node) {
            FactorOrigin::Variable(_) => true,
            FactorOrigin::NonUnary(_) => false,
        }
    }

    pub fn message_data(&self, edge: EdgeIndex<usize>) -> &MessageData {
        self.graph.edge_weight(edge).unwrap()
    }

    pub fn neighbors(
        &self,
        node: NodeIndex<usize>,
        direction: Direction,
    ) -> Neighbors<MessageData, usize> {
        self.graph.neighbors_directed(node, direction)
    }

    pub fn has_edges(&self, node_index: NodeIndex<usize>, direction: Direction) -> bool {
        self.neighbors(node_index, direction).next().is_some()
    }

    pub fn incoming_edges_in_iteration() {}

    pub fn incoming_edges_opposite_iteration() {}
}

pub trait RelaxationType {}

pub struct MinimalEdges {}
impl RelaxationType for MinimalEdges {}

enum RelaxationTypes {
    MinimalEdges(MinimalEdges),
    // todo: add more relaxation methods
}

pub trait ConstructRelaxation<RT>
where
    RT: RelaxationType,
{
    fn new(cfn: &GeneralCFN) -> Self;
}

impl ConstructRelaxation<MinimalEdges> for Relaxation {
    fn new(cfn: &GeneralCFN) -> Self {
        let edge_capacity = cfn
            .factors
            .iter()
            .map(|factor| match factor {
                FactorType::Nullary(_) => 0,
                FactorType::Unary(_) => 0,
                factor => factor.arity(),
            })
            .sum();
        let mut graph = DiGraph::with_capacity(cfn.num_factors(), edge_capacity);

        // Add nodes corresponding to original variables
        for variable_index in cfn.hypergraph.iter_node_indices() {
            graph.add_node(FactorOrigin::Variable(variable_index));
        }

        for factor in &cfn.factor_origins {
            match factor {
                FactorOrigin::Variable(_) => {}
                FactorOrigin::NonUnary(hyperedge_index) => {
                    // Add node corresponding to this non-unary factor
                    let factor_node_index =
                        graph.add_node(FactorOrigin::NonUnary(*hyperedge_index));

                    // Add edges from this factor's node to the nodes of all its endpoints
                    for &variable in cfn.hypergraph.hyperedge_endpoints(*hyperedge_index) {
                        let alpha = FactorOrigin::NonUnary(*hyperedge_index);
                        let beta = FactorOrigin::Variable(variable.into());
                        let iat =
                            MessageData::UnaryFactor(UnaryMessageData::new(&cfn, &alpha, &beta));
                        graph.add_edge(factor_node_index, variable.into(), iat);
                    }
                }
            }
        }

        Relaxation { graph }
    }
}

pub struct Messages {
    messages: Vec<FactorType>,
}

fn initialize_reparametrization(
    cfn: &GeneralCFN,
    relaxation_graph: &RelaxationGraph,
    factor: NodeIndex<usize>,
) -> FactorType {
    cfn.factor_clone_for_message_passing(relaxation_graph.node_weight(factor).unwrap())
}

impl Messages {
    pub fn new(cfn: &GeneralCFN, relaxation_graph: &RelaxationGraph) -> Self {
        let mut messages = Vec::with_capacity(relaxation_graph.edge_count());
        for edge in relaxation_graph.edge_references() {
            messages
                .push(cfn.new_zero_message(relaxation_graph.node_weight(edge.target()).unwrap()));
        }
        Messages { messages }
    }

    fn add_all_incoming_messages(
        &self,
        reparametrization: &mut FactorType,
        relaxation_graph: &RelaxationGraph,
        alpha: NodeIndex<usize>,
    ) {
        for gamma_alpha in relaxation_graph.edges_directed(alpha, Incoming) {
            reparametrization.add_incoming_message(
                &self.messages[gamma_alpha.id().index()],
                gamma_alpha.weight(),
            );
        }
    }

    fn subtract_all_outgoing_messages(
        &self,
        reparametrization: &mut FactorType,
        relaxation_graph: &RelaxationGraph,
        beta: NodeIndex<usize>,
    ) {
        for gamma_beta in relaxation_graph.edges_directed(beta, Outgoing) {
            reparametrization.subtract_outgoing_message(
                &self.messages[gamma_beta.id().index()],
                gamma_beta.weight(),
            );
        }
    }

    fn subtract_all_other_outgoing_messages(
        &self,
        reparametrization: &mut FactorType,
        relaxation_graph: &RelaxationGraph,
        alpha: NodeIndex<usize>,
        alpha_beta: EdgeReference<'_, MessageData, usize>,
    ) {
        for alpha_gamma in relaxation_graph.edges_directed(alpha, Outgoing) {
            if alpha_gamma.id().index() != alpha_beta.id().index() {
                reparametrization.subtract_outgoing_message(
                    &self.messages[alpha_gamma.id().index()],
                    alpha_gamma.weight(),
                );
            }
        }
    }

    fn subtract_all_other_outgoing_messages_alternative(
        &self,
        reparametrization: &mut FactorType,
        relaxation_graph: &RelaxationGraph,
        alpha: NodeIndex<usize>,
        alpha_beta: EdgeReference<'_, MessageData, usize>,
    ) {
        // Alternative implementation of subtract_all_other_outgoing_messages()
        // - removed nested if inside for loop, replaced with compensating addition after the loop
        // - may be faster due to avoiding if-jumps inside for-loop and vectorization of message addition
        for alpha_gamma in relaxation_graph.edges_directed(alpha, Outgoing) {
            reparametrization.subtract_outgoing_message(
                &self.messages[alpha_gamma.id().index()],
                alpha_gamma.weight(),
            );
        }
        reparametrization
            .add_outgoing_message(&self.messages[alpha_beta.id().index()], alpha_beta.weight());
    }

    fn update_and_renormalize(
        &mut self,
        reparametrization: &FactorType,
        alpha_beta: EdgeReference<'_, MessageData, usize>,
    ) -> f64 {
        let delta = reparametrization.update_message_with_min(
            &mut self.messages[alpha_beta.id().index()],
            alpha_beta.weight(),
        );
        self.messages[alpha_beta.id().index()].renormalize_message(delta);
        delta
    }

    pub fn send_message(
        &mut self,
        relaxation_graph: &RelaxationGraph,
        cfn: &GeneralCFN,
        alpha_beta: EdgeReference<'_, MessageData, usize>,
    ) -> f64 {
        // Equation (17) in the SRMP paper
        // Assumptions:
        // - `relaxation_graph` was built based on `cfn`
        // - `alpha_beta` is a reference to an edge in the relaxation graph of `cfn`
        let alpha = alpha_beta.source();
        let mut theta_alpha = initialize_reparametrization(cfn, relaxation_graph, alpha);
        self.add_all_incoming_messages(&mut theta_alpha, relaxation_graph, alpha);
        self.subtract_all_other_outgoing_messages(
            &mut theta_alpha,
            relaxation_graph,
            alpha,
            alpha_beta,
        );
        self.update_and_renormalize(&theta_alpha, alpha_beta)
    }

    // todo: avoid copy-paste from send_message()
    pub fn compute_reparametrization(
        &mut self,
        relaxation_graph: &RelaxationGraph,
        cfn: &GeneralCFN,
        beta: NodeIndex<usize>,
    ) -> FactorType {
        // Line 5 in SRMP pseudocode in the paper
        // Assumptions:
        // - `relaxation_graph` was built based on `cfn`
        // - `beta` is a node reference in `relaxation_graph`
        let mut theta_beta = initialize_reparametrization(cfn, relaxation_graph, beta);
        self.add_all_incoming_messages(&mut theta_beta, relaxation_graph, beta);
        self.subtract_all_outgoing_messages(&mut theta_beta, relaxation_graph, beta);
        theta_beta
    }

    pub fn sub_assign_reparametrization(
        &mut self,
        theta_beta: &FactorType,
        alpha_beta: EdgeReference<'_, MessageData, usize>,
    ) {
        self.messages[alpha_beta.id().index()].sub_assign(theta_beta);
    }

    pub fn send_srmp_init_message(
        &mut self,
        relaxation_graph: &RelaxationGraph,
        cfn: &GeneralCFN,
        alpha: NodeIndex<usize>,
    ) -> f64 {
        // todo: description
        // Assumptions:
        // - `relaxation_graph` was built based on `cfn`
        // - `alpha` is a node reference in `relaxation_graph`
        let mut theta_alpha = initialize_reparametrization(cfn, relaxation_graph, alpha);
        self.add_all_incoming_messages(&mut theta_alpha, relaxation_graph, alpha);
        let delta = theta_alpha.max();
        delta
    }

    // todo:
    // - compute restricted minimum
    // - send restricted message
}
