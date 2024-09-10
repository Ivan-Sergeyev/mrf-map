#![allow(dead_code)]

use petgraph::graph::DiGraph;

use crate::data_structures::hypergraph::Hypergraph;
use crate::{CostFunctionNetwork, GeneralCFN, NonUnaryOrigin, TermOrigin, UnaryOrigin};

use super::term_types::{Term, TermType};

pub type RelaxationGraph = DiGraph<TermOrigin, (), usize>;

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
            .terms
            .iter()
            .map(|term| match term {
                Term::Nullary(_) => 0,
                Term::Unary(_) => 0,
                term => term.arity(),
            })
            .sum();
        let mut graph = DiGraph::with_capacity(self.num_terms(), edge_capacity);

        // Add nodes corresponding to original variables
        for variable_idx in self.hypergraph.iter_node_indices() {
            graph.add_node(TermOrigin::Unary(UnaryOrigin {
                node_index: variable_idx,
            }));
        }

        for term in &self.term_origins {
            match term {
                TermOrigin::Unary(_) => {}
                TermOrigin::NonUnary(term) => {
                    // Add node corresponding to this non-unary term
                    let term_node_index = graph.add_node(TermOrigin::NonUnary(NonUnaryOrigin {
                        hyperedge_index: term.hyperedge_index,
                    }));
                    // Add edges from this term's node to the nodes of all its endpoints
                    for &variable in self.hypergraph.hyperedge_endpoints(term.hyperedge_index) {
                        graph.add_edge(term_node_index, variable.into(), ());
                    }
                }
            }
        }

        graph
    }
}
