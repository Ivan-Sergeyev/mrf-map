#![allow(dead_code)]

use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    str::FromStr,
};

use ndarray::{Array, Array1, Array2, ArrayD, Ix1, Ix2};
use petgraph::graph::DiGraph;

use crate::data_structures::hypergraph::{Hypergraph, UndirectedHypergraph};

pub trait CostFunctionNetwork {
    fn new() -> Self;
    fn from_domain_sizes(domain_sizes: Vec<usize>) -> Self;
    fn from_unary_terms(unary_function_tables: Vec<Vec<f64>>) -> Self;

    fn set_nullary_term(self, nullary_term: f64) -> Self;
    fn set_term(self, variables: Vec<usize>, function_table: ArrayD<f64>) -> Self;

    fn num_variables(&self) -> usize;
    fn domain_size(&self, variable: usize) -> usize;

    fn num_terms(&self) -> usize;
    fn num_non_unary_terms(&self) -> usize;
}

/// todo: add LG option
/// model format: https://uaicompetition.github.io/uci-2022/file-formats/model-format/
pub trait UAI
where
    Self: CostFunctionNetwork,
{
    fn read_from_uai(file: File) -> Self;
    fn write_to_uai(&self, file: File) -> io::Result<()>;
}

pub enum UAIState {
    GraphType,
    NumberOfVariables,
    DomainSizes,
    NumberOfFunctions,
    FunctionScopes(usize),
    NumberOfTableValues(usize),
    TableValues(usize, usize),
    EndOfFile,
}

/// todo: multiple variants and methods
pub struct RelaxationGraph {
    graph: DiGraph<CFNOrigin, (), usize>,
    node_index_of_term: Vec<usize>,
}

enum CFNOrigin {
    Node(usize),
    Hyperedge(usize),
}

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

// implementation details
struct UnaryTerm {
    function_table: Array1<f64>,
    variable_idx: usize,
}

struct PairwiseTerm {
    function_table: Array2<f64>,
    hyperedge_idx: usize,
}

struct GeneralTerm {
    function_table: ArrayD<f64>,
    hyperedge_idx: usize,
}

enum CFNTerm {
    Unary(UnaryTerm),
    Pairwise(PairwiseTerm),
    General(GeneralTerm),
}

struct CFNVariable {
    domain_size: usize,
    unary_term_idx: Option<usize>,
}

pub struct GeneralCFN {
    hypergraph: UndirectedHypergraph<CFNVariable, usize>,
    terms: Vec<CFNTerm>,
    nullary_term: f64,
}

impl CostFunctionNetwork for GeneralCFN {
    fn new() -> Self {
        GeneralCFN {
            hypergraph: UndirectedHypergraph::with_capacity(0, 0),
            terms: Vec::new(),
            nullary_term: 0.,
        }
    }

    fn from_domain_sizes(domain_sizes: Vec<usize>) -> Self {
        let node_data = domain_sizes
            .into_iter()
            .map(|domain_size| CFNVariable {
                domain_size: domain_size,
                unary_term_idx: None,
            })
            .collect();

        GeneralCFN {
            hypergraph: UndirectedHypergraph::from_node_data(node_data, 0),
            terms: Vec::new(),
            nullary_term: 0.,
        }
    }

    fn from_unary_terms(unary_function_tables: Vec<Vec<f64>>) -> Self {
        let node_data = unary_function_tables
            .iter()
            .enumerate()
            .map(|(index, unary_function_table)| CFNVariable {
                domain_size: unary_function_table.len(),
                unary_term_idx: Some(index),
            })
            .collect();
        let terms = unary_function_tables
            .into_iter()
            .enumerate()
            .map(|(index, unary_function_table)| {
                CFNTerm::Unary(UnaryTerm {
                    function_table: unary_function_table.into(),
                    variable_idx: index,
                })
            })
            .collect();

        GeneralCFN {
            hypergraph: UndirectedHypergraph::from_node_data(node_data, 0),
            terms: terms,
            nullary_term: 0.,
        }
    }

    fn set_nullary_term(mut self, nullary_term: f64) -> Self {
        self.nullary_term = nullary_term;
        self
    }

