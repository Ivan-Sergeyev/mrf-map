#![allow(dead_code)]

use petgraph::graph::DiGraph;

use crate::data_structures::hypergraph::Hypergraph;
use crate::{CostFunctionNetwork, FactorOrigin, GeneralCFN};

use super::factor_types::{Factor, FactorType};

pub struct IndexAlignmentTable {
    first_block_indices: Vec<usize>,
    second_block_indices: Vec<usize>,
}

impl IndexAlignmentTable {
    fn _compute_index_adjustment(
        cfn: &GeneralCFN,
        alpha_variables: &Vec<usize>,
        beta_variables: &Vec<usize>,
        beta_function_table_len: usize,
    ) -> Vec<usize> {
        let mut k_array = vec![0; beta_variables.len() + 1];
        k_array[beta_variables.len()] = 1; // barrier element
        let mut alpha_var_idx = alpha_variables.len() - 1;
        for (beta_var_idx, &beta_var) in beta_variables.iter().rev().enumerate() {
            k_array[beta_var_idx] = k_array[beta_var_idx + 1];
            while beta_var != alpha_variables[alpha_var_idx] {
                k_array[beta_var_idx] *= cfn.domain_size(alpha_variables[alpha_var_idx]);
                alpha_var_idx -= 1;
            }
        }

        let mut beta_labeling = vec![0; beta_variables.len()];
        let mut index_adjustment_table = vec![0; beta_function_table_len];
        index_adjustment_table[0] = 0;
        let mut beta_var_idx = beta_variables.len() - 1;
        let mut table_idx = 0;
        let mut k = 0;
        loop {
            if beta_labeling[beta_var_idx] < cfn.domain_size(beta_variables[beta_var_idx]) - 1 {
                // Move to next variable label
                beta_labeling[beta_var_idx] += 1;
                k += k_array[beta_var_idx];
                table_idx += 1;
                index_adjustment_table[table_idx] = k;
                beta_var_idx = beta_variables.len() - 1;
            } else {
                // "Carry over" to initial label
                k -= beta_labeling[beta_var_idx] * k_array[beta_var_idx];
                beta_labeling[beta_var_idx] = 0;
                if beta_var_idx == 0 {
                    break;
                }
                beta_var_idx -= 1;
            }
        }

        index_adjustment_table
    }

    fn new(cfn: &GeneralCFN, alpha: &FactorOrigin, beta: &FactorOrigin) -> Self {
        let alpha_variables = cfn.get_factor_variables(alpha);
        let beta_variables = cfn.get_factor_variables(beta);

        let alpha_ft_len = cfn.get_function_table_len(alpha);
        let beta_ft_len = cfn.get_function_table_len(beta);
        let difference_ft_len = alpha_ft_len / beta_ft_len;

        let first_block_indices = IndexAlignmentTable::_compute_index_adjustment(
            cfn,
            alpha_variables,
            beta_variables,
            beta_ft_len,
        );

        let difference = cfn.get_variables_difference(alpha, beta);
        let second_block_indices = IndexAlignmentTable::_compute_index_adjustment(
            cfn,
            alpha_variables,
            &difference,
            difference_ft_len,
        );

        IndexAlignmentTable {
            first_block_indices,
            second_block_indices,
        }
    }
}

pub type RelaxationGraph = DiGraph<FactorOrigin, IndexAlignmentTable, usize>;

pub struct MinimalEdges;

pub enum RelaxationType {
    MinimalEdges(MinimalEdges),
    // todo: add more relaxation methods
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
            graph.add_node(FactorOrigin::Variable(variable_index));
        }

        for term in &self.factor_origins {
            match term {
                FactorOrigin::Variable(_) => {}
                FactorOrigin::NonUnary(hyperedge_index) => {
                    // Add node corresponding to this non-unary term
                    let term_node_index = graph.add_node(FactorOrigin::NonUnary(*hyperedge_index));
                    // Add edges from this term's node to the nodes of all its endpoints
                    for &variable in self.hypergraph.hyperedge_endpoints(*hyperedge_index) {
                        let alpha = FactorOrigin::NonUnary(*hyperedge_index);
                        let beta = FactorOrigin::Variable(variable.into());
                        let iat = IndexAlignmentTable::new(&self, &alpha, &beta);
                        graph.add_edge(term_node_index, variable.into(), iat);
                    }
                }
            }
        }

        graph
    }
}
