#![allow(dead_code)]

use log::debug;
use petgraph::graph::{DiGraph, EdgeIndex, Edges, Neighbors, NodeIndex};
use petgraph::Directed;
use petgraph::Direction::{self};

use crate::data_structures::hypergraph::Hypergraph;
use crate::factor_types::factor_trait::Factor;
use crate::factor_types::factor_type::FactorType;
use crate::message::message_general::GeneralOutgoingAlignment;
use crate::{CostFunctionNetwork, FactorOrigin, GeneralCFN};

pub type RelaxationGraph = DiGraph<FactorOrigin, GeneralOutgoingAlignment, usize>;

pub struct Relaxation<'a> {
    cfn: &'a GeneralCFN,
    graph: RelaxationGraph,
}

impl<'a> Relaxation<'a> {
    pub fn cfn(&self) -> &GeneralCFN {
        self.cfn
    }

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

    pub fn outgoing_alignment(&self, edge: EdgeIndex<usize>) -> &GeneralOutgoingAlignment {
        self.graph.edge_weight(edge).unwrap()
    }

    pub fn edges_directed(
        &self,
        node: NodeIndex<usize>,
        direction: Direction,
    ) -> Edges<'_, GeneralOutgoingAlignment, Directed, usize> {
        self.graph.edges_directed(node, direction)
    }

    pub fn neighbors(
        &self,
        node: NodeIndex<usize>,
        direction: Direction,
    ) -> Neighbors<GeneralOutgoingAlignment, usize> {
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

pub trait ConstructRelaxation<'a, RT>
where
    RT: RelaxationType,
{
    fn new(cfn: &'a GeneralCFN) -> Self;
}

impl<'a> ConstructRelaxation<'a, MinimalEdges> for Relaxation<'a> {
    fn new(cfn: &'a GeneralCFN) -> Self {
        let edge_capacity = cfn
            .factors_iter()
            .map(|factor| match factor {
                FactorType::Unary(_) => 0,
                factor => factor.arity(),
            })
            .sum();
        let mut graph = DiGraph::with_capacity(cfn.num_factors(), edge_capacity);

        // Add nodes corresponding to original variables
        for variable_index in cfn.hypergraph.nodes_iter() {
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
                    for variable in cfn.hypergraph.hyperedge_endpoints(*hyperedge_index) {
                        let alpha = FactorOrigin::NonUnary(*hyperedge_index);
                        let beta = FactorOrigin::Variable(*variable);
                        debug!("Adding edge {} {}", *hyperedge_index, variable);
                        debug!("Endpoints are {:?}", cfn.hypergraph.hyperedge_endpoints(*hyperedge_index));
                        let weight = GeneralOutgoingAlignment::new(&cfn, &alpha, &beta);
                        graph.add_edge(factor_node_index, Into::<NodeIndex<usize>>::into(*variable), weight);
                    }
                }
            }
        }

        // todo: prune relaxation? SRMP paper mentions this (not implemented in cpp?):
        // - suppose there is factor aplha with a single child beta
        // - then we can reparametrize theta to get min_{x_a ~ x_b} theta_a(x_a) = 0 for all x_b and then remove alpha
        // - this won't affect the relaxation

        Relaxation { cfn, graph }
    }
}
