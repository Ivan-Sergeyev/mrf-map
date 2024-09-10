#![allow(dead_code)]

use ndarray::Array1;

use crate::{
    cfn::factor_types::factor_trait::Factor,
    data_structures::hypergraph::{Hypergraph, UndirectedHypergraph},
};

use super::factor_types::{unary_factor::UnaryFactor, FactorType};

pub trait CostFunctionNetwork {
    fn new() -> Self;
    // todo: create zeroed unary terms for all variables with specified domain sizes and reserve space for non-unary terms
    // fn new_full_reserved(domain_sizes: Vec<usize>, num_nonunary_factors: usize) -> Self;

    fn from_domain_sizes(domain_sizes: Vec<usize>) -> Self;
    fn from_unary_function_tables(unary_function_tables: Vec<Vec<f64>>) -> Self;

    fn set_nullary_factor(self, nullary_factor: f64) -> Self;
    fn set_unary_factor(self, variable: usize, unary_factor: UnaryFactor) -> Self;
    fn set_nonunary_factor(self, variables: Vec<usize>, nonunary_factor: FactorType) -> Self;
    fn set_factor(self, variables: Vec<usize>, term: FactorType) -> Self;

    fn get_factor(&self, term_origin: &FactorOrigin) -> Option<&FactorType>;

    fn get_factor_arity(&self, factor_origin: &FactorOrigin) -> usize;
    fn get_factor_variables(&self, factor_origin: &FactorOrigin) -> &Vec<usize>;
    fn get_function_table_len(&self, factor_origin: &FactorOrigin) -> usize;

    fn get_variables_difference(&self, alpha: &FactorOrigin, beta: &FactorOrigin) -> Vec<usize>;

    fn factor_clone_for_message_passing(&self, factor_origin: &FactorOrigin) -> FactorType;
    fn new_zero_message(&self, term_origin: &FactorOrigin) -> FactorType;

    fn map_factors_inplace(self, mapping: fn(&mut f64)) -> Self;

    fn num_variables(&self) -> usize;
    fn domain_size(&self, variable: usize) -> usize;

    fn num_factors(&self) -> usize;
    fn num_non_unary_factors(&self) -> usize;
}

pub enum FactorOrigin {
    Variable(usize), // node index in hypergraph
    NonUnary(usize), // hyperedge index in hypergraph
}

pub struct CFNVariable {
    singleton: Vec<usize>,
    domain_size: usize,
    unary_factor_index: Option<usize>, // index of corresponding unary factor in collective list (if it exits)
}

type CFNFactor = usize; // index of corresponding factor in collective list

pub struct GeneralCFN {
    pub hypergraph: UndirectedHypergraph<CFNVariable, CFNFactor>,
    pub factors: Vec<FactorType>,
    pub factor_origins: Vec<FactorOrigin>,
    pub nullary_factor: f64,
}

impl GeneralCFN {
    fn node_data(&self, node_index: usize) -> &CFNVariable {
        self.hypergraph.node_data(node_index)
    }

    fn hyperedge_data(&self, hyperedge_index: usize) -> &CFNFactor {
        self.hypergraph.hyperedge_data(hyperedge_index)
    }
}

impl CostFunctionNetwork for GeneralCFN {
    fn new() -> Self {
        GeneralCFN {
            hypergraph: UndirectedHypergraph::with_capacity(0, 0),
            factors: Vec::new(),
            factor_origins: Vec::new(),
            nullary_factor: 0.,
        }
    }

    fn from_domain_sizes(domain_sizes: Vec<usize>) -> Self {
        let node_data = domain_sizes
            .into_iter()
            .enumerate()
            .map(|(index, domain_size)| CFNVariable {
                singleton: vec![index],
                domain_size: domain_size,
                unary_factor_index: None,
            })
            .collect();

        GeneralCFN {
            hypergraph: UndirectedHypergraph::from_node_data(node_data, 0),
            factors: Vec::new(),
            factor_origins: Vec::new(),
            nullary_factor: 0.,
        }
    }

    fn from_unary_function_tables(unary_function_tables: Vec<Vec<f64>>) -> Self {
        let node_data = unary_function_tables
            .iter()
            .enumerate()
            .map(|(index, unary_function_table)| CFNVariable {
                singleton: vec![index],
                domain_size: unary_function_table.len(),
                unary_factor_index: Some(index),
            })
            .collect();
        let term_origin = (0..unary_function_tables.len())
            .map(|index| FactorOrigin::Variable(index))
            .collect();
        let terms = unary_function_tables
            .into_iter()
            .map(|unary_function_table| FactorType::Unary(unary_function_table.into()))
            .collect();

        GeneralCFN {
            hypergraph: UndirectedHypergraph::from_node_data(node_data, 0),
            factors: terms,
            factor_origins: term_origin,
            nullary_factor: 0.,
        }
    }

    fn set_nullary_factor(mut self, nullary_factor: f64) -> Self {
        self.nullary_factor = nullary_factor;
        self
    }