    fn set_term(mut self, variables: Vec<usize>, function_table: ArrayD<f64>) -> Self {
        assert!(variables
            .iter()
            .all(|&variable| variable < self.num_variables()));
        match variables.len() {
            0 => self.set_nullary_term(function_table[[0]]),
            1 => {
                let new_unary_term = CFNTerm::Unary(UnaryTerm {
                    function_table: function_table
                        .into_dimensionality::<Ix1>()
                        .expect("Function table should be 1-dimensional"),
                    variable_idx: variables[0],
                });
                if let Some(unary_term_index) =
                    self.hypergraph.node_data(variables[0]).unary_term_idx
                {
                    self.terms[unary_term_index] = new_unary_term;
                } else {
                    self.hypergraph.node_data_mut(variables[0]).unary_term_idx =
                        Some(self.terms.len());
                    self.terms.push(new_unary_term);
                }
                self
            }
            2 => {
                // todo: check if the term already exists
                let hyperedge_idx = self.hypergraph.add_hyperedge(variables, self.terms.len());
                self.terms.push(CFNTerm::Pairwise(PairwiseTerm {
                    function_table: function_table
                        .into_dimensionality::<Ix2>()
                        .expect("Function table should be 2-dimensional"),
                    hyperedge_idx: hyperedge_idx,
                }));
                self
            }
            _ => {
                // todo: check if the term already exists
                let hyperedge_idx = self.hypergraph.add_hyperedge(variables, self.terms.len());
                self.terms.push(CFNTerm::General(GeneralTerm {
                    function_table: function_table.into(),
                    hyperedge_idx: hyperedge_idx,
                }));
                self
            }
        }
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
                CFNTerm::Unary(_) => false,
                _ => true,
            })
            .count()
    }
}

fn string_to_vec<T>(string: &str) -> Vec<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    string
        .split_whitespace()
        .map(|s| s.parse::<T>().unwrap())
        .collect()
}

