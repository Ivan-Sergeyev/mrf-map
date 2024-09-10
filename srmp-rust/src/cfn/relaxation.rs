use petgraph::graph::DiGraph;

use crate::cfn::cost_function_network::*;

/// todo: docs
/// todo: multiple variants and methods
pub struct FactorGraph {
    graph: DiGraph<usize, (), usize>, // node data = index of term in terms vector
    term_node_index: Vec<usize>,      // node index of term in graph
}

pub struct MinimalEdges;

pub enum RelaxationType {
    MinimalEdges,
}

pub trait ConstructRelaxation<RelaxationType> {
    fn construct_relaxation(&mut self) -> FactorGraph;
}

impl ConstructRelaxation<MinimalEdges> for GeneralCFN {
    fn construct_relaxation(&mut self) -> FactorGraph {
        let mut factor_graph = FactorGraph {
            graph: DiGraph::with_capacity(
                self.num_terms(),
                2 * self.num_non_unary_terms(),
            ),
            term_node_index: Vec::with_capacity(self.num_terms()),
        };

        for (term_idx, term) in self.terms.iter().enumerate() {
            let factor_graph_node_index = factor_graph.graph.add_node(term_idx);
            factor_graph.term_node_index.push(factor_graph_node_index);

            match term {
                CFNTerm::Unary(_) => {
                    // no additional steps
                    continue;
                },
                CFNTerm::Pairwise(term) => {
                    // add edges from nodes corresponding to variables to node corresponding to term
                    let (var1, var2) = self.hypergraph.edge_endpoints(term.hyperedge_idx).unwrap();
                    factor_graph.graph.add_edge(factor_graph_node_index, var1, ());
                    factor_graph.graph.add_edge(factor_graph_node_index, var2, ());
                },
                CFNTerm::General(term) => {
                    // add edges from nodes corresponding to variables to node corresponding to term
                    unimplemented!("GeneralCFN does not support terms of higher arity than 2. Todo: use hypergraphs in implementation.");
                },
            }
        }

        factor_graph
    }
}
