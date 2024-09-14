#![allow(dead_code)]

use log::debug;
use petgraph::graph::{
    DiGraph, EdgeIndex, EdgeReferences, Edges, Neighbors, NodeIndex, NodeIndices,
};
use petgraph::Directed;
use petgraph::Direction::{self};

use crate::factor_types::factor_trait::Factor;
use crate::message::message_general::GeneralAlignment;
use crate::{CostFunctionNetwork, FactorOrigin};

type RNodeData = FactorOrigin;
type REdgeData = GeneralAlignment;
pub type RelaxationGraph = DiGraph<RNodeData, REdgeData, usize>;

pub struct Relaxation<'a> {
    cfn: &'a CostFunctionNetwork,
    graph: RelaxationGraph,
}

impl<'a> Relaxation<'a> {
    pub fn cfn(&self) -> &CostFunctionNetwork {
        self.cfn
    }

    pub fn edge_references(&self) -> EdgeReferences<GeneralAlignment, usize> {
        self.graph.edge_references()
    }

    pub fn node_indices(&self) -> NodeIndices<usize> {
        self.graph.node_indices()
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn factor_origin(&self, node: NodeIndex<usize>) -> &RNodeData {
        self.graph.node_weight(node).unwrap()
    }

    pub fn is_unary_factor(&self, node: NodeIndex<usize>) -> bool {
        match self.factor_origin(node) {
            RNodeData::Variable(_) => true,
            RNodeData::NonUnaryFactor(_) => false,
        }
    }

    pub fn edge_data(&self, edge: EdgeIndex<usize>) -> &REdgeData {
        self.graph.edge_weight(edge).unwrap()
    }

    pub fn edges_directed(
        &self,
        node: NodeIndex<usize>,
        direction: Direction,
    ) -> Edges<'_, REdgeData, Directed, usize> {
        self.graph.edges_directed(node, direction)
    }

    pub fn neighbors(
        &self,
        node: NodeIndex<usize>,
        direction: Direction,
    ) -> Neighbors<REdgeData, usize> {
        self.graph.neighbors_directed(node, direction)
    }

    pub fn has_edges(&self, node_index: NodeIndex<usize>, direction: Direction) -> bool {
        self.neighbors(node_index, direction).next().is_some()
    }
}

pub trait RelaxationType {}

pub struct MinimalEdges {}
impl RelaxationType for MinimalEdges {}

enum RelaxationTypes {
    MinimalEdges(MinimalEdges),
    // todo: add more relaxation methods
}

pub trait ConstructRelaxation<'a, RT: RelaxationType> {
    fn new(cfn: &'a CostFunctionNetwork) -> Self;
}

impl<'a> ConstructRelaxation<'a, MinimalEdges> for Relaxation<'a> {
    fn new(cfn: &'a CostFunctionNetwork) -> Self {
        debug!("Constructing new MinimalEdges relaxation");

        // Create an empty directed graph with reserved capacity for nodes and edge
        let num_non_unary_factors = cfn
            .factors_iter()
            .filter(|factor| factor.arity() > 1)
            .count();
        let edge_capacity = cfn
            .factors_iter()
            .filter_map(|factor| {
                if factor.arity() > 1 {
                    Some(factor.arity())
                } else {
                    None
                }
            })
            .sum();

        let mut graph = DiGraph::with_capacity(cfn.factors_len(), edge_capacity);
        let mut unary_nodes = Vec::with_capacity(cfn.factors_len());
        let mut non_unary_nodes = Vec::with_capacity(num_non_unary_factors);

        // Add nodes corresponding to original variables
        for variable in 0..cfn.num_variables() {
            unary_nodes.push(graph.add_node(RNodeData::Variable(variable)));
            debug!("Added variable {} as node {}", { variable }, {
                unary_nodes[variable].index()
            });
        }

        for (factor_index, factor) in cfn.factors_iter().enumerate() {
            if factor.arity() <= 1 {
                continue;
            }

            // Add a node corresponding to this non-unary factor
            non_unary_nodes.push(graph.add_node(RNodeData::NonUnaryFactor(factor_index)));
            let new_node = non_unary_nodes.last().unwrap();
            debug!("Added non-unary factor {} as node {}", { factor_index }, {
                new_node.index()
            });

            // Add edges from this factor's node to the nodes of all its endpoints
            for variable in factor.variables() {
                let variable_node = unary_nodes[*variable];
                debug!(
                    "Adding edge from node {} to node {}",
                    new_node.index(),
                    variable_node.index()
                );
                let alpha = RNodeData::NonUnaryFactor(factor_index);
                let beta = RNodeData::Variable(*variable);
                let weight = REdgeData::new(&cfn, &alpha, &beta);
                graph.add_edge(*new_node, variable_node, weight);
            }
        }

        debug!("Finished construction of MinimalEdges relaxation");

        // feature todo: prune relaxation? SRMP paper mentions this (not implemented in cpp?):
        // - suppose there is factor aplha with a single child beta
        // - then we can reparametrize theta to get min_{x_a ~ x_b} theta_a(x_a) = 0 for all x_b and then remove alpha
        // - this won't affect the relaxation

        Relaxation { cfn, graph }
    }
}