fn vec_to_string<T: ToString>(v: &Vec<T>) -> String {
    v.iter()
        .map(|elem| elem.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

impl UAI for GeneralCFN {
    fn read_from_uai(file: File) -> Self {
        let lines = BufReader::new(file).lines();

        let mut state = UAIState::GraphType;
        let mut trimmed_line;

        let mut num_variables = 0;
        let mut cfn = GeneralCFN::new();
        let mut function_scopes = Vec::new();
        let mut function_entries = Vec::new();

        for line in lines {
            let line = line.unwrap();
            trimmed_line = line.trim();

            // skip empty lines
            if trimmed_line.is_empty() {
                continue;
            }

            match state {
                UAIState::GraphType => {
                    if trimmed_line != "MARKOV" {
                        unimplemented!("Only MARKOV graph type is supported.");
                    }
                    state = UAIState::NumberOfVariables;
                }
                UAIState::NumberOfVariables => {
                    num_variables = trimmed_line.parse::<usize>().unwrap();
                    state = UAIState::DomainSizes;
                }
                UAIState::DomainSizes => {
                    let domain_sizes = string_to_vec(trimmed_line);
                    assert_eq!(num_variables, domain_sizes.len());
                    cfn = GeneralCFN::from_domain_sizes(domain_sizes);
                    state = UAIState::NumberOfFunctions;
                }
                UAIState::NumberOfFunctions => {
                    let num_functions = trimmed_line.parse::<usize>().unwrap();
                    function_scopes = Vec::with_capacity(num_functions);
                    state = UAIState::FunctionScopes(0);
                }
                UAIState::FunctionScopes(function_idx) => {
                    let function_desc = string_to_vec(trimmed_line);
                    let (scope_len, function_scope) = function_desc.split_at(1);
                    assert_eq!(scope_len[0], function_scope.len());
                    function_scopes.push(function_scope.to_vec());
                    state = if function_idx + 1 < function_scopes.capacity() {
                        UAIState::FunctionScopes(function_idx + 1)
                    } else {
                        UAIState::NumberOfTableValues(0)
                    };
                }
                UAIState::NumberOfTableValues(function_idx) => {
                    assert!(function_idx < function_scopes.len());
                    let num_entries = trimmed_line.parse::<usize>().unwrap();
                    function_entries = Vec::with_capacity(num_entries);
                    state = UAIState::TableValues(function_idx, num_entries);
                }
                UAIState::TableValues(function_idx, max_num_entries) => {
                    assert!(function_idx < function_scopes.len());
                    let mut new_entries = string_to_vec(trimmed_line);
                    function_entries.append(&mut new_entries);

                    let cur_num_entries = function_entries.len();
                    assert!(cur_num_entries <= max_num_entries);
                    if cur_num_entries < max_num_entries {
                        // need to collect more table entries
                        state = UAIState::TableValues(function_idx, max_num_entries);
                        continue;
                    }

                    // collected all table entries, ready to add term to cost function network
                    let function_table = Array::from_shape_vec(
                        function_scopes[function_idx]
                            .iter()
                            .map(|&var| cfn.domain_size(var))
                            .collect::<Vec<usize>>(),
                        function_entries.drain(..).collect(),
                    )
                    .unwrap();
                    cfn = cfn.set_term(function_scopes[function_idx].to_vec(), function_table);

                    state = if function_idx + 1 < function_scopes.len() {
                        UAIState::NumberOfTableValues(function_idx + 1)
                    } else {
                        UAIState::EndOfFile
                    };
                }
                UAIState::EndOfFile => {
                    break;
                }
            }
        }

        cfn
    }

    fn write_to_uai(&self, mut file: File) -> io::Result<()> {
        // preamble
        // - graph type, variables and domains
        let num_variables = self.num_variables();
        let domain_sizes: Vec<usize> = (0..self.num_variables())
            .map(|var| self.domain_size(var))
            .collect();
        write!(
            file,
            "MARKOV\n{}\n{}\n",
            num_variables,
            vec_to_string(&domain_sizes)
        )?;

        // - function scopes
        // -- number of functions
        write!(file, "{}\n", self.num_terms())?;
        // -- function scopes
        for term in &self.terms {
            // ---- number of variables, list of variables
            match term {
                CFNTerm::Unary(term) => {
                    write!(file, "1 {}\n", term.variable_idx)?;
                }
                CFNTerm::Pairwise(term) => {
                    let variables = self.hypergraph.hyperedge_endpoints(term.hyperedge_idx);
                    write!(file, "2 {}\n", vec_to_string(variables))?;
                }
                CFNTerm::General(term) => {
                    let variables = self.hypergraph.hyperedge_endpoints(term.hyperedge_idx);
                    let num_variables = variables.len();
                    write!(file, "{} {}\n", num_variables, vec_to_string(variables))?;
                }
            }
        }

        // function tables
        for term in &self.terms {
            // -- blank line, number of table values, table values
            match term {
                CFNTerm::Unary(term) => write!(
                    file,
                    "\n{}\n{}\n",
                    term.function_table.len(),
                    vec_to_string(&term.function_table.iter().collect::<Vec<_>>())
                )?,
                CFNTerm::Pairwise(term) => write!(
                    file,
                    "\n{}\n{}\n",
                    term.function_table.len(),
                    vec_to_string(&term.function_table.iter().collect::<Vec<_>>())
                )?,
                CFNTerm::General(term) => write!(
                    file,
                    "\n{}\n{}\n",
                    term.function_table.len(),
                    vec_to_string(&term.function_table.iter().collect::<Vec<_>>())
                )?,
            }
        }

        Ok(())
    }
}

impl ConstructRelaxation<MinimalEdges> for GeneralCFN {
    fn construct_relaxation(&self) -> RelaxationGraph {
        let mut graph = DiGraph::with_capacity(self.num_terms(), 2 * self.num_non_unary_terms());
        let mut node_index_of_term = Vec::with_capacity(self.num_terms());
        let mut variable_node_idx = Vec::with_capacity(self.num_variables());

        for variable_idx in self.hypergraph.iter_node_indices() {
            variable_node_idx.push(graph.add_node(CFNOrigin::Node(variable_idx)));
        }

        for term in &self.terms {
            match term {
                CFNTerm::Unary(term) => {
                    node_index_of_term.push(term.variable_idx); // same index as in CFN's hypergraph
                }
                CFNTerm::Pairwise(term) => {
                    let term_node_idx = graph.add_node(CFNOrigin::Hyperedge(term.hyperedge_idx));
                    node_index_of_term.push(term_node_idx.index());
                    for &variable in self.hypergraph.hyperedge_endpoints(term.hyperedge_idx) {
                        graph.add_edge(term_node_idx, variable_node_idx[variable], ());
                    }
                }
                CFNTerm::General(term) => {
                    let term_node_idx = graph.add_node(CFNOrigin::Hyperedge(term.hyperedge_idx));
                    node_index_of_term.push(term_node_idx.index());
                    for &variable in self.hypergraph.hyperedge_endpoints(term.hyperedge_idx) {
                        graph.add_edge(term_node_idx, variable_node_idx[variable], ());
                    }
                }
            }
        }

        RelaxationGraph {
            graph,
            node_index_of_term: node_index_of_term,
        }
    }
}
