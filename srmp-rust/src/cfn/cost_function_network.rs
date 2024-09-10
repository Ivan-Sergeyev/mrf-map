#![allow(dead_code)]

use crate::data_structures::hypergraph::{Hypergraph, UndirectedHypergraph};

use super::term_types::{Term, TermType, UnaryTerm};

pub trait CostFunctionNetwork {
    fn new() -> Self;
    // todo: create zeroed unary terms for all variables with specified domain sizes and reserve space for non-unary terms
    // fn new_full_reserved(domain_sizes: Vec<usize>, num_nonunary_terms: usize) -> Self;

    fn from_domain_sizes(domain_sizes: Vec<usize>) -> Self;
    fn from_unary_function_tables(unary_function_tables: Vec<Vec<f64>>) -> Self;

    fn set_nullary_term(self, nullary_term: f64) -> Self;
    fn set_unary_term(self, variable: usize, unary_term: UnaryTerm) -> Self;
    fn set_nonunary_term(self, variables: Vec<usize>, nonunary_term: Term) -> Self;
    fn set_term(self, variables: Vec<usize>, term: Term) -> Self;

    fn map_terms_inplace(self, mapping: fn(&mut f64)) -> Self;

    fn num_variables(&self) -> usize;
    fn domain_size(&self, variable: usize) -> usize;

    fn num_terms(&self) -> usize;
    fn num_non_unary_terms(&self) -> usize;
}

pub struct UnaryOrigin {
    pub node_index: usize,
}

pub struct NonUnaryOrigin {
    pub hyperedge_index: usize,
}

pub enum TermOrigin {
    Unary(UnaryOrigin),
    NonUnary(NonUnaryOrigin),
}

pub struct CFNVariable {
    domain_size: usize,
    unary_term_index: Option<usize>,
}

pub struct CFNTerm {
    term_index: usize,
}

pub struct GeneralCFN {
    pub hypergraph: UndirectedHypergraph<CFNVariable, CFNTerm>,
    pub terms: Vec<Term>,
    pub term_origins: Vec<TermOrigin>,
    pub nullary_term: f64,
}

impl CostFunctionNetwork for GeneralCFN {
    fn new() -> Self {
        GeneralCFN {
            hypergraph: UndirectedHypergraph::with_capacity(0, 0),
            terms: Vec::new(),
            term_origins: Vec::new(),
            nullary_term: 0.,
        }
    }

    fn from_domain_sizes(domain_sizes: Vec<usize>) -> Self {
        let node_data = domain_sizes
            .into_iter()
            .map(|domain_size| CFNVariable {
                domain_size: domain_size,
                unary_term_index: None,
            })
            .collect();

        GeneralCFN {
            hypergraph: UndirectedHypergraph::from_node_data(node_data, 0),
            terms: Vec::new(),
            term_origins: Vec::new(),
            nullary_term: 0.,
        }
    }

    fn from_unary_function_tables(unary_function_tables: Vec<Vec<f64>>) -> Self {
        let node_data = unary_function_tables
            .iter()
            .enumerate()
            .map(|(index, unary_function_table)| CFNVariable {
                domain_size: unary_function_table.len(),
                unary_term_index: Some(index),
            })
            .collect();
        let term_origin = (0..unary_function_tables.len())
            .map(|index| TermOrigin::Unary(UnaryOrigin { node_index: index }))
            .collect();
        let terms = unary_function_tables
            .into_iter()
            .map(|unary_function_table| Term::Unary(unary_function_table.into()))
            .collect();

        GeneralCFN {
            hypergraph: UndirectedHypergraph::from_node_data(node_data, 0),
            terms: terms,
            term_origins: term_origin,
            nullary_term: 0.,
        }
    }

    fn set_nullary_term(mut self, nullary_term: f64) -> Self {
        self.nullary_term = nullary_term;
        self
    }

    fn set_unary_term(mut self, variable: usize, unary_term: UnaryTerm) -> Self {
        if let Some(unary_term_index) = self.hypergraph.node_data(variable).unary_term_index {
            // overwrite existing unary term
            self.terms[unary_term_index] = Term::Unary(unary_term);
        } else {
            // add new unary term
            self.hypergraph.node_data_mut(variable).unary_term_index = Some(self.terms.len());
            self.terms.push(Term::Unary(unary_term));
            self.term_origins.push(TermOrigin::Unary(UnaryOrigin {
                node_index: variable,
            }));
        }
        self
    }

    fn set_nonunary_term(mut self, variables: Vec<usize>, nonunary_term: Term) -> Self {
        let hyperedge_index = self.hypergraph.add_hyperedge(
            variables,
            CFNTerm {
                term_index: self.terms.len(),
            },
        );
        self.terms.push(nonunary_term);
        self.term_origins.push(TermOrigin::NonUnary(NonUnaryOrigin {
            hyperedge_index: hyperedge_index,
        }));
        self
    }

    fn set_term(self, variables: Vec<usize>, term: Term) -> Self {
        assert_eq!(variables.len(), term.arity());
        match term {
            Term::Nullary(nullary_term) => self.set_nullary_term(nullary_term.value()),
            Term::Unary(unary_term) => self.set_unary_term(variables[0], unary_term),
            _ => self.set_nonunary_term(variables, term),
        }
    }

    fn map_terms_inplace(mut self, mapping: fn(&mut f64)) -> Self {
        mapping(&mut self.nullary_term);
        for i in 0..self.num_terms() {
            self.terms[i].map_inplace(mapping);
        }
        self
    }

    fn num_variables(&self) -> usize {
        self.hypergraph.num_nodes()
    }

    fn domain_size(&self, variable: usize) -> usize {
        self.hypergraph.node_data(variable).domain_size
    }

    fn num_terms(&self) -> usize {
        self.terms.len()
    }

    fn num_non_unary_terms(&self) -> usize {
        self.terms
            .iter()
            .filter(|&term| match term {
                Term::Nullary(_) => false,
                Term::Unary(_) => false,
                _ => true,
            })
            .count()
    }
}
