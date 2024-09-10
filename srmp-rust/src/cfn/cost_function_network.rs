#![allow(dead_code)]

use ndarray::Array1;

use crate::data_structures::hypergraph::{Hypergraph, UndirectedHypergraph};

use super::factor_types::{Factor, FactorType, UnaryFactor};

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
    fn get_factor_copy(&self, factor_origin: &FactorOrigin) -> FactorType;

    fn new_message(&self, term_origin: &FactorOrigin) -> FactorType;

    fn map_factors_inplace(self, mapping: fn(&mut f64)) -> Self;

    fn num_variables(&self) -> usize;
    fn domain_size(&self, variable: usize) -> usize;

    fn num_factors(&self) -> usize;
    fn num_non_unary_factors(&self) -> usize;
}

pub enum FactorOrigin {
    Unary(usize),    // node index in hypergraph
    NonUnary(usize), // hyperedge index in hypergraph
}

pub struct CFNVariable {
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
            .map(|domain_size| CFNVariable {
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
                domain_size: unary_function_table.len(),
                unary_factor_index: Some(index),
            })
            .collect();
        let term_origin = (0..unary_function_tables.len())
            .map(|index| FactorOrigin::Unary(index))
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
        if let Some(unary_factor_index) = self.hypergraph.node_data(variable).unary_factor_index {
            // overwrite existing unary term
            self.factors[unary_factor_index] = FactorType::Unary(unary_factor);
        } else {
            // add new unary term
            self.hypergraph.node_data_mut(variable).unary_factor_index = Some(self.factors.len());
            self.factors.push(FactorType::Unary(unary_factor));
            self.factor_origins.push(FactorOrigin::Unary(variable));
        }
        self
    }

    fn set_nonunary_factor(mut self, variables: Vec<usize>, nonunary_factor: FactorType) -> Self {
        let hyperedge_index = self.hypergraph.add_hyperedge(variables, self.factors.len());
        self.factors.push(nonunary_factor);
        self.factor_origins
            .push(FactorOrigin::NonUnary(hyperedge_index));
        self
    }

    fn set_factor(self, variables: Vec<usize>, term: FactorType) -> Self {
        assert_eq!(variables.len(), term.arity());
        match term {
            FactorType::Nullary(nullary_factor) => self.set_nullary_factor(nullary_factor.value()),
            FactorType::Unary(unary_factor) => self.set_unary_factor(variables[0], unary_factor),
            _ => self.set_nonunary_factor(variables, term),
        }
    }

    fn get_factor(&self, factor_origin: &FactorOrigin) -> Option<&FactorType> {
        match factor_origin {
            FactorOrigin::Unary(node_index) => {
                self.hypergraph.node_data(*node_index).unary_factor_index
            }
            FactorOrigin::NonUnary(factor_index) => Some(*factor_index),
        }
        .and_then(|factor_index| Some(&self.factors[factor_index]))
    }

    fn get_factor_copy(&self, factor_origin: &FactorOrigin) -> FactorType {
        match factor_origin {
            FactorOrigin::Unary(node_index) => {
                let variable = self.hypergraph.node_data(*node_index);
                match variable.unary_factor_index {
                    None => FactorType::Unary(Array1::zeros(variable.domain_size).into()),
                    Some(factor_index) => self.factors[factor_index].clone(),
                }
            }
            FactorOrigin::NonUnary(factor_index) => self.factors[*factor_index].clone(),
        }
    }

    fn new_message(&self, factor_origin: &FactorOrigin) -> FactorType {
        match factor_origin {
            FactorOrigin::Unary(node_index) => {
                let variable = self.hypergraph.node_data(*node_index);
                match variable.unary_factor_index {
                    None => FactorType::Unary(Array1::zeros(variable.domain_size).into()),
                    Some(factor_index) => self.factors[factor_index].new_message(),
                }
            }
            FactorOrigin::NonUnary(factor_index) => self.factors[*factor_index].new_message(),
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
        self.hypergraph.node_data(variable).domain_size
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