    fn set_unary_factor(mut self, variable: usize, unary_factor: UnaryFactor) -> Self {
        if let Some(unary_factor_index) = self.node_data(variable).unary_factor_index {
            // overwrite existing unary term
            self.factors[unary_factor_index] = FactorType::Unary(unary_factor);
        } else {
            // add new unary term
            self.hypergraph.node_data_mut(variable).unary_factor_index = Some(self.factors.len());
            self.factors.push(FactorType::Unary(unary_factor));
            self.factor_origins.push(FactorOrigin::Variable(variable));
        }
        self
    }

    fn set_nonunary_factor(mut self, variables: Vec<usize>, nonunary_factor: FactorType) -> Self {
        // Assumption: `variables` is sorted in increasing order
        let hyperedge_index = self.hypergraph.add_hyperedge(variables, self.factors.len());
        self.factors.push(nonunary_factor);
        self.factor_origins
            .push(FactorOrigin::NonUnary(hyperedge_index));
        self
    }

    fn set_factor(self, variables: Vec<usize>, term: FactorType) -> Self {
        // Assumption: `variables` is sorted in increasing order
        assert_eq!(variables.len(), term.arity());
        match term {
            FactorType::Nullary(nullary_factor) => self.set_nullary_factor(nullary_factor.value()),
            FactorType::Unary(unary_factor) => self.set_unary_factor(variables[0], unary_factor),
            _ => self.set_nonunary_factor(variables, term),
        }
    }

    fn get_factor(&self, factor_origin: &FactorOrigin) -> Option<&FactorType> {
        match factor_origin {
            FactorOrigin::Variable(node_index) => self.node_data(*node_index).unary_factor_index,
            FactorOrigin::NonUnary(factor_index) => Some(*factor_index),
        }
        .and_then(|factor_index| Some(&self.factors[factor_index]))
    }

    fn get_factor_arity(&self, factor_origin: &FactorOrigin) -> usize {
        match factor_origin {
            FactorOrigin::Variable(_) => 1,
            FactorOrigin::NonUnary(hyperedge_index) => {
                self.hypergraph.hyperedge_endpoints(*hyperedge_index).len()
            }
        }
    }

    fn get_factor_variables(&self, factor_origin: &FactorOrigin) -> &Vec<usize> {
        match factor_origin {
            FactorOrigin::Variable(node_index) => &self.node_data(*node_index).singleton,
            FactorOrigin::NonUnary(hyperedge_index) => {
                self.hypergraph.hyperedge_endpoints(*hyperedge_index)
            }
        }
    }

    fn get_function_table_len(&self, factor_origin: &FactorOrigin) -> usize {
        match factor_origin {
            FactorOrigin::Variable(node_index) => self.node_data(*node_index).domain_size,
            FactorOrigin::NonUnary(hyperedge_index) => {
                self.factors[*self.hyperedge_data(*hyperedge_index)].function_table_len()
            }
        }
    }

    fn get_variables_difference(&self, alpha: &FactorOrigin, beta: &FactorOrigin) -> Vec<usize> {
        // Assumption: alpha contains beta
        let alpha_variables = self.get_factor_variables(alpha);
        let beta_variables = self.get_factor_variables(beta);
        let mut difference = Vec::with_capacity(alpha_variables.len() - beta_variables.len());
        let mut var_b_iter = beta_variables.iter().peekable();
        for &var_a in alpha_variables {
            if var_b_iter.peek().is_some_and(|var_b| **var_b == var_a) {
                var_b_iter.next();
            } else {
                difference.push(var_a);
            }
        }
        difference
    }

    fn factor_clone_for_message_passing(&self, factor_origin: &FactorOrigin) -> FactorType {
        match factor_origin {
            FactorOrigin::Variable(node_index) => {
                let variable = self.node_data(*node_index);
                match variable.unary_factor_index {
                    None => FactorType::Unary(Array1::zeros(variable.domain_size).into()),
                    Some(factor_index) => self.factors[factor_index].clone_for_message_passing(),
                }
            }
            FactorOrigin::NonUnary(factor_index) => {
                self.factors[*factor_index].clone_for_message_passing()
            }
        }
    }

    fn new_zero_message(&self, factor_origin: &FactorOrigin) -> FactorType {
        match factor_origin {
            FactorOrigin::Variable(node_index) => {
                let variable = self.node_data(*node_index);
                match variable.unary_factor_index {
                    None => FactorType::Unary(Array1::zeros(variable.domain_size).into()),
                    Some(factor_index) => self.factors[factor_index].new_zero_message(),
                }
            }
            FactorOrigin::NonUnary(factor_index) => self.factors[*factor_index].new_zero_message(),
        }
    }

    fn map_factors_inplace(mut self, mapping: fn(&mut f64)) -> Self {
        mapping(&mut self.nullary_factor);
        for i in 0..self.num_factors() {
            self.factors[i].map_inplace(mapping);
        }
        self
    }

    fn num_variables(&self) -> usize {
        self.hypergraph.num_nodes()
    }

    fn domain_size(&self, variable: usize) -> usize {
        self.node_data(variable).domain_size
    }

    fn num_factors(&self) -> usize {
        self.factors.len()
    }

    fn num_non_unary_factors(&self) -> usize {
        self.factors
            .iter()
            .filter(|&term| match term {
                FactorType::Nullary(_) => false,
                FactorType::Unary(_) => false,
                _ => true,
            })
            .count()
    }
}
