#![allow(dead_code)]

use std::marker::{self, PhantomData};

use log::debug;
use petgraph::graph::{DiGraph, EdgeReferences, Edges, Neighbors, NodeIndex, NodeIndices};
use petgraph::Directed;
use petgraph::Direction::{self};

use crate::factors::factor_trait::Factor;
use crate::{CostFunctionNetwork, FactorOrigin};

pub struct Relaxation<'a> {
    graph: DiGraph<FactorOrigin, (), usize>,
    cfn: marker::PhantomData<&'a CostFunctionNetwork>,
}

impl<'a> Relaxation<'a> {
    // Returns an iterator over all edges of the relaxation graph
    pub fn edge_references(&self) -> EdgeReferences<(), usize> {
        self.graph.edge_references()
    }

    // Returns an iterator over all nodes of the relaxation graph
    pub fn node_indices(&self) -> NodeIndices<usize> {
        self.graph.node_indices()
    }

    // Returns the number of nodes in the relaxation graph
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    // Returns the number of edges in the relaxation graph
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    // Returns the factor origin of the given node in the relaxation graph
    pub fn factor_origin(&self, node: NodeIndex<usize>) -> &FactorOrigin {
        self.graph.node_weight(node).unwrap()
    }

    // Checks if the factor corresponding to the given node in the relaxation graph is unary
    pub fn is_unary_factor(&self, node: NodeIndex<usize>) -> bool {
        match self.factor_origin(node) {
            FactorOrigin::Variable(_) => true,
            FactorOrigin::NonUnaryFactor(_) => false,
        }
    }

    // // Returns the data associated with the given edge in the relaxation graph
    // pub fn edge_data(&self, edge: EdgeIndex<usize>) -> &() {
    //     self.graph.edge_weight(edge).unwrap()
    // }

    // Returns an iterator over all edges incident to the given node in the relaxation graph pointing in the given direction
    pub fn edges_directed(
        &self,
        node: NodeIndex<usize>,
        direction: Direction,
    ) -> Edges<'_, (), Directed, usize> {
        self.graph.edges_directed(node, direction)
    }

    // Returns an iterator over the neighbors of the given node in the relaxation graph
    pub fn neighbors(&self, node: NodeIndex<usize>, direction: Direction) -> Neighbors<(), usize> {
        self.graph.neighbors_directed(node, direction)
    }

    // Checks if the given node in the relaxation graph has any edges pointing in the given direction
    pub fn has_edges(&self, node: NodeIndex<usize>, direction: Direction) -> bool {
        self.neighbors(node, direction).next().is_some()
    }

    // // Creates a new zero message corresponding to the given factor (unary or non-unary)
    // pub fn message_zero(
    //     &self,
    //     cfn: &CostFunctionNetwork,
    //     factor_origin: &FactorOrigin,
    // ) -> MessageND {
    //     // todo: change argument from FactorOrigin to NodeIndex
    //     // todo: match on factor type, return corresponding message type
    //     MessageND::zero(&cfn, factor_origin)
    // }

    // // Creates a new infinite message corresponding to the given factor (unary or non-unary)
    // pub fn message_inf(
    //     &self,
    //     cfn: &CostFunctionNetwork,
    //     factor_origin: &FactorOrigin,
    // ) -> MessageND {
    //     // todo: change argument from FactorOrigin to NodeIndex
    //     // todo: match on factor type, return corresponding message type
    //     MessageND::inf(&cfn, factor_origin)
    // }

    // // Creates a new message initialized with contents of the given factor (unary or non-unary)
    // pub fn message_clone(
    //     &self,
    //     cfn: &CostFunctionNetwork,
    //     factor_origin: &FactorOrigin,
    // ) -> MessageND {
    //     // todo: change argument from FactorOrigin to NodeIndex
    //     // todo: match on factor type, return corresponding message type
    //     MessageND::clone_factor(&cfn, factor_origin)
    // }
}

// Trait for defining relaxation types
pub trait RelaxationType {}

// Interface for constructing relaxations of different types
pub trait ConstructRelaxation<'a, RT: RelaxationType> {
    fn new(cfn: &'a CostFunctionNetwork) -> Self;

    // feature todo: pruning
    // SRMP paper mentions the following (seems to not be implemented in cpp) operation:
    // - suppose there is factor aplha with a single child beta
    // - then we can reparametrize theta to get min_{x_a ~ x_b} theta_a(x_a) = 0 for all x_b and then remove alpha
    // - this won't affect the relaxation
    // clearly, this can be iterated or, more efficiently, processed in one go in top-sort order
}

// The minimal edges relaxation type, which consists of edges from every non-unary factor to all its associated variables
pub struct MinimalEdges {}
impl RelaxationType for MinimalEdges {}

impl<'a> ConstructRelaxation<'a, MinimalEdges> for Relaxation<'a> {
    fn new(cfn: &'a CostFunctionNetwork) -> Self {
        debug!("Constructing new MinimalEdges relaxation.");

        // Create an empty directed graph with reserved capacity for nodes and edges
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

        // Create Vecs for keeping track of node indices for unary and non-unary factors
        let mut unary_nodes = Vec::with_capacity(cfn.factors_len());
        let mut non_unary_nodes = Vec::with_capacity(num_non_unary_factors);

        // Add nodes corresponding to original variables
        for variable in 0..cfn.num_variables() {
            unary_nodes.push(graph.add_node(FactorOrigin::Variable(variable)));
            debug!("Added variable {} as node {}.", { variable }, {
                unary_nodes[variable].index()
            });
        }

        // Iterate over non-unary factors
        for (factor_index, factor) in cfn
            .factors_iter()
            .enumerate()
            .filter(|(_factor_index, factor)| factor.arity() >= 2)
        {
            // Add a node corresponding to this factor
            non_unary_nodes.push(graph.add_node(FactorOrigin::NonUnaryFactor(factor_index)));
            let new_node = non_unary_nodes.last().unwrap();
            debug!("Added non-unary factor {} as node {}.", { factor_index }, {
                new_node.index()
            });

            // Add edges from this factor's to all its variables
            for variable in factor.variables() {
                let variable_node = unary_nodes[*variable];
                debug!(
                    "Adding edge from node {} to node {}.",
                    new_node.index(),
                    variable_node.index()
                );
                graph.add_edge(*new_node, variable_node, ());
            }
        }

        debug!("Finished constructing MinimalEdges relaxation.");

        Relaxation {
            graph,
            cfn: PhantomData,
        }
    }
}

enum RelaxationTypes {
    MinimalEdges(MinimalEdges),
    // todo: add more relaxation methods
}
