#![allow(dead_code)]

use petgraph::graph::DiGraph;

use crate::data_structures::hypergraph::Hypergraph;
use crate::{CostFunctionNetwork, FactorOrigin, GeneralCFN};

use super::factor_types::{Factor, FactorType};

pub type RelaxationGraph = DiGraph<FactorOrigin, (), usize>;

// todo: multiple relaxation methods
pub struct MinimalEdges;

pub enum RelaxationType {
    MinimalEdges(MinimalEdges),
}

pub trait ConstructRelaxation<RelaxationType>
where
    Self: CostFunctionNetwork,
{
    fn construct_relaxation(&self) -> RelaxationGraph;
}

impl ConstructRelaxation<MinimalEdges> for GeneralCFN {
    fn construct_relaxation(&self) -> RelaxationGraph {
        let edge_capacity = self
            .factors
            .iter()
            .map(|term| match term {
                FactorType::Nullary(_) => 0,
                FactorType::Unary(_) => 0,
                term => term.arity(),
            })
            .sum();
        let mut graph = DiGraph::with_capacity(self.num_factors(), edge_capacity);

        // Add nodes corresponding to original variables
        for variable_index in self.hypergraph.iter_node_indices() {
            graph.add_node(FactorOrigin::Unary(variable_index));
        }

        for term in &self.factor_origins {
            match term {
                FactorOrigin::Unary(_) => {}
                FactorOrigin::NonUnary(hyperedge_index) => {
                    // Add node corresponding to this non-unary term
                    let term_node_index = graph.add_node(FactorOrigin::NonUnary(*hyperedge_index));
                    // Add edges from this term's node to the nodes of all its endpoints
                    for &variable in self.hypergraph.hyperedge_endpoints(*hyperedge_index) {
                        graph.add_edge(term_node_index, variable.into(), ());
                    }
                }
            }
        }

        graph
    }
}
